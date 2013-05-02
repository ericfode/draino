extern mod std;

use std::int;
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

pub enum ParseResult<T>{
    ParseFailure(ParseError),
    ParseEmpty,
    ParseSuccess(T)
}
pub struct ParseError{
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

pub struct Request {
    host: ip::IpAddr,
    headers: LinearMap<~str,~str>,
    method: Method,
    request_uri: ~str, 
    message_body: Option<~[u8]>,
    body_length: uint,
    close_connection: bool,
    http_version: (int, int),
    return_status: int 
}


impl Request {

  pub fn create_request<T : io::ReaderUtil>(reader: &T, addr: &ip::IpAddr) -> ParseResult<Request>{
  let headerStr = getHeaderStr(reader);
  debug!(fmt!("%?",headerStr));
  if (headerStr == ~"") {
    debug!("header seems to be empty");
    return ParseEmpty;
  }
  let mut request = match parseRequest(headerStr, addr) {
      ParseSuccess(val) => val,
      ParseFailure(error) =>  return ParseFailure(error),
      ParseEmpty => return ParseEmpty
  };

  request.message_body = Some(reader.read_bytes(request.body_length));
  return ParseSuccess(request)
  }
}

priv fn getHeaderStr<T : io::ReaderUtil>(reader: &T) -> ~str{
  let mut req: ~str = ~"";  
  loop {
    req += reader.read_until('\n',true);
    if str::contains(req, "\r\n\r\n") || req == ~""{
      return req;
    }
  }
}



// HEADER : HEADERNAME ':' SP HEADERVALUE 
pub fn parseHeaders(requestLines: &[&str]) -> LinearMap<~str,~str>{  
  if( requestLines.len() == 0){
    debug!("no headers found at:parseHeaders");
    return LinearMap::new();
  }  
  let mut headers = LinearMap::new();
    requestLines.each( |line| {
        debug!(fmt!("str: %?", *line));
        match str::find_char(*line, ':'){
            Some(pos) => {
                headers.insert(line.slice(0,pos).to_owned(), line.slice(pos+2, line.len()).to_owned() )
            },
            None      => {
                if (*line == ~"\r\n") | (*line == ~"\n") | (*line == ~"") {
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
              debug!("header version string error");
              return ParseFailure(ParseError{line:0,return_status:400});
            }
            
            let base_version_number_string = http_version_string.slice(5,http_version_string.len());
            
            let mut version_number = ~[];
            for str::each_split_char(base_version_number_string, '.') |num| {version_number.push(num)}
            
            if version_number.len() != 2 {
                debug!("verstion error");
                return ParseFailure(ParseError{line:0,return_status:400})
            }

            let http_version = (
                int::from_str(version_number[0]).unwrap(),
                int::from_str(version_number[1]).unwrap());

            if http_version < (1,0){
                return ParseFailure(ParseError{line:0,return_status:505})
            }

            let m = match Method::from_str(words[0]) {
                Some(val) => val,
                None      => return ParseFailure(ParseError{line:0, return_status:405})
            };

            ParseSuccess(HTTPHeader {
                            method: m,
                            request_uri: words[1].to_owned(), 
                            close_connection: true, 
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
        ParseSuccess(header)  => header,
        ParseEmpty            => return ParseEmpty
    };
    debug!(fmt!("lines: %?", lines)); 
    let headers = parseHeaders(lines);
    let mut close_connection =false;
    let mut body_length = 0;
    if (headers.len() > 0) {
      lines.remove(headers.len() - 1);
      close_connection = match headers.find(&~"Connection") {
        Some(val)  => {  match val.to_lower(){
          ~"close"      => true,
          ~"keep-alive" => false,
          _             => false 
        }},
        None       => false 
      };

      body_length = match headers.find(&~"Content-Length"){
        Some(val) =>  match uint::from_str(*val) {
              Some(val) => val,
              None()    => 0
            },
        None()    => 0
      };
    } 
    //TODO: NEED to get message body size.
   ParseSuccess(Request{
        host: *ip,
        method: httpHeader.method,
        request_uri: httpHeader.request_uri.to_owned(), 
        return_status: httpHeader.return_status,
        http_version: httpHeader.http_version,
        headers: headers,
        body_length: body_length,
        message_body: None,
        close_connection: close_connection,
       })
} 

#[test]
fn vaild_header_qaulified_path()
{
    match parseHTTPHeader("GET /test/test.html HTTP/1.1"){
        ParseFailure(_) => fail!(~"Parse failed"),
        ParseSuccess(header) => {
            assert!(header.request_uri == ~"/test/test.html");
            assert!(header.return_status == 200);
            assert!(header.http_version == (1,1));
            assert!(match header.method {
                GET => true
            });
        }
    }
}


#[test]
fn vaild_header_path()
{
    match parseHTTPHeader("GET /test/test HTTP/1.1"){
        ParseFailure(_) => fail!(~"Parse failed"),
        ParseSuccess(header) => {
            assert!(header.request_uri == ~"/test/test");
            assert!(header.return_status == 200);
            assert!(header.http_version == (1,1));
            assert!(match header.method {
                GET => true
            });
        }
    }
}

#[test]
fn bad_http_version()
{
    match parseHTTPHeader("GET /test/test HTTP/1.0"){
        ParseFailure(error) => assert!(error.return_status == 505),
        ParseSuccess(_) => fail!(~"ignored")
    }
}

#[test]
fn invalid_method()
{
    match parseHTTPHeader("GARBAGE /test/test HTTP/1.1"){
        ParseFailure(error) => {assert!(error.return_status == 405)},
        ParseSuccess(_) => fail!(~"ignored")
    }   
}

#[test]
fn headers_some()
{
    let myStr ="test: param\n\
                       xss: iscool\n\
                       all: win\n\
                       \r\n\
                       things: failed";
    let mut words = ~[];
    for str::each_line_any(myStr) |word| {words.push(word)}
 
    let val =  parseHeaders(words);
    io::println(fmt!("%?",val));
    assert!(val.get(&~"test") == &~"param");
    assert!(val.get(&~"xss") == &~"iscool");
    assert!(val.get(&~"all") == &~"win");
    assert!(!val.contains_key(&~"things"));
}

#[test]
fn headers_none()
{
  let mut words = ~[];
  let val = parseHeaders(words);
}

