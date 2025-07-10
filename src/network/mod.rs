pub mod client;
pub mod protocol;

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use log::{debug, error, info, trace};
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::Mutex;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

use crate::network::client::{Client};
use crate::tui::TuiEvent;

pub async fn start_client(event_send: Sender<TuiEvent>, address: SocketAddr, username: String, password: String) {
    info!("Starting client with args: {address} {username} {password}");

    let mut client = Client::new();

    if let Err(e) = client.connect(address).await {
        error!("Failed to connect {e}");
    }

    if let Err(e) = client.login(username.clone(), password).await {
        error!("Failed to login {e}");
    }

    event_send.send(TuiEvent::LoggedIn(username)).await.expect("todo error handling");

    let polling_interval = 1;
}
