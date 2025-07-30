use std::net::SocketAddr;

use clap::Parser;
use log::LevelFilter;

use crate::network::client::ConnectionType;

pub const DEFAULT_ADDRESS: &str = "0.0.0.0";
pub const DEFAULT_PORT: u16 = 4348;

/// Simple CLI to simulate login
#[derive(Parser, Debug)]
#[command(name = "chatger", version = "1.0", author = "blockdoth", about = "A chatger TUI client")]
pub struct CliArgs {
    /// Server address of chatger server to connect to
    #[arg(long, default_value = DEFAULT_ADDRESS)]
    pub address: String,

    /// Server port of chatger server to connect to
    #[arg(long, default_value_t = DEFAULT_PORT)]
    pub port: u16,

    /// Username
    #[arg(long, default_value = "penger")]
    pub username: String,

    /// Password
    #[arg(long, default_value = "epicpass4")]
    pub password: String,

    /// Log level (error, warn, info, debug, trace)
    #[arg(long, default_value_t = LevelFilter::Info)]
    pub loglevel: LevelFilter,

    /// Automatically login
    #[arg(long, default_value_t = false)]
    pub auto_login: bool,

    /// Enable TLS encryption
    #[arg(long, default_value_t = false)]
    pub enable_tls: bool,
}

pub struct AppConfig {
    pub address: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub auto_login: bool,
    pub loglevel: LevelFilter,
    pub enable_tls: bool,
}
