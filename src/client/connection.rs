#![allow(unused)]

use crate::{messages::to_bytes, messages::Messages, token::Token};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct Client {
    stream: Arc<Mutex<TcpStream>>,
    pub addr: SocketAddr,
    pub tx: Sender<Messages>,
}

impl Client {
    // Await a new client connection and then returns the client's instance
    // with a TCP stream, address, sender, and receiver.
    pub async fn new(listener: Arc<TcpListener>, tx: Sender<Messages>) -> Self {
        let (stream, addr) = listener
            .accept()
            .await
            .expect("Failed to accept connection!!");

        let stream = Arc::new(Mutex::new(stream));

        Self { stream, addr, tx }
    }

    // Handle client's connection and messages broadcasting
    pub async fn handle_client(self: Arc<Self>, mut rx: Receiver<Messages>, token: Arc<Token>) {
        let mut stream = self.stream.lock().await;

        let (reader, writer) = stream.split();
        let mut reader = BufReader::new(reader);
        let mut line: String = String::new();

        // Bind the writer inside `Mutex` to ensure exclusive access,
        // stored in `Arc` to easily allow shared ownership
        let writer = Arc::new(Mutex::new(writer));

        loop {
            tokio::select! {

                bytes_read = reader.read_line(&mut line) => {

                let bytes_read = bytes_read.unwrap_or(0);
                if bytes_read == 0 {

                   let _ = self.tx.send(Messages::ClientDisconnected(token.clone()))
                       .map_err(|err| eprintln!("Couldn't broadcast the message!! {}", err) );
                   break;
                };

                let msg = format!("{}: {}", self.addr, line);
                println!("{}", msg);

                if let Err(err) = self.tx.send(Messages::NewMessage { token: token.clone(), message: to_bytes(&line) }){
                    eprintln!("Couldn't broadcast the message!! {}", err);
                }
                line.clear()
                },

                msg = rx.recv() => {
                    if let Ok(msg) = msg{
                        let mut writer = writer.lock().await;

                        /*
                        if let Err(err) = writer.write_all(msg.as_bytes()).await{
                            eprintln!("Couldn't send message to {}: {}", Sens(self.addr), err);
                        }*/
                    }
                }
            }
        }
    }
}
