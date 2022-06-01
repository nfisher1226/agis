use std::io::BufWriter;

use {
    crate::CONFIG,
    std::io::Write,
};

pub trait Log {
    type Error;

    fn log(&self) -> Result<(), Self::Error>;
}

pub trait LogError {
    type Error;

    fn log_err(&self) -> Result<(), Self::Error>;
}

impl Log for crate::Request {
    type Error = std::io::Error;

    fn log(&self) -> Result<(), Self::Error> {
        let msg = format!(
            "  Request:\n    host: {}\n    path: {}\n    query: {}\n    length: {}\n",
            &self.host,
            self.path.display(),
            self.query.as_ref().unwrap_or(&String::from("none")),
            self.length,
        );
        match CONFIG.access_log.as_ref() {
            Some(log) => {
                let fd = std::fs::OpenOptions::new()
                    .append(true)
                    .open(log)?;
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
        match self {
            Self::Success { mimetype: _, body: _ } |
            Self::Redirect(_) => {
                match CONFIG.access_log.as_ref() {
                    Some(log) => {
                        let msg = format!("  {}\n", self);
                        let fd = std::fs::OpenOptions::new()
                            .append(true)
                            .open(log)?;
                        let mut writer = BufWriter::new(fd);
                        writer.write_all(msg.as_bytes())?;
                    },
                    None => println!("  {}", self),
                }
            },
            Self::ClientError(_) |
            Self::ServerError(_) => {
                match CONFIG.error_log.as_ref() {
                    Some(log) => {
                        let msg = format!("  {}\n", self);
                        let fd = std::fs::OpenOptions::new()
                            .append(true)
                            .open(log)?;
                        let mut writer = BufWriter::new(fd);
                        writer.write_all(msg.as_bytes())?;
                    },
                    None => println!("  {}", self),
                }
            },
        }
        Ok(())
    }
}

impl<T> LogError for T
where T: std::error::Error
{
    type Error = std::io::Error;

    fn log_err(&self) -> Result<(), Self::Error> {
        match CONFIG.error_log.as_ref() {
            Some(l) => {
                let fd = std::fs::OpenOptions::new()
                    .append(true)
                    .open(l)?;
                let mut writer = BufWriter::new(fd);
                writer.write_all(self.to_string().as_bytes())?;
            },
            None => eprint!("{}", self),
        }
        Ok(())
    }
}
