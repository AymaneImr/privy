use getrandom;
use std::fmt::Write;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct Token(pub String);

pub fn get_token() -> Token {
    let mut buffer = [0u8; 16];
    let _ =
        getrandom::fill(&mut buffer).map_err(|err| eprintln!("couldn't generate token, {}", err));

    let mut token = String::new();
    buffer.iter().for_each(|b| {
        let _ = write!(&mut token, "{b:02X}");
    });

    Token(token)
}
