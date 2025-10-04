#![allow(unused)]

use privy::{
    messages::{from_bytes, to_bytes, Messages},
    token::{get_token, Token},
};
use std::io::{self, BufRead};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    // Connect to the server
    let stream = match TcpStream::connect("0.0.0.0:1337").await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to connect to server: {}", e);
            return;
        }
    };
    println!("Connected to server");

    let (reader, mut writer) = stream.into_split();

    // Generate a unique token for this client
    let token = Arc::new(get_token());

    // Task to read messages from the server
    tokio::spawn(async move {
        let mut lines = BufReader::new(reader).lines();

        while let Ok(Some(line)) = lines.next_line().await {
            println!("Server: {}", line);
        }
        println!("Server connection closed");
    });

    // Read user input synchronously and send to server asynchronously
    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    let mut input = String::new();

    loop {
        input.clear();
        match stdin_lock.read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let message = input.trim_end().to_string();

                if message.is_empty() {
                    continue;
                }

                let msg_bytes = to_bytes(&message);

                if let Err(e) = writer.write_all(&msg_bytes).await {
                    eprintln!("Failed to send message: {}", e);
                    break;
                }
                if let Err(e) = writer.write_all(b"\n").await {
                    eprintln!("Failed to send newline: {}", e);
                    break;
                }
            }
            Err(e) => {
                eprintln!("Failed to read from stdin: {}", e);
                break;
            }
        }
    }
}
