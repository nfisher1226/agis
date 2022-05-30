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
        io::{BufReader, Read},
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
                        let r = Request {
                            host: request.host,
                            path: PathBuf::from(path),
                            query: request.query,
                            length: request.length,
                            content: request.content,
                        };
                        return Self::from(r);
                    }
                    Directive::Redirect(path) => return Self::Redirect(path.clone()),
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
        path.push(request.path);
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
