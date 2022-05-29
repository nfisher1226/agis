use {
    crate::{
        CONFIG,
        config::Server,
        response::ServerError,
    },
    std::{
        path::Path,
        process::{Command, Output},
    },
    super::Request,
};

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
    pub fn new(request: &Request, server: &Server, dir: &Path) -> Result<Self, ServerError> {
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

    pub fn run(&self) -> std::io::Result<Output> {
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
