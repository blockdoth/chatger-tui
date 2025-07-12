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

use crate::network::client::Client;
use crate::tui::events::TuiEvent;
