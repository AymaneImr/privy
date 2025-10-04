use crate::{client::connection::Client, token::Token};
use std::sync::Arc;

//TODO: Add username variant
#[derive(Debug, Clone)]
pub enum Messages {
    ClientConnected {
        client: Arc<Client>,
        token: Arc<Token>,
    },
    ClientDisconnected {
        client: Arc<Client>,
        token: Arc<Token>,
    },
    NewMessage {
        token: Arc<Token>,
        message: Vec<u8>,
    },
}

pub fn from_bytes(b_message: &Vec<u8>) -> Result<String, ()> {
    String::from_utf8(b_message.to_owned())
        .map_err(|err| eprintln!("couldn't convert message to String: {}", err))
}

pub fn to_bytes(message: &String) -> Vec<u8> {
    message.as_bytes().to_vec()
}
