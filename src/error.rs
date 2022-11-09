#![allow(clippy::module_name_repetitions)]
use std::{error::Error, fmt, io, string::FromUtf8Error};

#[derive(Debug)]
/// Errors which might occur while parsing a request
pub enum RequestError {
    /// The request header did not have a CrLf termination
    MissingSeparator,
    /// The request header was missing one or more fields
    MissingField,
    /// The request header had one or more extra fields
    ExtraField,
    /// The content length was not a valid number
    InvalidContentLength,
    /// The request was not valid utf8
    InvalidUtf8,
    /// There was an error reading the request
    ReadError(std::io::Error),
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSeparator => write!(f, "Missing separator"),
            Self::MissingField => write!(f, "Missing field"),
            Self::ExtraField => write!(f, "Extra field"),
            Self::InvalidContentLength => write!(f, "Invalid content length"),
            Self::InvalidUtf8 => write!(f, "Utf8 error"),
            Self::ReadError(e) => write!(f, "Read error: {e}"),
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

impl From<io::Error> for RequestError {
    fn from(error: io::Error) -> Self {
        Self::ReadError(error)
    }
}

impl From<FromUtf8Error> for RequestError {
    fn from(_: FromUtf8Error) -> Self {
        Self::InvalidUtf8
    }
}

#[derive(Debug)]
/// Errors which might occur while processing a valid request into a response
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

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "Resource not found"),
            Self::CgiError => write!(f, "Script failed"),
            Self::Unauthorized => write!(f, "Not authorized"),
            Self::IoError(e) => write!(f, "Io error: {e}"),
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

impl From<io::Error> for ServerError {
    fn from(error: io::Error) -> Self {
        Self::IoError(error)
    }
}
