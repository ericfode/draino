extern mod std;
use std::net::{ip, tcp};
use std::{uv, task};
use hashmap = core::hashmap::linear;

use statuscodes::StatusCode;
use request::Request;
use response::Response;

mod statuscodes {





    pub struct StatusCode(int);
    impl StatusCode {
        pub fn short_message(&self) -> ~str {
            match **self {
              100 => ~"Continue",
              101 => ~"Switching Protocols",
              200 => ~"Ok",
              201 => ~"Created",
              202 => ~"Accepted",
              203 => ~"Non-Authoritative Information",
              204 => ~"No Content",
              205 => ~"Reset Content",
              206 => ~"Partial Content",
              300 => ~"Multiple Choices",
              301 => ~"Moved Permanently",
              302 => ~"Found",
              303 => ~"See Other",
              304 => ~"Not Modified",
              305 => ~"Use Proxy",
              307 => ~"Temporary Redirect",
              400 => ~"Bad Request",
              401 => ~"Not Authorized",
              402 => ~"Payment Required",
              403 => ~"Forbidden",
              404 => ~"Not Found",
              405 => ~"Method Not Allowed",
              406 => ~"Not Acceptable",
              407 => ~"Proxy Authentication Required",
              408 => ~"Request Timeout",
              409 => ~"Conflict",
              410 => ~"Gone",
              411 => ~"Length Required",
              412 => ~"Precondition Failed",
              413 => ~"Request Entity Too Large",
              414 => ~"Request-URI Too Long",
              415 => ~"Unsupported Media Type",
              416 => ~"Requested Range Not Satisfiable",
              417 => ~"Expectation Failed",
              500 => ~"Internal Server Error",
              501 => ~"Not Implemented",
              502 => ~"Bad Gateway",
              503 => ~"Service Unavailable",
              504 => ~"Gateway Timeout",
              505 => ~"HTTP Version Not Supported",
              _ => {
                fail!(fmt !
                      ( "No message avalibe for error code %?" , self ));
              }
            }
        }
        pub fn to_str(&self) -> ~str { (**self).to_str() }
        pub fn to_int(&self) -> int { (**self) }
    }
}
mod request {
    extern mod std;
    use std::net::{tcp, ip};
    use core::hashmap::linear::LinearMap;
    use method::Method;
    mod method {
        /// HTTP Methods
        pub enum Method {
            GET,
            HEAD,
            POST,
            PUT,
            DELETE,
            TRACE,
            OPTIONS,
            CONNECT,
            PATCH,
        }
        impl Method {
            pub fn from_str(method: &str) -> Option<Method> {
                match method {
                  "GET" => Some(GET),
                  "HEAD" => Some(HEAD),
                  _ => None
                }
            }
        }
    }
    /**
 * A HTTP request
 * 
 * * Method (method -> method::Method)
 * * Request-URI (request-uri -> String pointing to requested resource)
 * * HTTP-Version (http-version -> string version)
 * * Headers (headers -> Map KeyValue)
 * * Message-Body (message-body)
 *
 **/
    pub struct Request {
        host: ip::IpAddr,
        headers: LinearMap<~str, ~str>,
        method: Method,
        request_uri: ~str,
        message_body: ~str,
        close_connection: bool,
        http_version: (int, int),
        return_status: int,
    }
    priv enum ParseResult<T> { ParseFailure(ParseError), ParseSuccess(T), }
    priv struct ParseError {
        line: int,
        return_status: int,
    }
    pub struct HTTPHeader {
        method: Method,
        request_uri: ~str,
        close_connection: bool,
        http_version: (int, int),
        valid: bool,
        return_status: int,
    }
    impl Request {
        pub fn get(socket: &tcp::TcpSocket) -> Option<Request> {
            let request = socket.read(0u);
            if request.is_err() { return None }
            let request = str::from_bytes(request.get());
            match parseRequest(request, &socket.get_peer_addr()) {
              ParseSuccess(val) => Some(val),
              ParseFailure(_) => None
            }
        }
    }
    pub fn parseHeaders(request: &str) -> LinearMap<~str, ~str> {
        let mut headers = LinearMap::new();
        str::each_line_any(request, |line| {
                           match str::find_char(line, ':') {
                             Some(pos) => {
                               headers.insert(line.slice(0, pos).to_owned(),
                                              line.slice(pos + 2,
                                                         line.len()).to_owned())
                             }
                             None => {
                               if (line == ~"\r\n") | (line == ~"\n") |
                                      (line == ~"") {
                                   false
                               } else { true }
                             }
                           } });
        headers
    }
    pub fn parseHTTPHeader(HTTPHeaderStr: &str) -> ParseResult<HTTPHeader> {
        let mut words = ~[];
        for str::each_word(HTTPHeaderStr) |word| { words.push(word) }
        match words.len() {
          3 => {
            let http_version_string = words[2];
            if http_version_string.slice(0, 5) != "HTTP/" {
                return ParseFailure(ParseError{line: 0, return_status: 400,});
            }
            let base_version_number_string =
                http_version_string.slice(5, http_version_string.len());
            let mut version_number = ~[];
            for str::each_split_char(base_version_number_string, '.') |num| {
                version_number.push(num)
            }
            if version_number.len() != 2 {
                return ParseFailure(ParseError{line: 0, return_status: 400,})
            }
            let http_version =
                (int::from_str(version_number[0]).unwrap(),
                 int::from_str(version_number[1]).unwrap());
            if http_version < (1, 1) {
                return ParseFailure(ParseError{line: 0, return_status: 505,})
            }
            let m =
                match Method::from_str(words[0]) {
                  Some(val) => val,
                  None =>
                  return ParseFailure(ParseError{line: 0,
                                                 return_status: 405,})
                };
            ParseSuccess(HTTPHeader{method: m,
                                    request_uri: words[1].to_owned(),
                                    close_connection: false,
                                    http_version: http_version,
                                    valid: true,
                                    return_status: 200,})
          }
          _ => { ParseFailure(ParseError{line: 0, return_status: 400,}) }
        }
    }
    priv fn parseRequest(request: &str, ip: &ip::IpAddr) ->
     ParseResult<Request> {
        let mut lines = ~[];
        for str::each_line_any(request) |line| { lines.push(line) }
        let httpHeader =
            match parseHTTPHeader(lines.remove(0)) {
              ParseFailure(error) => return ParseFailure(error),
              ParseSuccess(header) => header
            };
        let headers = parseHeaders(request);
        lines.remove(headers.len());
        let close_connection =
            match headers.find(&~"Connection").unwrap().to_lower() {
              ~"close" => true,
              ~"keep-alive" => false,
              _ => false
            };
        ParseSuccess(Request{host: *ip,
                             method: httpHeader.method,
                             request_uri: httpHeader.request_uri.to_owned(),
                             return_status: httpHeader.return_status,
                             http_version: httpHeader.http_version,
                             headers: headers,
                             message_body: str::connect_slices(lines, "\r\n"),
                             close_connection: close_connection,})
    }
    #[test]
    fn vaild_header_qaulified_path() {
        match parseHTTPHeader("GET /test/test.html HTTP/1.1") {
          ParseFailure(_) => fail!(~ "Parse failed"),
          ParseSuccess(header) => {
            assert!(header . request_uri == ~ "/test/test.html");
            assert!(header . return_status == 200);
            assert!(header . http_version == ( 1 , 1 ));
            assert!(match header . method { GET => true });
          }
        }
    }
    #[test]
    fn vaild_header_path() {
        match parseHTTPHeader("GET /test/test HTTP/1.1") {
          ParseFailure(_) => fail!(~ "Parse failed"),
          ParseSuccess(header) => {
            assert!(header . request_uri == ~ "/test/test");
            assert!(header . return_status == 200);
            assert!(header . http_version == ( 1 , 1 ));
            assert!(match header . method { GET => true });
          }
        }
    }
    #[test]
    fn bad_http_version() {
        match parseHTTPHeader("GET /test/test HTTP/1.0") {
          ParseFailure(error) => assert!(error . return_status == 505),
          ParseSuccess(_) => fail!(~ "ignored")
        }
    }
    #[test]
    fn invalid_method() {
        match parseHTTPHeader("GARBAGE /test/test HTTP/1.1") {
          ParseFailure(error) => { assert!(error . return_status == 405) }
          ParseSuccess(_) => fail!(~ "ignored")
        }
    }
    #[test]
    fn headers_some() {
        let val =
            parseHeaders("GET /test/test HTTP/1.1 \ntest: param\nxss: iscool\nall: win\n\r\nthings: failed");
        io::println(fmt!("%?" , val));
        assert!(val . get ( & ~ "test" ) == & ~ "param");
        assert!(val . get ( & ~ "xss" ) == & ~ "iscool");
        assert!(val . get ( & ~ "all" ) == & ~ "win");
        assert!(! val . contains_key ( & ~ "things" ));
    }
}
mod response {
    extern mod std;
    use core::hashmap::linear::LinearMap;
    use statuscodes::StatusCode;
    mod statuscodes {
        pub struct StatusCode(int);
        impl StatusCode {
            pub fn short_message(&self) -> ~str {
                match **self {
                  100 => ~"Continue",
                  101 => ~"Switching Protocols",
                  200 => ~"Ok",
                  201 => ~"Created",
                  202 => ~"Accepted",
                  203 => ~"Non-Authoritative Information",
                  204 => ~"No Content",
                  205 => ~"Reset Content",
                  206 => ~"Partial Content",
                  300 => ~"Multiple Choices",
                  301 => ~"Moved Permanently",
                  302 => ~"Found",
                  303 => ~"See Other",
                  304 => ~"Not Modified",
                  305 => ~"Use Proxy",
                  307 => ~"Temporary Redirect",
                  400 => ~"Bad Request",
                  401 => ~"Not Authorized",
                  402 => ~"Payment Required",
                  403 => ~"Forbidden",
                  404 => ~"Not Found",
                  405 => ~"Method Not Allowed",
                  406 => ~"Not Acceptable",
                  407 => ~"Proxy Authentication Required",
                  408 => ~"Request Timeout",
                  409 => ~"Conflict",
                  410 => ~"Gone",
                  411 => ~"Length Required",
                  412 => ~"Precondition Failed",
                  413 => ~"Request Entity Too Large",
                  414 => ~"Request-URI Too Long",
                  415 => ~"Unsupported Media Type",
                  416 => ~"Requested Range Not Satisfiable",
                  417 => ~"Expectation Failed",
                  500 => ~"Internal Server Error",
                  501 => ~"Not Implemented",
                  502 => ~"Bad Gateway",
                  503 => ~"Service Unavailable",
                  504 => ~"Gateway Timeout",
                  505 => ~"HTTP Version Not Supported",
                  _ => {
                    fail!(fmt !
                          ( "No message avalibe for error code %?" , self ));
                  }
                }
            }
            pub fn to_str(&self) -> ~str { (**self).to_str() }
            pub fn to_int(&self) -> int { (**self) }
        }
    }
    pub struct Response {
        http_version: (int, int),
        status_code: StatusCode,
        headers: LinearMap<~str, ~str>,
        body: ~str,
    }
    impl Response {
        pub fn to_str(&self) -> ~str {
            let (version_major, version_minor) = self.http_version;
            let status_line =
                fmt!("HTTP/%d.%d %d %s\r\n" , version_major , version_minor ,
                     self . status_code . to_int ( ) , self . status_code .
                     short_message ( ));
            let mut header_strs = ~[];
            for self.headers.each_key |key| {
                header_strs.push(fmt!("%s: %s\r\n" , * key , * self . headers
                                      . get ( key )));
            }
            header_strs.push(fmt!("Content-Length: %u\r\n" , self . body . len
                                  ( )));
            str::concat(vec::concat([~[status_line], header_strs, ~[~"\r\n"],
                                     ~[self.body.to_owned()]]))
        }
        pub fn to_bytes(&self) -> ~[u8] { self.to_str().to_bytes() }
    }
    #[test]
    pub fn genral_test() {
        let mut headers = LinearMap::new();
        headers.insert(~"Awesome", ~"true");
        headers.insert(~"Content-Type", ~"text/html");
        headers.insert(~"cows-will", ~"win");
        let res =
            Response{http_version: (1, 1),
                     status_code: StatusCode(200),
                     headers: headers,
                     body: ~"derp",};
        io::println(fmt!("%?" , res . to_str ( )));
        let res_str = res.to_str();
        assert!(res_str . contains ( ~ "HTTP/1.1 200 Ok\r\n" ))
        assert!(res_str . contains ( "Content-Type: text/html\r\n" ))
        assert!(res_str . contains ( "Awesome: true\r\n" ))
        assert!(res_str . contains ( "cows-will: win\r\n" ))
        assert!(res_str . contains ( "Content-Length: 4\r\n" ))
        assert!(res_str . contains ( "\r\nderp" ))
    }
}
mod method {
    /// HTTP Methods
    pub enum Method {
        GET,
        HEAD,
        POST,
        PUT,
        DELETE,
        TRACE,
        OPTIONS,
        CONNECT,
        PATCH,
    }
    impl Method {
        pub fn from_str(method: &str) -> Option<Method> {
            match method {
              "GET" => Some(GET),
              "HEAD" => Some(HEAD),
              _ => None
            }
        }
    }
}
pub struct Server {
    port: int,
    bind: ip::IpAddr,
}
impl Server {
    pub fn run(&self, callback: extern "Rust" fn(&Request) -> Response) ->
     result::Result<(), tcp::TcpListenErrData> {
        tcp::listen(self.bind, self.port as uint, 1000,
                    &uv::global_loop::get(), |_| { }, |new_conn, kill_ch| {
                    do task::spawn_supervised || {
                        let socket =
                            match tcp::accept(new_conn) {
                              result::Err(err) => {
                                kill_ch.send(Some(err));
                                fail!();
                              }
                              result::Ok(socket) => @socket
                            };
                        loop  {
                            match Request::get(socket) {
                              Some(request) => {
                                let response = @callback(&request);
                                socket.write(response.to_bytes());
                                if request.close_connection == true {
                                    break ;
                                }
                                task::yield();
                              }
                              None => { task::yield(); break ; }
                            }
                        }
                    } })
    }
}
fn cb(request: &Request) -> Response {
    Response{http_version: (1, 1),
             headers: hashmap::LinearMap::new::<~str, ~str>(),
             status_code: StatusCode(200),
             body: ~"<h1>Hi!</h1>",}
}
fn main() {
    let port =
        match os::getenv("PORT") {
          Some(s) => match int::from_str(s) { Some(s) => s, None => fail!() },
          None => fail!()
        };
    io::println(fmt!("port: %?" , port));
    let server = Server{port: port, bind: ip::v4::parse_addr("127.0.0.1"),};
    server.run(cb);
}
