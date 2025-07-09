pub mod client;
pub mod protocol;

use std::net::SocketAddr;

use log::{error, info};

use crate::network::client::Client;

pub async fn start_client(address: SocketAddr, username: String, password: String) {
    info!("Starting client with args: {address} {username} {password}");

    let mut client = Client::new();
    if let Err(e) = client.connect(address).await {
        error!("Failed to connect {e}");
    }
}
