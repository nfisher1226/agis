use std::{collections::HashMap, path::PathBuf};

use serde::Deserialize;

#[derive(Default, Deserialize)]
pub struct Server {
    /// The domain name of this vserver
    pub name: String,
    /// Where the files are to be served from
    pub root: PathBuf,
    /// Directory specific directives
    pub directories: HashMap<PathBuf, Directive>,
}

#[derive(Deserialize)]
pub enum Directive {
    /// Whether to allow or deny access to this path
    Allow(bool),
    /// Causes requests for this path to be served from a different path
    Alias(String),
    /// Causes to server to send a redirect code back to the client
    Redirect(PathBuf),
    /// Paths under this directory are Common Gateway Interface programs
    Cgi,
}
