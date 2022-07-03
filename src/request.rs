use {
    crate::error::RequestError,
    std::{
        convert::TryFrom,
        fmt,
        io::{BufRead, BufReader, Read},
        net::{IpAddr, TcpStream},
    },
};

#[derive(Clone)]
/// Represents a valid request
pub struct Request {
    /// The fully qualified domain name of the host
    pub host: String,
    /// The absolute path of the requested document
    pub path: String,
    /// The optional query string
    pub query: Option<String>,
    /// Client Ip address
    pub client_ip: IpAddr,
    /// The length of submitted content
    pub length: usize,
    /// Content to be uploaded
    pub content: Option<Vec<u8>>,
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Request: {{ host: {}; path: {}; query: {}; client_ip: {}; length: {}; }}",
            &self.host,
            &self.path,
            self.query.as_ref().unwrap_or(&String::from("none")),
            self.client_ip,
            self.length,
        )
    }
}

impl TryFrom<&TcpStream> for Request {
    type Error = RequestError;

    fn try_from(stream: &TcpStream) -> Result<Self, Self::Error> {
        let mut reader = BufReader::new(stream);
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
                let (mut path, query) = if let Some((p, q)) = parts[1].split_once('?') {
                    (p.to_string(), Some(q.to_string()))
                } else {
                    (parts[1].to_string(), None)
                };
                if path.is_empty() {
                    path.push('/');
                }
                let client_ip = stream.peer_addr()?.ip();
                Ok(Self {
                    host: parts[0].to_string(),
                    path,
                    query,
                    client_ip,
                    length,
                    content,
                })
            }
            _ => Err(RequestError::ExtraField),
        }
    }
}
