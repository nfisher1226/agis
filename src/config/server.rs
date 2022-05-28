use std::{collections::HashMap, path::PathBuf};

use serde::Deserialize;

#[derive(Default, Deserialize)]
pub struct Server {
    /// The domain name of this vserver
    pub name: String,
    /// Where the files are to be served from
    pub root: String,
    pub directories: HashMap<PathBuf, Directory>,
}

#[derive(Default, Deserialize)]
pub struct Directory {
    pub directives: Vec<Directive>,
}

#[derive(Deserialize)]
pub enum Directive {
    Allow(bool),
    Alias(String),
    Redirect(PathBuf),
    Script(String),
}
