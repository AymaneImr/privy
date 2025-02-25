#![allow(unused)]
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio::sync::Mutex;

#[derive(Debug)]
struct Client {
    stream: TcpStream,
    addr: SocketAddr,
    tx: Sender<String>,
    rx: Receiver<String>,
}
impl Client {
    // Await a new client connection and then returns the client's instance
    // with a TCP stream, address, sender, and receiver.
    async fn new(listener: &TcpListener, tx: Sender<String>, rx: Receiver<String>) -> Self {
        let (stream, addr) = listener
            .accept()
            .await
            .expect("Failed to accept connection!!");

        Self {
            stream,
            addr,
            tx,
            rx,
        }
    }

    // Handle client's connection and messages broadcasting
    async fn handle_client(mut self) {
        let (reader, mut writer) = self.stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line: String = String::new();

        // Bind the writer inside `Mutex` to ensure exclusive access,
        // stored in `Arc` to easily allow shared ownership
        let writer = Arc::new(Mutex::new(writer));

        loop {
            tokio::select! {

                //
                bytes_read = reader.read_line(&mut line) => {

                let bytes_read = bytes_read.unwrap_or(0);
                if bytes_read == 0 {
                    println!("Client disconnected!");
                    break;
                }

                let msg = format!("{:?}: {}", self.addr, line);
                println!("{}", msg);

                if let Err(err) = self.tx.send(msg){
                    eprintln!("Couldn't broadcast the message!! {}", err);
                }
                line.clear()
                },

                msg = self.rx.recv() => {
                    if let Ok(msg) = msg{
                        let mut writer = writer.lock().await;

                        if let Err(err) = writer.write_all(msg.as_bytes()).await{
                            eprintln!("Couldn't send message to {}: {}", self.addr, err);
                        }
                    }

                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("failed to connect to the server");
    println!("server listening to the localhost");

    // Create a broadcast channel to send and receive multiple messages
    let (tx, rx) = broadcast::channel(16);

    // Loop infinitely to create and accept new connections,
    loop {
        let tx = tx.clone();
        let rx = tx.subscribe();
        let client = Client::new(&listener, tx, rx).await;
        // Spawn a task to handle each client simultaneously
        tokio::spawn(client.handle_client());
    }
}
