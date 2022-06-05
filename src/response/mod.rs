pub mod cgi;

use {
    crate::{
        config::Directive,
        error::{RequestError, ServerError},
        request::Request,
        CONFIG,
    },
    cgi::Cgi,
    std::{
        fmt::{self, Write},
        fs::{self, File},
        io::{self, BufReader, ErrorKind, Read},
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

impl From<ServerError> for Response {
    fn from(err: ServerError) -> Self {
        Self::ServerError(err)
    }
}

impl From<RequestError> for Response {
    fn from(err: RequestError) -> Self {
        Self::ClientError(err)
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
        let contents = match fs::read_dir(&dir) {
            Ok(c) => c,
            Err(e) => return Self::ServerError(e.into()),
        };
        let mut body = String::from("# Directory listing\n=> .. Parent\n");
        for entry in contents {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => return Self::ServerError(e.into()),
            };
            let entry = match entry.file_name().to_os_string().to_str() {
                Some(e) => e.to_string(),
                None => {
                    let err = io::Error::new(
                        ErrorKind::Other,
                        "Invalid pathname"
                    );
                    return Self::ServerError(err.into());
                }
            };
            if let Err(e) = writeln!(body, "=> {}", entry) {
                let err = io::Error::new(ErrorKind::Other, e);
                return Self::ServerError(err.into());
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
            None => return ServerError::NotFound.into(),
        };
        for (dir, directive) in &server.directories {
            if request.path.starts_with(&dir) {
                match directive {
                    Directive::Allow(val) => {
                        if !val {
                            return ServerError::Unauthorized.into();
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
                            path,
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
                        let cgi = match Cgi::new(request, server, dir) {
                            Ok(c) => c,
                            Err(e) => return e.into(),
                        };
                        match cgi.run() {
                            Ok(output) => {
                                let idx = match output.stdout.iter().position(|&x| x == b'\n') {
                                    Some(i) => i,
                                    None => return ServerError::CgiError.into(),
                                };
                                let mimetype = String::from_utf8_lossy(&output.stdout[0..idx]);
                                let body = Vec::from(&output.stdout[idx + 1..]);
                                return Self::Success {
                                    mimetype: mimetype.to_string(),
                                    body,
                                };
                            }
                            Err(_) => return ServerError::CgiError.into(),
                        }
                    }
                }
            }
        }
        let mut path = server.root.clone();
        let request_base = match request.path.strip_prefix("/") {
            Ok(p) => p,
            Err(e) => {
                let err = io::Error::new(ErrorKind::Other, e);
                return Self::ServerError(err.into());
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
        let fd = match File::open(&path) {
            Ok(f) => f,
            Err(e) => return Self::ServerError(e.into()),
        };
        let mut reader = BufReader::new(fd);
        let mut buf = vec![];
        if let Err(e) = reader.read_to_end(&mut buf) {
            return Self::ServerError(e.into());
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
