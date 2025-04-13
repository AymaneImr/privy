use crate::{client::connection::Client, messages::from_bytes, messages::Messages, token::Token};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use tokio::sync::broadcast::Receiver;
use tokio::task::JoinSet;

const SAFE_MODE: bool = false;

#[derive(Debug, Default)]
pub struct ServerManager {
    clients: HashMap<Arc<Token>, Arc<Client>>,
}

#[derive(Debug)]
struct Private<T>(T);

impl<T: fmt::Display> fmt::Display for Private<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(inner) = self;
        if SAFE_MODE {
            write!(f, "[HIDDEN]")
        } else {
            write!(f, "{}", inner)
        }
    }
}

impl ServerManager {
    fn add_client(&mut self, client: Arc<Client>, token: Arc<Token>) -> Option<Arc<Client>> {
        self.clients.insert(token, client)
    }

    fn remove_client(&mut self, token: Arc<Token>) {
        self.clients.remove(&token);
    }

    pub async fn handle_messages(mut self, mut receiver: Receiver<Messages>) {
        loop {
            if let Ok(recv_message) = receiver.recv().await {
                let clients = self.clients.clone();

                match recv_message {
                    Messages::ClientConnected {
                        ref client,
                        ref token,
                    } => {
                        //TODO: properly handle in case `Some()` was returned
                        if self.add_client(client.clone(), token.clone()).is_none() {
                            eprintln!("{} Successfully joined the server.", client.addr);

                            self.broadcast(clients, token.clone(), recv_message).await;
                        }
                    }
                    Messages::ClientDisconnected(ref token) => {
                        if let Some(client) = self.clients.get(token) {
                            eprintln!("{} disconnected", client.addr);

                            self.remove_client(token.clone());
                            self.broadcast(clients, token.clone(), recv_message).await;
                        }
                    }
                    Messages::NewMessage {
                        ref token,
                        ref message,
                    } => {
                        if let Ok(msg) = from_bytes(message) {
                            let client_addr = self.clients.get(token);
                            println!("{:?}: {}", client_addr, Private(msg));

                            self.broadcast(clients, token.clone(), recv_message).await
                        }
                    }
                }
            }
        }
    }

    async fn broadcast(
        &self,
        clients: HashMap<Arc<Token>, Arc<Client>>,
        client_token: Arc<Token>,
        message: Messages,
    ) {
        let mut set = JoinSet::new();

        for (token, client) in clients {
            let message = message.clone();
            if token != client_token {
                set.spawn(async move { client.tx.send(message) });
            }
        }

        while let Some(task) = set.join_next().await {
            match task {
                Ok(send_result) => {
                    if let Err(e) = send_result {
                        eprintln!("Send failed: {}", e);
                    }
                }
                Err(task_err) => {
                    eprintln!("Task failed: {}", task_err);
                }
            }
        }
    }
}
