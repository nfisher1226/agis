use {
    crate::request::RequestError,
    std::{
        error::Error,
        fmt::Display,
        path::PathBuf,
    }
};

#[derive(Debug)]
pub enum ServerError {
    NotFound,
    CgiError,
    IoError(std::io::Error),
}

impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "Resource not found"),
            Self::CgiError => write!(f, "Script failed"),
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

pub enum Response {
    Success {
        mimetype: String,
        body: Vec<u8>,
    },
    Redirect(PathBuf),
    ClientError(RequestError),
    ServerError(ServerError),
}
