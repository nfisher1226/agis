#![allow(clippy::unsafe_derive_deserialize)]
use {
    serde::{Deserialize, Serialize},
    std::{
        collections::HashMap,
        ffi::CString,
        fs,
        io::{Error, ErrorKind},
    },
};

/// A name based Virtual Host
mod server;

pub use server::{Directive, Server};

#[derive(Deserialize, Serialize)]
pub struct Config {
    /// The ip address to bind to
    pub address: String,
    /// The port to run on
    pub port: String,
    /// The user the server should run as
    pub user: String,
    /// The group the server should run as
    pub group: String,
    /// The number of worker threads to launch
    pub threads: usize,
    /// The Virtual Hosts to serve
    pub vhosts: HashMap<String, Server>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            address: String::from("0.0.0.0"),
            port: String::from("300"),
            user: String::from("agis"),
            group: String::from("agis"),
            threads: 4,
            vhosts: HashMap::from([(String::from("example.com"), Server::default())]),
        }
    }
}

impl Config {
    /// Loads the server configuration from file
    /// # Errors
    /// Returns an `io::Error` if the file cannot be read or if it is invalid
    pub fn load() -> Result<Self, Error> {
        let raw = fs::read_to_string("/etc/agis/config.ron")?;
        match ron::de::from_str(&raw) {
            Ok(c) => Ok(c),
            Err(e) => {
                let err = format!(
                    "Error encoding config:\n  code: {:?}\n  position:\n    line: {}\n    column: {}",
                    e.code,
                    e.position.line,
                    e.position.col,
                );
                Err(Error::new(ErrorKind::Other, err))
            },
        }
    }

    /// Gets the `libc::passwd` for the user that the server will run as
    /// # Errors
    /// Returns an `io::Error` if unable to create a `CString`
    pub fn getpwnam(&self) -> Result<*mut libc::passwd, std::io::Error> {
        let user = CString::new(self.user.as_bytes())?;
        let uid = unsafe { libc::getpwnam(user.as_ptr()) };
        if uid.is_null() {
            eprintln!("Unable to getpwnam of user: {}", &self.user);
            return Err(Error::last_os_error());
        }
        Ok(uid)
    }

    /// Gets the `libc::group` for the group that the server will run as
    /// # Errors
    /// Returns an `io::Error` if unable to create a `CString`
    pub fn getgrnam(&self) -> Result<*mut libc::group, std::io::Error> {
        let group = CString::new(self.group.as_bytes())?;
        let gid = unsafe { libc::getgrnam(group.as_ptr()) };
        if gid.is_null() {
            eprintln!("Unable to get getgrnam of group: {}", &self.group);
            return Err(Error::last_os_error());
        }
        Ok(gid)
    }
}
