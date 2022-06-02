use std::fmt::Display;

use {
    crate::CONFIG,
    chrono::{DateTime, Utc},
    std::io::{BufWriter, Write},
};

pub trait Log {
    type Error;

    fn log(&self) -> Result<(), Self::Error>;
}

pub trait LogError {
    type Error;

    fn log_err(&self) -> Result<(), Self::Error>;
}

impl Log for std::string::String {
    type Error = std::io::Error;

    fn log(&self) -> Result<(), Self::Error> {
        let dt: DateTime<Utc> = Utc::now();
        let msg = format!("{} {};\n", dt.to_rfc3339(), self);
        match CONFIG.access_log.as_ref() {
            Some(log) => {
                let fd = std::fs::OpenOptions::new().append(true).open(log)?;
                let mut writer = BufWriter::new(fd);
                writer.write_all(msg.as_bytes())?;
            }
            None => print!("{}", msg),
        }
        Ok(())
    }
}

impl Log for crate::Response {
    type Error = std::io::Error;

    fn log(&self) -> Result<(), Self::Error> {
        let dt: DateTime<Utc> = Utc::now();
        match self {
            Self::Success {
                mimetype: _,
                body: _,
            }
            | Self::Redirect(_) => {
                let msg = format!("{} {};\n", dt.to_rfc3339(), self);
                match CONFIG.access_log.as_ref() {
                    Some(log) => {
                        let fd = std::fs::OpenOptions::new().append(true).open(log)?;
                        let mut writer = BufWriter::new(fd);
                        writer.write_all(msg.as_bytes())?;
                    }
                    None => print!("{}", msg),
                }
            }
            Self::ClientError(_) | Self::ServerError(_) => {
                let msg = format!("{} {};\n", dt.to_rfc3339(), self);
                match CONFIG.error_log.as_ref() {
                    Some(log) => {
                        let fd = std::fs::OpenOptions::new().append(true).open(log)?;
                        let mut writer = BufWriter::new(fd);
                        writer.write_all(msg.as_bytes())?;
                    }
                    None => print!("{}", msg),
                }
            }
        }
        Ok(())
    }
}

impl<T> LogError for T
where
    T: Display,
{
    type Error = std::io::Error;

    fn log_err(&self) -> Result<(), Self::Error> {
        let dt: DateTime<Utc> = Utc::now();
        let msg = format!("{} {}\n", dt.to_rfc3339(), self);
        match CONFIG.error_log.as_ref() {
            Some(l) => {
                let fd = std::fs::OpenOptions::new().append(true).open(l)?;
                let mut writer = BufWriter::new(fd);
                writer.write_all(msg.as_bytes())?;
            }
            None => eprint!("{}", msg),
        }
        Ok(())
    }
}
