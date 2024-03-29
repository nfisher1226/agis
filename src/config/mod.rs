#![allow(clippy::unsafe_derive_deserialize, clippy::module_name_repetitions)]
use {
    serde::Deserialize,
    std::{
        collections::HashMap,
        ffi::CString,
        fs,
        io::{Error, ErrorKind},
        path::PathBuf,
    },
};

/// A name based Virtual Host
mod server;

pub use server::{Directive, Server};

#[derive(Deserialize)]
pub struct Address {
    /// The ip address to bind to
    pub ip: String,
    /// The port to run on
    pub port: String,
}

impl Default for Address {
    fn default() -> Self {
        Self {
            ip: String::from("0.0.0.0"),
            port: String::from("300"),
        }
    }
}

#[derive(Deserialize)]
/// Configuration variables for the server
pub struct Config {
    /// The ip address and port to bind to
    pub address: Address,
    /// An optional second address, in case of running both ipv4 and ipv6
    pub address1: Option<Address>,
    /// The user the server should run as
    pub user: String,
    /// The group the server should run as
    pub group: String,
    /// The number of worker threads to launch
    pub threads: usize,
    /// Access log
    pub access_log: Option<PathBuf>,
    /// Error log
    pub error_log: Option<PathBuf>,
    /// The Virtual Hosts to serve
    pub vhosts: HashMap<String, Server>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            address: Address::default(),
            address1: None,
            user: String::from("agis"),
            group: String::from("agis"),
            threads: 4,
            access_log: Some(PathBuf::from("/var/log/agis/access.log")),
            error_log: Some(PathBuf::from("/var/log/agis/error.log")),
            vhosts: HashMap::from([(String::from("example.com"), Server::default())]),
        }
    }
}

impl Config {
    /// Loads the server configuration from file
    /// # Errors
    /// Returns an `io::Error` if the file cannot be read or if it is invalid
    /// # Panics
    /// Will panic if unable to get the command line options
    pub fn load() -> Result<Self, Error> {
        let opts = crate::options().unwrap();
        let cfg = opts
            .opt_str("c")
            .unwrap_or_else(|| "/etc/agis/config.ron".to_string());
        let raw = fs::read_to_string(cfg)?;
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
            }
        }
    }

    /// Gets the `libc::passwd` for the user that the server will run as
    /// # Errors
    /// Returns an `io::Error` if unable to create a `CString`
    pub fn getpwnam(&self) -> Result<*mut libc::passwd, Error> {
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
    pub fn getgrnam(&self) -> Result<*mut libc::group, Error> {
        let group = CString::new(self.group.as_bytes())?;
        let gid = unsafe { libc::getgrnam(group.as_ptr()) };
        if gid.is_null() {
            eprintln!("Unable to get getgrnam of group: {}", &self.group);
            return Err(Error::last_os_error());
        }
        Ok(gid)
    }
}
