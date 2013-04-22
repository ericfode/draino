extern mod std;


use std::net::{tcp,ip};
use core::hashmap::linear::LinearMap;
use method::Method;
mod method;

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
    headers: LinearMap<~str,~str>,
    method: Method,
    request_uri: ~str, 
    message_body: ~str,
    close_connection: bool,
    http_version: (int, int),
    return_status: int 
 }

priv enum ParseResult<T>{
    ParseFailure(ParseError),
    ParseSuccess(T)
}

priv struct ParseError{
    line: int,
    return_status: int
}

//Should probably be an emun that has this and a failure structure as options
pub struct HTTPHeader{
    method: Method,
    request_uri: ~str,
    close_connection: bool,
    http_version: (int,int),
    valid: bool,
    return_status: int
}

impl Request {
 
    pub fn get(socket: &tcp::TcpSocket) -> Option<Request>{
        let request = socket.read(0u);
        if request.is_err(){
            return None
        }
        let request = str::from_bytes(request.get());
        io::println(request);
        match parseRequest(request, &socket.get_peer_addr()) {
            ParseSuccess(val) => Some(val),
            ParseFailure(_) => None
        }
    }

}

// HEADER : HEADERNAME ':' SP HEADERVALUE 
pub fn parseHeaders(request: &str) -> LinearMap<~str,~str>{  
    if request == "\r\n"{
        return LinearMap::new(); 
    }  
    let mut headers = LinearMap::new();
    str::each_line_any(request, |line | {
        match str::find_char(line, ':'){
            Some(pos) => {
                headers.insert(line.slice(0,pos).to_owned(), line.slice(pos+2, line.len()).to_owned() )
            },
            None      => {
                if (line == ~"\r\n") | (line == ~"\n") | (line == ~"") {
                    false
                } else {
                    true
                }         
            }
        }
    });
    headers
}

//REQUEST: METHOD SP REQUEST_URI SP 'HTTP/' VERSION 
pub fn parseHTTPHeader(HTTPHeaderStr:&str) -> ParseResult<HTTPHeader>{
    let mut words = ~[];
    for str::each_word(HTTPHeaderStr) |word| {words.push(word)}
    match words.len() {
        //Version Number
        3 => {
            //get http version string
            let http_version_string = words[2];

            if http_version_string.slice(0,5) != "HTTP/" {
                return ParseFailure(ParseError{line:0,return_status:400});
            }
            
            let base_version_number_string = http_version_string.slice(5,http_version_string.len());
            
            let mut version_number = ~[];
            for str::each_split_char(base_version_number_string, '.') |num| {version_number.push(num)}
            
            if version_number.len() != 2 {
                return ParseFailure(ParseError{line:0,return_status:400})
            }

            let http_version = (
                int::from_str(version_number[0]).unwrap(),
                int::from_str(version_number[1]).unwrap());

            if http_version < (1,1){
                return ParseFailure(ParseError{line:0,return_status:505})
            }

            let m = match Method::from_str(words[0]) {
                Some(val) => val,
                None      => return ParseFailure(ParseError{line:0, return_status:405})
            };

            ParseSuccess(HTTPHeader {
                            method: m,
                            request_uri: words[1].to_owned(), 
                            close_connection: false, 
                            http_version: http_version, 
                            valid: true,
                            return_status: 200})
        },   
        _ => {
            ParseFailure(ParseError{line:0,return_status:400})
        }
    }
}

priv fn parseRequest(request: &str,ip: &ip::IpAddr) -> ParseResult<Request>{
    let mut lines = ~[];
    for str::each_line_any(request)|line|{lines.push(line)}

    let httpHeader = match parseHTTPHeader(lines.remove(0)) {
        ParseFailure(error)   => return ParseFailure(error),
        ParseSuccess(header)  => header 
    };
    if (lines.len() > 1){
      let headers = parseheaders(request);
      lines.remove(headers.len());
    }
    //todo: this should probably have some default configuration
    let close_connection = match headers.find(&~"connection").unwrap().to_lower(){
        ~"close" => true,
        ~"keep-alive" => false,
        _ => false
    };

   parsesuccess(request{
        host: *ip,
        method: httpheader.method,
        request_uri: httpheader.request_uri.to_owned(), 
        return_status: httpheader.return_status,
        http_version: httpheader.http_version,
        headers: headers,
        message_body: str::connect_slices(lines,"\r\n"),
        close_connection: close_connection,
       })
} 

#[test]
fn vaild_header_qaulified_path()
{
    match parsehttpheader("get /test/test.html http/1.1"){
        parsefailure(_) => fail!(~"parse failed"),
        parsesuccess(header) => {
            assert!(header.request_uri == ~"/test/test.html");
            assert!(header.return_status == 200);
            assert!(header.http_version == (1,1));
            assert!(match header.method {
                get => true
            });
        }
    }
}


#[test]
fn vaild_header_path()
{
    match parsehttpheader("get /test/test http/1.1"){
        parsefailure(_) => fail!(~"parse failed"),
        parsesuccess(header) => {
            assert!(header.request_uri == ~"/test/test");
            assert!(header.return_status == 200);
            assert!(header.http_version == (1,1));
            assert!(match header.method {
                get => true
            });
        }
    }
}

#[test]
fn bad_http_version()
{
    match parsehttpheader("get /test/test http/1.0"){
        parsefailure(error) => assert!(error.return_status == 505),
        parsesuccess(_) => fail!(~"ignored")
    }
}

#[test]
fn invalid_method()
{
    match parsehttpheader("garbage /test/test http/1.1"){
        parsefailure(error) => {assert!(error.return_status == 405)},
        parsesuccess(_) => fail!(~"ignored")
    }   
}

#[test]
fn headers_some()
{
    let val =  parseHeaders("GET /test/test HTTP/1.1 \n\
                           test: param\n\
                           xss: iscool\n\
                           all: win\n\
                           \r\n\
                           things: failed");
    io::println(fmt!("%?",val));
    assert!(val.get(&~"test") == &~"param");
    assert!(val.get(&~"xss") == &~"iscool");
    assert!(val.get(&~"all") == &~"win");
    assert!(!val.contains_key(&~"things"));
}

#[test]
fn headers_none()
{
  let val = parseHeaders("\r\n");
}
