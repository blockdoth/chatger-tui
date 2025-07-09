use std::net::SocketAddr;

use clap::Parser;

/// Simple CLI to simulate login
#[derive(Parser, Debug)]
#[command(name = "chatger", version = "1.0", author = "blockdoth", about = "A chatger TUI client")]
pub struct CliArgs {
    #[arg(long)]
    pub address: String,

    #[arg(long)]
    pub username: String,

    #[arg(long)]
    pub password: String,
}

pub struct AppConfig {
    pub address: SocketAddr,
    pub username: String,
    pub password: String,
}
