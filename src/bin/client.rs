#![allow(unused)]

use privy::{client::connection::Client, messages::Messages, token::Token};
use std::net::TcpStream;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::broadcast::{self, Receiver};

#[tokio::main]
async fn main() {
    if let Ok(stream) = TcpStream::connect("127.0.0.1:8080") {
        println!("just connected")
    }
}
