use std::net::{tcp,ip};
use hashmap = core::hashmap::linear;
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
    host: ip:IpAddr,
    headers: hashmap::LinearMap<~str,~str>,
    method: Method,
    request_uri: ~str, 
    message_body: ~str,
    close_connection: bool,
    http_version: (int, int),
    vaild: bool
    return_status: int 
 }

 impl Request {

    priv fn parseRequest(request: ~str,ip: &IpAddr){

        let mut lines = ~[];
        for str::each_line_any(request)|line|{lines.push(line)}
        debug!("%?",lines)
        let mut words = ~[];
        for str::each_word(lines.remove(0)) |word| {words.push(word)}

        //REQUEST: METHOD SP REQUEST_URI SP 'HTTP/' VERSION 
        let (command, request_uri, close_connection,http_version,vaild,return_status) = match words.len() {
            //Version Number
            3 => {
                //get http version string
                let http_version_string = words[2]
                let valid = true
                let return_status = 200

                //slice ident
                if http_version_string.slice(0,5) != "HTTP/" {
                    valid = false    
                    return_status = 400
                    debug!("%?",http_version)
                    ("","",false,(1,1),false,400)
                }
                
                let base_version_number_string = http_version_string.slice(5,http_version.len());
                let mut version_number = ~[]
                for str::each_split_char(base_version_number_string, '.') |num|{version_number.push(num)}
                if version_number.len() != 2 {
                    vaild = false
                    return_status = 400
                    ("","",false,(1,1),false,400)
                }

                let http_version = (
                    int::from_str(version_number[0]).unwrap(),
                    int::from_str(version_number[0]).unwrap());

                let close_connection = if version_number >= (1,1) {false} else {true};
                (words[0], words[1], close_connection, http_version, valid, return_status)
            },   
            _ => {
                debug!("Bad HTTP request %?", words))
                ("","",false,(1,1),false,400)
            }
        }

        let mut headers = hashmap::LinearMap::new::<~str,~str>();

        loop {
            let line = lines.remove(0);
            //look for terminateing line
            if (line == ~"\r\n") | (line == ~"\n") | (line == ~"") {break;}
            // HEADER : HEADERNAME ':' SP HEADERVALUE 
            match str::find_char(line, ':'){
                Some(pos) => {headers.insert(line.slice(0,pos).to_owned), line.slice(pos+2, line.len()).to_owned());},
                None      => {break;}
            }
        }

        let close_connection = match headers.find(&~"Connection").unwrap().to_lower(){
            ~"close" => true,
            ~"keep-alive" => false,
            _ => close_connection
        }

        Request{
            host: ip
            headers: headers
            method: method,
            request_uri: request_uri.to_owned(), 
            message_body: str::connect_slices(lines,"\r\n"),
            close_connection: close_connection,
            http_version: http_version,
            vaild: vaild
            return_status: return_status}
    } 

    pub fn get(socket: &tcp::TcpSocket) -> Request{
        let request = socket.read(0u);
        if request.is_err(){
            fail!(~"Bad connection!");
        }
        let request = str::from_bytes(request.get());
        parseRequest(request, socket.get_peer_addr())
    }

 }
