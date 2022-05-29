mod cgi;

use {
    cgi::Cgi,
    crate::{
        CONFIG,
        config::Directive,
        request::{Request, RequestError},
    },
    std::{
        error::Error,
        fmt::Display,
        io::{BufReader, Read},
        path::PathBuf,
    }
};

#[derive(Debug)]
pub enum ServerError {
    NotFound,
    CgiError,
    Unauthorized,
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

pub enum Response {
    Success {
        mimetype: String,
        body: Vec<u8>,
    },
    Redirect(PathBuf),
    ClientError(RequestError),
    ServerError(ServerError),
}

impl From<Request> for Response {
    fn from(request: Request) -> Self {
        let server = match CONFIG.vhosts.get(&request.host) {
            Some(s) => s,
            None => return Self::ServerError(ServerError::NotFound),
        };
        for (dir, directive) in &server.directories {
            if request.path.starts_with(&dir) {
                match directive {
                    Directive::Allow(val) => {
                        if !val {
                            return Self::ServerError(ServerError::Unauthorized);
                        }
                    },
                    Directive::Alias(path) => {
                        let r = Request {
                            host: request.host,
                            path: PathBuf::from(path),
                            query: request.query,
                            length: request.length,
                            content: request.content,
                        };
                        return Self::from(r);
                    },
                    Directive::Redirect(path) => return Self::Redirect(path.to_path_buf()),
                    Directive::Cgi => {
                        let cgi = match Cgi::new(&request, &server, &dir) {
                            Ok(c) => c,
                            Err(e) => return Self::ServerError(e),
                        };
                        match cgi.run() {
                            Ok(output) => {
                                let idx = match output.stdout.iter().position(|&x| x == b'\n') {
                                    Some(i) => i,
                                    None => return Self::ServerError(ServerError::CgiError),
                                };
                                let mimetype = String::from_utf8_lossy(&output.stdout[0..idx]);
                                let body = Vec::from(&output.stdout[idx + 1..]);
                                return Self::Success { mimetype: mimetype.to_string(), body };
                            },
                            Err(_) => return Self::ServerError(ServerError::CgiError),
                        }
                    },
                }
            }
        }
        let mut path = server.root.clone();
        path.push(request.path);
        let fd = match std::fs::File::open(path) {
            Ok(f) => f,
            Err(e) => return Self::ServerError(ServerError::IoError(e)),
        };
        let mut reader = BufReader::new(fd);
        let mut buf = vec![];
        if let Err(e) = reader.read_to_end(&mut buf) {
            return Self::ServerError(ServerError::IoError(e));
        }
        Self::ServerError(ServerError::NotFound)
    }
}
