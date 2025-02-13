#![allow(unused)]
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};

#[derive(Debug)]
struct Client {
    stream: TcpStream,
    addr: SocketAddr,
}
impl Client {
    fn new(listener: &TcpListener) -> Self {
        let (stream, addr) = listener.accept().expect("client disconnected!!");

        Self { stream, addr }
    }

    fn handle_client(mut self) {
        let mut buffer = [0; 1024];
        loop {
            let bytes_read = self
                .stream
                .read(&mut buffer)
                .expect("oops couldn't read the data!!!");

            if bytes_read == 0 {
                break;
            }
            let request = String::from_utf8_lossy(&buffer);
            println!("{:?}: {}", self.addr, request);
        }
        let response = "\nThanks for your messages";
        self.stream
            .write(format!("{}", response).as_bytes())
            .expect("oops failed to write response");
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("failed to connect to the server");
    println!("server listening to the localhost");

    loop {
        let client = Client::new(&listener);
        std::thread::spawn(move || client.handle_client());
    }
}
