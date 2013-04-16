/// HTTP Methods
pub enum Method { GET, HEAD, POST, PUT, DELETE, TRACE, OPTIONS, CONNECT, PATCH}

impl Method {
    pub fn from_str(method: &str) -> Option<Method> {
        match method {
            "GET"    => Some(GET),
            "HEAD"   => Some(HEAD),
             _        => None
        }
    }

}