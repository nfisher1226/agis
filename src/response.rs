use {
    crate::{
        CONFIG,
        config::{Directive, Server},
        request::{Request, RequestError},
    },
    std::{
        error::Error,
        fmt::Display,
        io::{BufReader, Read, Write},
        path::{Path, PathBuf},
        process::{Command, Output},
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

pub struct Cgi {
    document_root: String,
    query_string: String,
    request_uri: String,
    script_filename: String,
    script_name: String,
    server_name: String,
    server_port: String,
    server_software: String,
}

impl  Cgi {
    fn new(request: &Request, server: &Server, dir: &Path) -> Result<Self, ServerError> {
        let base = match request.path.strip_prefix(dir) {
            Ok(b) => b,
            Err(_) => return Err(ServerError::CgiError),
        };
        let mut parts = base.components();
        let script_base = match parts.next() {
            Some(s) => s,
            None => return Err(ServerError::CgiError),
        };
        let mut script_name = dir.to_path_buf();
        script_name.push(script_base);
        let mut script_filename = server.root.clone();
        script_filename.push(&script_name);
        let query_string = match &request.query {
            Some(q) => q.to_string(),
            None => String::new(),
        };
        let server_software = format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        Ok(Self {
            document_root: format!("{}", server.root.display()),
            query_string,
            request_uri: format!("{}{}", request.path.display(), request.query.as_ref().unwrap_or(&"".to_string())),
            script_filename: format!("{}", script_filename.display()),
            script_name: format!("{}", script_name.display()),
            server_name: server.name.clone(),
            server_port: CONFIG.port.clone(),
            server_software,
        })
    }

    fn run(&self) -> std::io::Result<Output> {
        Command::new(&self.script_filename)
            .env("DOCUMENT_ROOT", &self.document_root)
            .env("QUERY_STRING", &self.query_string)
            .env("REQUEST_URI", &self.request_uri)
            .env("SCRIPT_FILENAME", &self.script_filename)
            .env("SCRIPT_NAME", &self.script_name)
            .env("SERVER_NAME", &self.server_name)
            .env("SERVER_PORT", &self.server_port)
            .env("SERVER_SOFTWARE", &self.server_software)
            .output()
    }
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
