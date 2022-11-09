//! This module handles CGI (Common Gateway Interface) requests. The spec is a
//! subset of the CGI 1.1 spec, with http specific environment variables omitted
//! and the addition of the ability to handle a request body saved to a temporary
//! file. The CGI environment variables which are passed to the program are as
//! follows:
//! - DOCUMENT_ROOT is the document root of the virtual host serving this request.
//! - QUERY_STRING is the portion of the request following the '?' character,
//!   useful for setting additional variables.
//! - REMOTE_ADDR is the ip address of the client making the request
//! - REQUEST_URI is the interpreted pathname of the requested document or CGI
//!   (relative to the document root).
//! - SCRIPT_FILENAME is the full filesystem path to the CGI program
//! - SCRIPT_NAME is the interpreted pathname of the current CGI (relative to
//!   the document root).
//! - SERVER_NAME is the server's fully qualified domain name.
//! - SERVER_SOFTWARE is the name and version string of this server.
//! - REQUEST_BODY is the path to a temporary file which contains the request
//!   body. This variable will be an empty string if there was no request body.
//!   The file that it points to may contain any arbitrary data and should as
//!   such be treated as untrusted input.
use {
    super::Request,
    crate::{config::Server, response::ServerError, CONFIG},
    std::{
        fs::File,
        io::{self, Write},
        path::{Path, PathBuf},
        process::{Command, Output},
    },
};

/// The data to be passed into the CGI environment
pub struct Cgi {
    document_root: String,
    query_string: String,
    remote_addr: String,
    request_uri: String,
    script_filename: String,
    script_name: String,
    server_name: String,
    server_port: String,
    server_software: String,
    body: Option<Vec<u8>>,
}

impl Cgi {
    /// Constructs the Cgi struct from a `Request`, `Server` and a path
    pub fn new(request: Request, server: &Server, dir: &Path) -> Result<Self, ServerError> {
        let base = match PathBuf::from(&request.path).strip_prefix(dir) {
            Ok(b) => b.to_path_buf(),
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
        script_filename.push(script_name.strip_prefix("/").unwrap_or(&script_name));
        let query_string = match &request.query {
            Some(q) => q.to_string(),
            None => String::new(),
        };
        let server_software = format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        Ok(Self {
            document_root: format!("{}", server.root.display()),
            query_string,
            remote_addr: format!("{}", request.client_ip),
            request_uri: format!(
                "{}{}",
                &request.path,
                request
                    .query
                    .as_ref()
                    .map_or("", std::string::String::as_str)
            ),
            script_filename: format!("{}", script_filename.display()),
            script_name: format!("{}", script_name.display()),
            server_name: server.name.clone(),
            server_port: CONFIG.address.port.clone(),
            server_software,
            body: request.content,
        })
    }

    /// Formulates a `Response` from the output of a CGI script which has been
    /// aliased to a path
    pub fn from_script_alias(
        request: Request,
        server: &Server,
        script_alias: &Path,
    ) -> Result<Self, ServerError> {
        let script_name = match script_alias.file_name() {
            Some(name) => name.to_string_lossy(),
            None => return Err(ServerError::CgiError),
        };
        let mut script_filename = server.root.clone();
        script_filename.push(script_alias.strip_prefix("/").unwrap_or(script_alias));
        let query_string = match &request.query {
            Some(q) => q.to_string(),
            None => String::new(),
        };
        let server_software = format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        Ok(Self {
            document_root: format!("{}", server.root.display()),
            query_string,
            remote_addr: format!("{}", request.client_ip),
            request_uri: match request.query {
                Some(q) => format!("{}?{q}", &request.path),
                None => request.path.to_string(),
            },
            script_filename: format!("{}", script_filename.display()),
            script_name: format!("{}", script_name),
            server_name: server.name.clone(),
            server_port: CONFIG.address.port.clone(),
            server_software,
            body: request.content,
        })
    }

    /// Runs the CGI program and returns it's output
    pub fn run(&self) -> io::Result<Output> {
        let dir = tempfile::tempdir()?;
        let tmpfile = match self.body.as_ref() {
            Some(body) => {
                let path = dir.path().join("body");
                let mut fd = File::create(&path)?;
                fd.write_all(body)?;
                path.display().to_string()
            }
            None => String::new(),
        };
        Command::new(&self.script_filename)
            .env_clear()
            .envs([
                ("PATH", "/usr/local/bin:/usr/bin:/bin"),
                ("DOCUMENT_ROOT", &self.document_root),
                ("QUERY_STRING", &self.query_string),
                ("REMOTE_ADDR", &self.remote_addr),
                ("REQUEST_URI", &self.request_uri),
                ("SCRIPT_FILENAME", &self.script_filename),
                ("SCRIPT_NAME", &self.script_name),
                ("SERVER_NAME", &self.server_name),
                ("SERVER_PORT", &self.server_port),
                ("SERVER_SOFTWARE", &self.server_software),
                ("REQUEST_BODY", &tmpfile),
            ])
            .output()
    }
}
