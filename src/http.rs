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
                        result::Ok(socket) => @socket
                    };
                    loop {
                        match Request::get(socket){
                            Some(request) => {
                                let response = @callback(&request);
                                
                                socket.write(response.to_bytes());
                                if request.close_connection == true{
                                    break;
                                }
                                task::yield();
                            },
                            None => {task::yield();break;}
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
        None => fail!()
    };
    io::println(fmt!("port: %?", port));
    let server = Server{port: port, bind: ip::v4::parse_addr("127.0.0.1")};
    server.run(cb);
}
