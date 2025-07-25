mod cli;
mod network;
mod tui;
use anyhow::Result;
use clap::Parser;

use crate::cli::{AppConfig, CliArgs};

#[tokio::main]
async fn main() -> Result<()> {
    let args = CliArgs::parse();

    // TODO merge cli args and app config
    let config = AppConfig {
        address: args.address,
        port: args.port,
        username: args.username,
        password: args.password,
        loglevel: args.loglevel,
        auto_login: args.auto_login,
        enable_tls: args.enable_tls,
    };

    tui::run(config).await
}
