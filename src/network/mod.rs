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
use tokio::time::{Duration, sleep};

use crate::network::client::Client;
use crate::network::protocol::{Packet, Payload};
use crate::tui::TuiEvent;

pub async fn start_client(event_send: Sender<TuiEvent>, address: SocketAddr, username: String, password: String) {
    info!("Starting client with args: {address} {username} {password}");

    let mut client = Client::new();
    {
        let mut client = client.lock().await;
        if let Err(e) = client.connect(address).await {
            error!("Failed to connect {e}");
        }

        if let Err(e) = client.login(username.clone(), password).await {
            error!("Failed to login {e}");
        }

        event_send.send(TuiEvent::LoggedIn(username)).await.expect("todo error handling");
    }
    let polling_interval = 1;

    start_polling(client, address, polling_interval, event_send).await;
}

pub async fn start_polling(client: Arc<Mutex<Client>>, address: SocketAddr, polling_interval: u64, event_send: Sender<TuiEvent>) -> JoinHandle<()> {
    tokio::spawn(async move {
        info!("Started polling task");
        let mut is_healthy = true;
        loop {
            let mut client = client.lock().await;

            if !is_healthy {
                if let Err(e) = client.connect(address).await {
                    error!("Failed to reconnect to server {address:?}");
                } else {
                    info!("Succesfully reconnected to server {address:?}");
                }
            }

            if let Err(e) = client.healthcheck().await {
                event_send.send(TuiEvent::HealthCheck(false)).await.expect("TODO error handling");
                error!("Healthcheck failed {e}");
            } else {
                event_send.send(TuiEvent::HealthCheck(true)).await.expect("TODO error handling");
                debug!("Healthcheck succeeded");
            };

            sleep(Duration::from_secs(polling_interval)).await;
        }
    })
}
