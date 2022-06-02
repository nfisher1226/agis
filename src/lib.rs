use std::{ffi::CString, os::unix::prelude::OsStrExt};

use log::LogError;

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
    lazy_static::lazy_static,
    log::Log,
    response::Response,
    std::{
        error::Error,
        io::{BufReader, BufWriter, Write},
        net::TcpStream,
        process,
    },
};

pub use {config::Config, request::Request, threadpool::ThreadPool};

lazy_static! {
    pub static ref CONFIG: Config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Unable to load config: {e}");
            process::exit(1);
        }
    };
}

/// Drops priviledges after starting the server
/// # Safety
/// This function should only be called if it can be certain that both *user*
/// and *group* are valid pointers, which can be reliably gotten by calling the
/// `Config::getpwnam` and `Config::getgrnam` methods, respectively. Since those
/// methods will return an error if the user and group referred to in your
/// config.ron do not exist on the system, this function should not do dangerous
/// things.
pub unsafe fn privdrop(user: *mut libc::passwd, group: *mut libc::group) -> std::io::Result<()> {
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
///
/// # Safety
/// This function uses a number of unsafe libc interfaces. It is only called at
/// startup time, and the unsafe code only runs if either log is missing.
pub unsafe fn init_logs(uid: libc::uid_t, gid: libc::gid_t) -> Result<(), std::io::Error> {
    if let Some(log) = CONFIG.access_log.as_ref() {
        if let Some(parent) = log.parent() {
            if !parent.exists() {
                println!("Creating log directory");
                std::fs::create_dir_all(parent)?;
            }
        }
        if !log.exists() {
            println!("Creating access log");
            {
                std::fs::File::create(&log)?;
            }
            let logstr = CString::new(log.clone().as_os_str().as_bytes())?;
            println!("Setting access log permissions");
            _ = libc::chown(logstr.as_ptr(), uid, gid);
        }
    }
    if let Some(log) = CONFIG.error_log.as_ref() {
        if let Some(parent) = log.parent() {
            if !parent.exists() {
                println!("Creating log directory");
                std::fs::create_dir_all(parent)?;
            }
        }
        if !log.exists() {
            println!("Creating error log");
            {
                std::fs::File::create(&log)?;
            }
            let logstr = CString::new(log.clone().as_os_str().as_bytes())?;
            println!("Setting error log permissions");
            _ = libc::chown(logstr.as_ptr(), uid, gid);
        }
    }
    Ok(())
}

/// Handles the connection
pub fn handle_connection(mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let reader = BufReader::new(&stream);
    let request = match Request::try_from(reader) {
        Ok(r) => r,
        Err(e) => {
            e.log_err()?;
            return Err(e.into());
        }
    };
    let msg = request.to_string();
    let response = Response::from(request);
    match response {
        Response::Success {
            mimetype: _,
            body: _,
        }
        | Response::Redirect(_) => msg.log()?,
        Response::ClientError(_) | Response::ServerError(_) => msg.log_err()?,
    }
    response.log()?;
    let mut writer = BufWriter::new(&mut stream);
    writer.write_all(&Vec::from(response))?;
    Ok(())
}
