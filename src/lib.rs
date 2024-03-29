#![warn(clippy::all, clippy::pedantic)]
#![doc = include_str!("../README.md")]

/// Server configuration
pub mod config;
/// Possible errors
pub mod error;
/// Log access and errors
pub mod log;
/// Parses requests
pub mod request;
/// Prepares a resonse
pub mod response;
/// Creates and manages worker threads
pub mod threadpool;

use {
    getopts::{Fail, Matches, Options},
    log::{Log, LogError},
    once_cell::sync::Lazy,
    response::Response,
    std::{
        env,
        ffi::CString,
        fs::{self, File},
        io::{self, BufWriter, Write},
        net::TcpStream,
        os::unix::prelude::OsStrExt,
        process,
    },
};

pub use {config::Config, request::Request, threadpool::ThreadPool};

pub static CONFIG: Lazy<Config> = Lazy::new(|| match Config::load() {
    Ok(c) => c,
    Err(e) => {
        eprintln!("Unable to load config: {e}");
        process::exit(1);
    }
});

/// Drops priviledges after starting the server
/// # Safety
/// This function should only be called if it can be certain that both *user*
/// and *group* are valid pointers, which can be reliably gotten by calling the
/// `Config::getpwnam` and `Config::getgrnam` methods, respectively. Since those
/// methods will return an error if the user and group referred to in your
/// config.ron do not exist on the system, this function should not do dangerous
/// things.
/// # Errors
/// Returns the last OS error if setting the correct user or group permissions fail
pub unsafe fn privdrop(user: *mut libc::passwd, group: *mut libc::group) -> io::Result<()> {
    if libc::setgid((*group).gr_gid) != 0 {
        eprintln!("privdrop: Unable to setgid of group: {}", &CONFIG.group);
        return Err(std::io::Error::last_os_error());
    }
    if libc::setuid((*user).pw_uid) != 0 {
        eprintln!("privdrop: Unable to setuid of user: {}", &CONFIG.user);
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

/// Initializes the access and error logs if they don't exist
/// # Safety
/// This function uses a number of unsafe libc interfaces. It is only called at
/// startup time, and the unsafe code only runs if either log is missing.
/// # Errors
/// Returns an error if
/// * Unable to create logging directory
/// * Unable to create access or error log files
pub unsafe fn init_logs(user: libc::uid_t, group: libc::gid_t) -> Result<(), io::Error> {
    if let Some(log) = CONFIG.access_log.as_ref() {
        if let Some(parent) = log.parent() {
            if !parent.exists() {
                println!("Creating log directory");
                fs::create_dir_all(parent)?;
            }
        }
        if !log.exists() {
            println!("Creating access log");
            {
                File::create(log)?;
            }
            let logstr = CString::new(log.clone().as_os_str().as_bytes())?;
            println!("Setting access log permissions");
            _ = libc::chown(logstr.as_ptr(), user, group);
        }
    }
    if let Some(log) = CONFIG.error_log.as_ref() {
        if let Some(parent) = log.parent() {
            if !parent.exists() {
                println!("Creating log directory");
                fs::create_dir_all(parent)?;
            }
        }
        if !log.exists() {
            println!("Creating error log");
            {
                File::create(log)?;
            }
            let logstr = CString::new(log.clone().as_os_str().as_bytes())?;
            println!("Setting error log permissions");
            _ = libc::chown(logstr.as_ptr(), user, group);
        }
    }
    Ok(())
}

/// Attempts to parse a `Request` from the stream and to formulate
/// a `Response` based upon that input.
/// # Errors
/// Returns an `io::Error` if:
/// * Unable to log an error
/// * Unable to write to the `TcpStream` successfully
pub fn handle_connection(mut stream: TcpStream) -> Result<(), io::Error> {
    let (request, response) = match Request::try_from(&stream) {
        Ok(request) => (request.to_string(), Response::from(request)),
        Err(e) => (String::from("Malformed request"), e.into()),
    };
    let msg = response.to_string();
    match response {
        Response::Success {
            mimetype: _,
            body: _,
        }
        | Response::Redirect(_) => {
            request.log()?;
            msg.log()?;
        }
        Response::ClientError(_) | Response::ServerError(_) => {
            request.log_err()?;
            msg.log_err()?;
        }
    }
    let mut writer = BufWriter::new(&mut stream);
    writer.write_all(&Vec::from(response))?;
    Ok(())
}

/// Collects and parses command line arguments
/// # Errors
/// Returns `getopt::Fail` if unable to parse options
pub fn options() -> Result<Matches, Fail> {
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.optopt("c", "config", "Use NAME as config file", "NAME");
    opts.optflag("h", "help", "Print this help menu");
    opts.optflag("v", "version", "Print the program version");
    opts.parse(&args[1..])
}

/// Formulates a Usage string and prints it to stdout
pub fn usage() {
    let ustr = "_PROGNAME_ _VERSION_\n\
        The JeanGnie <jeang3nie@hitchhiker-linux.org>\n\
        A Spartan protocol server\n\
        \n\
        USAGE:\n    \
        _PROGNAME_ <OPTIONS>\n\
        \n\
        OPTIONS:\n    \
        -h, --help\n        \
        Print help information\n\
        \n\
        -v, --version\n        \
        Print the program version\n\
        \n\
        -c, --config <config>\n        \
        Use <config> as the config file";
    let ustr = ustr
        .replace("_PROGNAME_", env!("CARGO_PKG_NAME"))
        .replace("_VERSION_", env!("CARGO_PKG_VERSION"));
    println!("{ustr}");
}

pub fn version() {
    println!("{}", env!("CARGO_PKG_VERSION"));
}
