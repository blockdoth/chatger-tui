use std::net::SocketAddr;

use clap::Parser;
use log::LevelFilter;

/// Simple CLI to simulate login
#[derive(Parser, Debug)]
#[command(name = "chatger", version = "1.0", author = "blockdoth", about = "A chatger TUI client")]
pub struct CliArgs {
    /// Server address of chatger server to connect to
    #[arg(long, default_value = "0.0.0.0:4348")]
    pub address: String,

    /// Username
    #[arg(long, default_value = "penger")]
    pub username: String,

    /// Password
    #[arg(long, default_value = "password6")]
    pub password: String,

    /// Log level (error, warn, info, debug, trace)
    #[arg(long, default_value_t = LevelFilter::Info)]
    pub loglevel: LevelFilter,

    /// Automatically login
    #[arg(long, default_value_t = false)]
    pub auto_login: bool,
}

pub struct AppConfig {
    pub address: SocketAddr,
    pub username: String,
    pub password: String,
    pub auto_login: bool,
    pub loglevel: LevelFilter,
}
