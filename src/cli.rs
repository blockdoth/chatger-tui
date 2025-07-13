use std::net::SocketAddr;

use clap::Parser;
use log::LevelFilter;

/// Simple CLI to simulate login
#[derive(Parser, Debug)]
#[command(name = "chatger", version = "1.0", author = "blockdoth", about = "A chatger TUI client")]
pub struct CliArgs {
    /// Server address to connect to
    #[arg(long, default_value = "127.0.0.1:8080")]
    pub address: String,

    /// Username for login
    #[arg(long, default_value = "penger")]
    pub username: String,

    /// Password for login
    #[arg(long, default_value = "epicpass4")]
    pub password: String,

    /// Log level (error, warn, info, debug, trace)
    #[arg(long, default_value_t = LevelFilter::Debug)]
    pub loglevel: LevelFilter,
}

pub struct AppConfig {
    pub address: SocketAddr,
    pub username: String,
    pub password: String,
    pub loglevel: LevelFilter,
}
