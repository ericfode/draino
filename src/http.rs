extern mod std;
use std::net::{ip,tcp};
use std::{uv,task};
use hashmap = core::hashmap::linear;

use StatusCode = statuscodes::StatusCode;
use request::Request;
use response::Response;

mod statuscodes;
mod request;
mod response;
mod method;


pub struct Server{
    port: int,
    bind: ip::IpAddr
}

impl Server {
    pub fn run(&self,callback:extern fn(&Request) -> Response) -> result::Result<(), tcp::TcpListenErrData>{
        tcp::listen(self.bind, self.port as uint, 1000, &uv::global_loop::get(), |_| {},
            |new_conn, kill_ch|{
                do task::spawn_supervised{
                    let socket = match tcp::accept(new_conn) {
                        result::Err(err) => {
                            kill_ch.send(Some(err));
                            fail!();
                        },
                        result::Ok(socket) => socket
                    };
                    
                      let ip = socket.get_peer_addr(); 
                      let buf = tcp::socket_buf(socket); 
                      loop {
                        io::println("parsing request");
                        match Request::create_request(&buf,&ip){
                            Some(request) => {
                                io::println("responding");
                                let response = callback(&request);
                                 
                                buf.write(response.to_bytes());
                                buf.flush();
                                if request.close_connection == true{
                                    break;
                                }
                                task::yield();
                            },
                            None => {io::println("error or boring keepalive"); break;}
                        }
                    }
                }
            }
        )
    }
}

fn cb(request: &Request) -> Response{
    Response{
        http_version: (1,1),
        headers: hashmap::LinearMap::new::<~str,~str>(),
        status_code: StatusCode(200),
        body: ~"<h1>Hi!</h1>"
    }
}

fn main(){
    let port = match os::getenv("PORT"){
        Some(s) => match int::from_str(s) { Some(s) => s, None => fail!() },
        None => fail!(~"no ports man")
    };
    io::println(fmt!("port: %?", port));
    let server = Server{port: port, bind: ip::v4::parse_addr("0.0.0.0")};
    server.run(cb);
}
