mod cgi;

use {
    crate::{
        config::Directive,
        error::{RequestError, ServerError},
        request::Request,
        CONFIG,
    },
    cgi::Cgi,
    std::{
        fmt::{Display, Write},
        io::{BufReader, ErrorKind, Read},
        path::PathBuf,
    },
};

/// Represents the response which will be sent back to the client
pub enum Response {
    /// The resource is valid and will be served
    Success { mimetype: String, body: Vec<u8> },
    /// The client is directed to resubmit the request with a different Url path
    Redirect(PathBuf),
    /// The client sent a non-conforming request
    ClientError(RequestError),
    /// The server encountered an error processing a valid request
    ServerError(ServerError),
}

impl Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success { mimetype, body: _ } => {
                write!(f, "Response::Success({})", mimetype)
            }
            Self::Redirect(path) => write!(f, "Response::Redirect({})", path.display()),
            Self::ClientError(e) => write!(f, "Response::ClientError({})", &e),
            Self::ServerError(e) => write!(f, "Response::ServerError({})", &e),
        }
    }
}

impl From<Response> for Vec<u8> {
    fn from(response: Response) -> Self {
        match response {
            Response::Success { mimetype, mut body } => {
                let mut buf = format!("2 {}\r\n", mimetype).into_bytes();
                buf.append(&mut body);
                buf
            }
            Response::Redirect(path) => format!("3 {}\r\n", path.display()).into_bytes(),
            Response::ClientError(e) => format!("4 {}\r\n", e).into_bytes(),
            Response::ServerError(e) => format!("5 {}\r\n", e).into_bytes(),
        }
    }
}

impl From<PathBuf> for Response {
    fn from(dir: PathBuf) -> Response {
        let contents = match std::fs::read_dir(&dir) {
            Ok(c) => c,
            Err(e) => return Self::ServerError(ServerError::IoError(e)),
        };
        let mut body = String::from("# Directory listing\n=> .. Parent\n");
        for entry in contents {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => return Self::ServerError(ServerError::IoError(e)),
            };
            let entry = match entry.file_name().to_os_string().to_str() {
                Some(e) => e.to_string(),
                None => {
                    return Self::ServerError(ServerError::IoError(std::io::Error::new(
                        ErrorKind::Other,
                        "Invalid pathname",
                    )))
                }
            };
            if let Err(e) = writeln!(body, "=> {}", entry) {
                return Self::ServerError(ServerError::IoError(std::io::Error::new(
                    ErrorKind::Other,
                    e,
                )));
            }
        }
        Self::Success {
            mimetype: String::from("text/gemini"),
            body: body.into_bytes(),
        }
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
                    }
                    Directive::Alias(path) => {
                        let children = request.path.strip_prefix(dir).unwrap();
                        let children = if children.starts_with("/") {
                            children.strip_prefix("/").unwrap()
                        } else {
                            children
                        };
                        let mut path = PathBuf::from(path);
                        path.push(children);
                        let r = Request {
                            host: request.host,
                            path: PathBuf::from(path),
                            query: request.query,
                            length: request.length,
                            content: request.content,
                        };
                        return Self::from(r);
                    }
                    Directive::Redirect(path) => {
                        if request.path.as_path() == dir.as_path() {
                            return Self::Redirect(path.clone());
                        }
                    }
                    Directive::Cgi => {
                        let cgi = match Cgi::new(&request, server, dir) {
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
                                return Self::Success {
                                    mimetype: mimetype.to_string(),
                                    body,
                                };
                            }
                            Err(_) => return Self::ServerError(ServerError::CgiError),
                        }
                    }
                }
            }
        }
        let mut path = server.root.clone();
        let request_base = match request.path.strip_prefix("/") {
            Ok(p) => p,
            Err(e) => {
                let err = std::io::Error::new(ErrorKind::Other, e);
                return Self::ServerError(ServerError::IoError(err));
            }
        };
        path.push(request_base);
        if path.is_dir() {
            path.push("index.gmi");
            if !path.exists() {
                _ = path.pop();
                return path.into();
            }
        }
        let fd = match std::fs::File::open(&path) {
            Ok(f) => f,
            Err(e) => return Self::ServerError(ServerError::IoError(e)),
        };
        let mut reader = BufReader::new(fd);
        let mut buf = vec![];
        if let Err(e) = reader.read_to_end(&mut buf) {
            return Self::ServerError(ServerError::IoError(e));
        }
        let mimetype = match path.extension() {
            Some(ext) if ext == "gmi" => "text/gemini",
            _ => tree_magic_mini::from_u8(&buf),
        }
        .to_string();
        Self::Success {
            mimetype,
            body: buf,
        }
    }
}
