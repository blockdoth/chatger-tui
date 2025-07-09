use std::net::SocketAddr;

use anyhow::{Result, anyhow};
use clap::Error;
use log::info;
use tokio::net::TcpStream;

pub struct Client {
    is_connected: bool,
    stream: Option<TcpStream>,
}

impl Client {
    pub fn new() -> Self {
        Client {
            is_connected: false,
            stream: None,
        }
    }

    pub async fn connect(&mut self, target_addr: SocketAddr) -> Result<()> {
        if self.is_connected {
            return Err(anyhow!("Already connected to {}", target_addr));
        }

        let stream = TcpStream::connect(target_addr).await?;
        let src_addr = stream.local_addr().unwrap();

        self.stream = Some(stream);

        info!("Connected to {target_addr} from {src_addr}");

        Ok(())
    }
}
