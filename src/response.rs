extern mod std;


use core::hashmap::linear::LinearMap;

use statuscodes::StatusCode;

mod statuscodes;

pub struct Response{
    http_version: (int, int),
    status_code : StatusCode,
    headers: LinearMap<~str,~str>,
    body: ~str
}

impl Response{
    pub fn to_str(&self) -> ~str{
        let (version_major, version_minor) = self.http_version;
        let status_line = fmt!("HTTP/%d.%d %d %s\r\n",
            version_major,
            version_minor,
            self.status_code.to_int(),
            self.status_code.short_message());
        let mut header_strs = ~[];
        for self.headers.each_key()|key| {
           header_strs.push(fmt!("%s: %s\r\n", *key,*self.headers.get(key)));
        }
        match self.headers.find(&~"Content-Type") {
            None => {header_strs.push(fmt!("Content-Type: text/html"))}
            _ => {}
        }
        header_strs.push(fmt!("Content-Length: %u\r\n",self.body.len()));
        str::concat(vec::concat([~[status_line],header_strs,~[~"\r\n"],~[self.body.to_owned()]]))
    }

    pub fn to_bytes(&self) -> ~[u8]{
        self.to_str().to_bytes()
    }
}


#[test]
pub fn genral_test(){
    let mut headers = LinearMap::new();
    headers.insert(~"Awesome", ~"true");
    headers.insert(~"Content-Type", ~"text/html");
    headers.insert(~"cows-will", ~"win");
    let res = Response{http_version:(1,1), status_code: StatusCode(200), headers:headers, body:~"derp"};
    io::println(fmt!("%?",res.to_str()));
    let res_str = res.to_str();
    assert!(res_str.contains(~"HTTP/1.1 200 Ok\r\n"))
    assert!(res_str.contains("Content-Type: text/html\r\n"))
    assert!(res_str.contains("Awesome: true\r\n"))
    assert!(res_str.contains("cows-will: win\r\n"))
    assert!(res_str.contains("Content-Length: 4\r\n"))
    assert!(res_str.contains("\r\nderp")) 
                     
}