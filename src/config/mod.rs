use {
    serde::Deserialize,
    std::{
        collections::HashMap,
        ffi::CString,
        fs,
        io::{Error, ErrorKind},
    },
};

mod server;

pub use server::{Directive, Server};

#[derive(Deserialize)]
pub struct Config {
    /// The ip address to bind to
    pub address: String,
    /// The port to run on
    pub port: String,
    /// The user the server should run as
    pub user: String,
    /// The group the server should run as
    pub group: String,
    pub threads: usize,
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
            vhosts: HashMap::new(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, Error> {
        let raw = fs::read_to_string("/etc/agis/config.toml")?;
        match toml::from_str(&raw) {
            Ok(c) => Ok(c),
            Err(_) => Err(Error::new(ErrorKind::Other, "Error decoding config file")),
        }
    }

    pub fn getpwnam(&self) -> Result<*mut libc::passwd, std::io::Error> {
        let user = CString::new(self.user.as_bytes())?;
        let uid = unsafe { libc::getpwnam(user.as_ptr()) };
        if uid.is_null() {
            eprintln!("Unable to getpwnam of user: {}", &self.user);
            return Err(Error::last_os_error());
        }
        Ok(uid)
    }

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

