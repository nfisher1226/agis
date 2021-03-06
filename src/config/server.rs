use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf};

#[derive(Deserialize)]
/// A name-based virtual host
pub struct Server {
    /// The domain name of this vserver
    pub name: String,
    /// Where the files are to be served from
    pub root: PathBuf,
    /// Directory specific directives
    pub directories: HashMap<PathBuf, Directive>,
}

#[derive(Deserialize)]
/// Path specific directives
pub enum Directive {
    /// Denies access to this path
    Allow(bool),
    /// Causes requests for this path to be served from a different path
    Alias(String),
    /// Causes to server to send a redirect code back to the client
    Redirect(PathBuf),
    /// Files under this directory will be processed by the given interpreter
    Interpreter(String),
    /// Paths under this directory are Common Gateway Interface programs
    Cgi,
    /// Paths under this directory will run <script>
    ScriptAlias(PathBuf),
}

impl Default for Server {
    fn default() -> Self {
        Self {
            name: String::from("example.com"),
            root: PathBuf::from("/srv/spartan"),
            directories: HashMap::from([(PathBuf::from("/"), Directive::Allow(true))]),
        }
    }
}
