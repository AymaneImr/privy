#![allow(unused)]
use crate::{
    client::connection::Client,
    messages::from_bytes,
    messages::to_bytes,
    messages::Messages,
    messages::MessagesHistory,
    roles::{Permission, Roles},
    token::Token,
};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tokio::io::AsyncWriteExt;
use tokio::sync::broadcast::Receiver;
use tokio::sync::Mutex;
use tokio::task::JoinSet;

const SAFE_MODE: bool = true;
const USE_CACHE: bool = true;

#[derive(Debug, Default, Clone)]
pub struct ServerManager {
    pub server_token: Arc<Token>,
    pub clients: HashMap<Arc<Token>, Arc<Client>>,
    pub logs: MessagesHistory,
    pub cache: ServerCache,
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

#[derive(Debug, Default, Clone)]
pub struct ServerCache {
    clients: HashMap<Arc<Token>, Arc<Client>>,
    use_cache: bool,
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
                            println!(" {} Successfully joined the server.", Private(client.addr));
                            let mssg = format!(
                                "{} Successfully joined the server.\n",
                                Private(client.addr)
                            );

                            self.broadcast(clients, token.clone(), to_bytes(&mssg))
                                .await;
                        }
                    }
                    Messages::ClientDisconnected { ref token, .. } => {
                        if let Some(client) = self.clients.get(token) {
                            println!(" {} disconnected", Private(client.addr));

                            let mssg = format!(" {} disconnected\n", Private(client.addr));

                            self.remove_client(token.clone());
                            self.broadcast(clients, token.clone(), to_bytes(&mssg))
                                .await;
                        }
                    }
                    Messages::NewMessage {
                        ref token,
                        ref message,
                    } => {
                        if let Ok(msg) = from_bytes(message) {
                            let client = self.clients.get(token).unwrap();
                            //TODO: notify the client that he doesn't have the permission
                            //      to send messages
                            if !client.role.permissions().send {
                                eprintln!("this client can't send messages");
                                continue;
                            }

                            //Limit messages sent rate
                            let mut last_sent = client.last_sent.lock().await;
                            let now = Instant::now();
                            if now.duration_since(*last_sent) < Duration::from_secs(2) {
                                eprintln!("Client is sending messages too fast");
                                continue;
                            }
                            *last_sent = now;

                            let mssg =
                                format!("{:?}: {}\n", Private(client.addr), Private(msg.clone()));
                            println!("{}", mssg);

                            self.logs.append(token.clone(), to_bytes(&mssg));

                            self.broadcast(clients, token.clone(), to_bytes(&mssg))
                                .await;
                        }
                    }
                }
            }
        }
    }

    pub async fn broadcast(
        &self,
        clients: HashMap<Arc<Token>, Arc<Client>>,
        client_token: Arc<Token>,
        message: Vec<u8>,
    ) {
        let mut set = JoinSet::new();

        for (token, client) in clients.into_iter() {
            let perm = client.role.permissions();

            if !Arc::ptr_eq(&token, &client_token) {
                let message = message.clone();
                set.spawn(async move {
                    let mut writer = client.writer.lock().await;
                    writer.write_all(&message).await
                });
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
