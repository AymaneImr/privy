//#![allow(unused)]
use clap::{arg, command, Parser};
use privy::server::listener::start_server;

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[arg(default_value = "0.0.0.0")]
    address: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    start_server(args.address).await
}
