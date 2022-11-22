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
        path::{Path, PathBuf},
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
                write!(f, "Response::Success({mimetype})")
            }
            Self::Redirect(path) => write!(f, "Response::Redirect({})", path.display()),
            Self::ClientError(e) => write!(f, "Response::ClientError({e})"),
            Self::ServerError(e) => write!(f, "Response::ServerError({e})"),
        }
    }
}

impl From<Response> for Vec<u8> {
    fn from(response: Response) -> Self {
        match response {
            Response::Success { mimetype, mut body } => {
                let mut buf = format!("2 {mimetype}\r\n").into_bytes();
                buf.append(&mut body);
                buf
            }
            Response::Redirect(path) => format!("3 {}\r\n", path.display()).into_bytes(),
            Response::ClientError(e) => format!("4 {e}\r\n").into_bytes(),
            Response::ServerError(e) => format!("5 {e}\r\n").into_bytes(),
        }
    }
}

impl From<PathBuf> for Response {
    fn from(dir: PathBuf) -> Response {
        let contents = match fs::read_dir(dir) {
            Ok(c) => c,
            Err(e) => return Self::ServerError(e.into()),
        };
        let mut body = String::from("# Directory listing\n=> .. Parent\n");
        for entry in contents {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => return Self::ServerError(e.into()),
            };
            let entry = if let Some(e) = entry.file_name().to_str() {
                e.to_string()
            } else {
                let err = io::Error::new(ErrorKind::Other, "Invalid pathname");
                return Self::ServerError(err.into());
            };
            if let Err(e) = writeln!(body, "=> {entry}") {
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
            if PathBuf::from(&request.path).starts_with(dir) {
                match directive {
                    Directive::Allow(val) => {
                        if !val {
                            return ServerError::Unauthorized.into();
                        }
                    }
                    Directive::Alias(path) => {
                        let children = PathBuf::from(&request.path)
                            .strip_prefix(dir)
                            .map(Path::to_path_buf)
                            .unwrap();
                        let children = if children.starts_with("/") {
                            children.strip_prefix("/").map(Path::to_path_buf).unwrap()
                        } else {
                            children
                        };
                        let mut path = PathBuf::from(path);
                        path.push(children);
                        let path = path.to_string_lossy().to_string();
                        let r = Request { path, ..request };
                        return Self::from(r);
                    }
                    Directive::Redirect(path) => {
                        if PathBuf::from(&request.path).as_path() == dir.as_path() {
                            return Self::Redirect(path.clone());
                        }
                    }
                    Directive::Interpreter(_prog) => {
                        unimplemented!();
                    }
                    Directive::Cgi => {
                        let cgi = match Cgi::new(request, server, dir) {
                            Ok(c) => c,
                            Err(e) => return e.into(),
                        };
                        return cgi.into();
                    }
                    Directive::ScriptAlias(script) => {
                        let cgi = match Cgi::from_script_alias(request, server, script) {
                            Ok(c) => c,
                            Err(e) => return e.into(),
                        };
                        return cgi.into();
                    }
                }
            }
        }
        let mut path = server.root.clone();
        let request_base = match PathBuf::from(&request.path).strip_prefix("/") {
            Ok(p) => p.to_path_buf(),
            Err(e) => {
                let err = io::Error::new(ErrorKind::Other, e);
                return Self::ServerError(err.into());
            }
        };
        path.push(request_base);
        if path.is_dir() {
            if !request.path.ends_with('/') {
                let mut path = request.path.clone();
                path.push('/');
                return Self::Redirect(PathBuf::from(path));
            }
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
        let mut body = vec![];
        if let Err(e) = reader.read_to_end(&mut body) {
            return Self::ServerError(e.into());
        }
        let mimetype = match path.extension() {
            Some(ext) if ext == "gmi" => "text/gemini",
            _ => tree_magic_mini::from_u8(&body),
        }
        .to_string();
        Self::Success { mimetype, body }
    }
}
