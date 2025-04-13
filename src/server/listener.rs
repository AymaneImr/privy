//#![allow(unused)]
use crate::{
    client::connection::Client, messages::Messages, server::manager::ServerManager,
    token::get_token,
};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::broadcast;

const PORT: u16 = 1337;

pub async fn start_server(addr: String) {
    println!("Running Server...");

    let addr = format!("{}:{}", addr, PORT);
    let listener = TcpListener::bind(&addr)
        .await
        .expect("failed to connect to the server");
    println!("Server running at {addr}");

    let manager = ServerManager::default();

    // Create a broadcast channel to send and receive multiple messages
    let (tx, rx) = broadcast::channel(16);

    tokio::spawn(manager.handle_messages(rx));

    let listener = Arc::new(listener);

    // Loop infinitely to create and accept new connections,
    loop {
        let tx = tx.clone();
        let rx = tx.subscribe();
        let client = Arc::new(Client::new(Arc::clone(&listener), tx.clone()).await);

        let token = Arc::new(get_token());

        let _ = client
            .tx
            .send(Messages::ClientConnected {
                client: Arc::clone(&client),
                token: Arc::clone(&token),
            })
            .map_err(|err| eprintln!("Couldn't broadcast the message!! {}", err));

        // Spawn a task to handle each client simultaneously
        tokio::spawn(client.handle_client(rx, token));
    }
}
