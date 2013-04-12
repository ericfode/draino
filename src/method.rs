/// HTTP Methods
pub enum Method { GET, HEAD, POST, PUT, DELETE, TRACE, OPTIONS, CONNECT, PATCH}

impl Method {
    pub fn from_str(method: &str) -> Method {
        match method {
            "GET"    => GET,
            "HEAD"   => HEAD,
            "POST"   => POST,
            "PUT"    => PUT,
            "DELETE" => DELETE,
            "TRACE"  => TRACE,
            "OPTIONS"=> OPTIONS,
            "CONNECT"=> CONNECT,
            "PATCH"  => PATCH,
            _        => {fail!(fmt!("Bad HTTP method: %?",method))}
        }
    }

}