#![allow(unused)]

use crate::messages::from_bytes;
use crate::{messages::to_bytes, messages::Messages, token::Token};
use std::net::SocketAddr;
use std::sync::Arc;
use std::thread::sleep;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::Mutex;

//TODO: Add username field
#[derive(Debug, Clone)]
pub struct Client {
    pub reader: Arc<Mutex<ReadHalf<TcpStream>>>,
    pub writer: Arc<Mutex<WriteHalf<TcpStream>>>,
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

        let (reader, writer) = tokio::io::split(stream);

        Self {
            reader: Arc::new(Mutex::new(reader)),
            writer: Arc::new(Mutex::new(writer)),
            addr,
            tx,
        }
    }

    // Handle client's connection and messages broadcasting
    pub async fn handle_client(self: Arc<Self>, client_token: Arc<Token>) {
        let mut reader = self.reader.lock().await;
        let mut reader = BufReader::new(&mut *reader);
        let mut line: String = String::new();

        loop {
            let bytes_read = reader.read_line(&mut line);

            let bytes_read = bytes_read.await.unwrap_or(0);
            if bytes_read == 0 {
                let _ = self
                    .tx
                    .send(Messages::ClientDisconnected {
                        client: self.clone(),
                        token: client_token.clone(),
                    })
                    .map_err(|err| eprintln!("Couldn't broadcast the message!! {}", err));
                break;
            };

            let msg = format!("{}: {}", self.addr, line);

            if let Err(err) = self.tx.send(Messages::NewMessage {
                token: client_token.clone(),
                message: to_bytes(&line),
            }) {
                eprintln!("Couldn't broadcast the message!! {}", err);
            }

            line.clear();
        }
    }
}
