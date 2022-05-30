use {
    crate::error::RequestError,
    std::{
        convert::TryFrom,
        io::{BufRead, BufReader, Read},
        net::TcpStream,
        path::PathBuf,
    },
};

pub struct Request {
    /// The fully qualified domain name of the host
    pub host: String,
    /// The absolute path of the requested document
    pub path: PathBuf,
    /// The optional query string
    pub query: Option<String>,
    /// The length of submitted content
    pub length: usize,
    /// Content to be uploaded
    pub content: Option<Vec<u8>>,
}

impl TryFrom<BufReader<&TcpStream>> for Request {
    type Error = RequestError;

    fn try_from(mut reader: BufReader<&TcpStream>) -> Result<Self, Self::Error> {
        let mut request_header = String::new();
        reader.read_line(&mut request_header)?;
        let parts: Vec<&str> = request_header.split_whitespace().collect();
        match parts.len() {
            1 => Err(RequestError::MissingSeparator),
            2 => Err(RequestError::MissingField),
            3 => {
                let length: usize = match parts[2].parse() {
                    Ok(l) => l,
                    Err(_) => return Err(RequestError::InvalidContentLength),
                };
                let content = match length {
                    0 => None,
                    length => {
                        let mut buf = Vec::with_capacity(length);
                        reader.read_exact(&mut buf)?;
                        Some(buf)
                    }
                };
                let (path, query) = if let Some((p, q)) = parts[1].split_once('?') {
                    (PathBuf::from(p), Some(q.to_string()))
                } else {
                    (PathBuf::from(&parts[1]), None)
                };
                Ok(Self {
                    host: parts[0].to_string(),
                    path,
                    query,
                    length,
                    content,
                })
            }
            _ => Err(RequestError::ExtraField),
        }
    }
}
