#![allow(unused)]
use crate::server;
use crate::{
    client::connection::Client, messages::Messages, roles::Roles, server::manager::ServerManager,
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

    //Add a check token functionality
    let mut manager = server::manager::ServerManager {
        server_token: Arc::new(get_token()),
        ..Default::default()
    };
    let mut admin_exists = false;

    // Create a broadcast channel to send and receive multiple messages
    let (tx, rx) = broadcast::channel(16);

    tokio::spawn(manager.clone().handle_messages(rx));

    let listener = Arc::new(listener);

    // Loop infinitely to create and accept new connections.
    loop {
        let tx = tx.clone();

        // Give Admin role to the first connection
        let role = if !admin_exists {
            admin_exists = true; // mark that admin now exists
            Roles::Admin
        } else {
            Roles::User
        };

        let client = Arc::new(Client::new(Arc::clone(&listener), tx.clone(), role.clone()).await);

        let token = Arc::new(get_token());

        // Notify the manager  (through the broadcast channel)
        let _ = tx.send(Messages::ClientConnected {
            client: Arc::clone(&client),
            token: Arc::clone(&token),
        });

        // Spawn a task to handle each client simultaneously
        tokio::spawn(client.handle_client(token));
    }
}
