#![allow(clippy::module_name_repetitions)]
use {
    crate::CONFIG,
    chrono::Utc,
    std::{
        fmt::Display,
        fs::OpenOptions,
        io::{self, BufWriter, Write},
    },
};

/// Logging access to the server
pub trait Log {
    type Error;

    /// Writes server access to either the configured access log or stdout
    /// # Errors
    /// Returns an error (usually an `io::Error`) if unable to write to the
    /// log file
    fn log(&self) -> Result<(), Self::Error>;
}

/// Logging server errors
pub trait LogError {
    type Error;

    /// Writes errors to either the configured error log or stderr
    /// # Errors
    /// Returns an error (usually an `io::Error`) if unable to write to the
    /// log file
    fn log_err(&self) -> Result<(), Self::Error>;
}

impl Log for std::string::String {
    type Error = io::Error;

    fn log(&self) -> Result<(), Self::Error> {
        let dt = Utc::now().to_rfc3339();
        let msg = format!("{dt} {self};\n");
        match CONFIG.access_log.as_ref() {
            Some(log) => match OpenOptions::new().append(true).open(log) {
                Ok(fd) => {
                    let mut writer = BufWriter::new(fd);
                    writer.write_all(msg.as_bytes())?;
                }
                Err(e) => {
                    eprintln!("{e}");
                    print!("{msg}");
                }
            },
            None => print!("{msg}"),
        }
        Ok(())
    }
}

impl Log for crate::Response {
    type Error = io::Error;

    fn log(&self) -> Result<(), Self::Error> {
        let dt = Utc::now().to_rfc3339();
        match self {
            Self::Success {
                mimetype: _,
                body: _,
            }
            | Self::Redirect(_) => {
                let msg = format!("{dt} {self};\n");
                match CONFIG.access_log.as_ref() {
                    Some(log) => match OpenOptions::new().append(true).open(log) {
                        Ok(fd) => {
                            let mut writer = BufWriter::new(fd);
                            writer.write_all(msg.as_bytes())?;
                        }
                        Err(e) => {
                            eprintln!("{e}");
                            print!("{msg}");
                        }
                    },
                    None => print!("{msg}"),
                }
            }
            Self::ClientError(_) | Self::ServerError(_) => {
                let msg = format!("{dt} {self};\n");
                match CONFIG.error_log.as_ref() {
                    Some(log) => match OpenOptions::new().append(true).open(log) {
                        Ok(fd) => {
                            let mut writer = BufWriter::new(fd);
                            writer.write_all(msg.as_bytes())?;
                        }
                        Err(e) => {
                            eprintln!("{e}");
                            eprint!("{msg}");
                        }
                    },
                    None => print!("{msg}"),
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
    type Error = io::Error;

    fn log_err(&self) -> Result<(), Self::Error> {
        let dt = Utc::now().to_rfc3339();
        let msg = format!("{dt} {self}\n");
        match CONFIG.error_log.as_ref() {
            Some(log) => match OpenOptions::new().append(true).open(log) {
                Ok(fd) => {
                    let mut writer = BufWriter::new(fd);
                    writer.write_all(msg.as_bytes())?;
                }
                Err(e) => {
                    eprintln!("{e}");
                    eprint!("{msg}");
                }
            },
            None => eprint!("{msg}"),
        }
        Ok(())
    }
}
