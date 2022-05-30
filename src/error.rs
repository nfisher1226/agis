#![allow(clippy::module_name_repetitions)]
use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum RequestError {
    /// The request header did not have a CrLf termination
    MissingSeparator,
    /// The request header was missing one or more fields
    MissingField,
    /// The request header had one or more extra fields
    ExtraField,
    /// The content length was not a valid number
    InvalidContentLength,
    /// There was an error reading the request
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

#[derive(Debug)]
pub enum ServerError {
    /// The requested resource does not exist
    NotFound,
    /// A Cgi program encountered an error
    CgiError,
    /// The requested path is not authorized
    Unauthorized,
    /// The server encountered an io error
    IoError(std::io::Error),
}

impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "Resource not found"),
            Self::CgiError => write!(f, "Script failed"),
            Self::Unauthorized => write!(f, "Not authorized"),
            Self::IoError(e) => write!(f, "Io error: {}", &e),
        }
    }
}

impl Error for ServerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ServerError {
    fn from(error: std::io::Error) -> Self {
        Self::IoError(error)
    }
}
