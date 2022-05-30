/// Server configuration
pub mod config;
/// Possible errors
pub mod error;
/// Parses requests
pub mod request;
/// Prepares a resonse
pub mod response;
/// Creates and manages worker threads
pub mod threadpool;

use {
    lazy_static::lazy_static,
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

/// Handles the connection
pub fn handle_connection(mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let reader = BufReader::new(&stream);
    let request = Request::try_from(reader)?;
    let response: Vec<u8> = Response::from(request).into();
    let mut writer = BufWriter::new(&mut stream);
    writer.write_all(&response)?;
    Ok(())
}
