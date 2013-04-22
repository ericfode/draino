/// HTTP Methods
pub enum Method { GET, HEAD, POST, PUT, DELETE, TRACE, OPTIONS, CONNECT, PATCH}

impl Method {
    pub fn from_str(method: &str) -> Option<Method> {
        match method {
            "GET"    => Some(GET),
            "HEAD"   => Some(HEAD),
            "POST"   => Some(POST),
            "PUT"    => Some(PUT),
            "DELETE" => Some(DELETE),
            "TRACE"  => Some(TRACE),
            "OPTIONS"=> Some(OPTIONS),
            "CONNECT"=> Some(CONNECT),
            "PATCH"  => Some(PATCH),
             _       => None
        }
    }


}