use {
    super::Request,
    crate::{config::Server, response::ServerError, CONFIG},
    std::{
        path::Path,
        process::{Command, Output},
    },
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

impl Cgi {
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
            request_uri: format!(
                "{}{}",
                request.path.display(),
                request.query.as_ref().unwrap_or(&"".to_string())
            ),
            script_filename: format!("{}", script_filename.display()),
            script_name: format!("{}", script_name.display()),
            server_name: server.name.clone(),
            server_port: CONFIG.port.clone(),
            server_software,
        })
    }

    pub fn run(&self) -> std::io::Result<Output> {
        Command::new(&self.script_filename)
            .envs([
                ("DOCUMENT_ROOT", &self.document_root),
                ("QUERY_STRING", &self.query_string),
                ("REQUEST_URI", &self.request_uri),
                ("SCRIPT_FILENAME", &self.script_filename),
                ("SCRIPT_NAME", &self.script_name),
                ("SERVER_NAME", &self.server_name),
                ("SERVER_PORT", &self.server_port),
                ("SERVER_SOFTWARE", &self.server_software),
            ])
            .output()
    }
}
