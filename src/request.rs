use {
    crate::Config,
    std::{
        convert::TryFrom,
        path::PathBuf,
        usize,
        fmt::Display,
        error::Error,
        io::{BufRead, BufReader, Read},
        net::TcpStream,
    },
};

pub struct Request {
    pub host: String,
    pub path: PathBuf,
    pub length: usize,
    pub content: Option<Vec<u8>>,
}

#[derive(Debug)]
pub enum RequestError {
    MissingSeparator,
    MissingField,
    ExtraField,
    InvalidContentLength,
    ReadError(std::io::Error),
}

impl Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingSeparator => write!(f, "Missing separator"),
            Self::MissingField => write!(f, "Missing field"),
            Self::ExtraField => write!(f, "Extra field"),
            Self::InvalidContentLength => write!(f, "Invalid content length"),
            Self::ReadError(e) => write!(f, "Read error: {}", &e),
        }
    }
}

impl Error for RequestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ReadError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for RequestError {
    fn from(error: std::io::Error) -> Self {
        Self::ReadError(error)
    }
}

impl TryFrom<&mut BufReader<&TcpStream>> for Request {
    type Error = RequestError;

    fn try_from(reader: &mut BufReader<&TcpStream>) -> Result<Self, Self::Error> {
        let mut header = String::new();
        reader.read_line(&mut header)?;
        let parts: Vec<&str> = header
            .split_whitespace()
            .collect();
        match parts.len() {
            1 => Err(RequestError::MissingSeparator),
            2 => Err(RequestError::MissingField),
            3 => {
                let length: usize = match parts[2].parse() {
                    Ok(l) => l,
                    Err(_) => return Err(RequestError::InvalidContentLength),
                };
                let content = match length {
                    0 => None,
                    length => {
                        let mut buf = Vec::with_capacity(length);
                        reader.read_exact(&mut buf)?;
                        Some(buf)
                    },
                };
                Ok(Self {
                    host: parts[0].to_string(),
                    path: PathBuf::from(&parts[1]),
                    length,
                    content,
                })
            },
            _ => Err(RequestError::ExtraField),
        }
    }
}

impl Request {
    pub fn respond(&self) {
    }
}
