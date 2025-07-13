use std::sync::Arc;

use anyhow::{Result, anyhow};
use log::{error, info};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Sender;

use crate::network::client::Client;
use crate::network::protocol::client::{ClientPacketType, ClientPayload};
use crate::network::protocol::server::{HealthCheckPacket, HealthKind, ServerPayload, Status};
use crate::tui::events::TuiEvent;
pub mod client;
pub mod protocol;

pub async fn handle_message(payload: ServerPayload, stream: &mut Arc<Mutex<OwnedWriteHalf>>, event_send: Sender<TuiEvent>) -> Result<()> {
    use ServerPayload::*;
    use Status::*;

    match payload {
        Health(packet) => match packet.kind {
            HealthKind::Ping => {
                let mut stream = stream.lock().await;
                let payload = ClientPayload::Health(HealthCheckPacket { kind: HealthKind::Pong });
                Client::send_message(&mut stream, ClientPacketType::Healthcheck, payload).await?;
                event_send.send(TuiEvent::HealthCheck).await?;
                Ok(())
            }
            HealthKind::Pong => panic!("todo"),
        },
        Login(packet) => match packet.status {
            Success => {
                info!("Succefully logged in");
                event_send.send(TuiEvent::LoggedIn).await?;
                Ok(())
            }
            Failed => match packet.error_message {
                Some(message) => {
                    error!("failed to log in {message}");
                    Err(anyhow!("failed to log in {}", message))
                }
                None => {
                    error!("failed to log in");
                    Err(anyhow!("failed to log in"))
                }
            },
            Notification => panic!("todo"),
        },
        Channels(packet) => match packet.status {
            Success => {
                event_send.send(TuiEvent::Channels(packet.channels)).await?;
                Ok(())
            }
            Failed => todo!(),
            Notification => panic!("todo"),
        },
        ChannelsList(packet) => match packet.status {
            Success => {
                event_send.send(TuiEvent::ChannelIDs(packet.channel_ids)).await?;
                Ok(())
            }
            Failed => todo!(),
            Notification => panic!("todo"),
        },
        UserStatuses(packet) => match packet.status {
            Success => {
                event_send.send(TuiEvent::UserStatusesUpdate(packet.users)).await?;
                Ok(())
            }
            Failed => todo!(),
            Notification => panic!("todo"),
        },
        Users(packet) => match packet.status {
            Success => {
                event_send.send(TuiEvent::Users(packet.users)).await?;
                Ok(())
            }
            Failed => todo!(),
            Notification => panic!("todo"),
        },
        History(packet) => match packet.status {
            Success => {
                event_send.send(TuiEvent::HistoryUpdate(packet.messages)).await?;
                Ok(())
            }
            Failed => todo!(),
            Notification => panic!("todo"),
        },
        SendMessageAck(packet) => match packet.status {
            Success => {
                event_send.send(TuiEvent::MessageSendAck(packet.message_id)).await?;
                Ok(())
            }
            Failed => {
                error!("Failed to send message {:?}", packet.error_message);
                Ok(())
            }
            Notification => {
                info!("Got message notification from server TODO handle");
                Ok(())
            }
        },
        SendMediaAck(packet) => match packet.status {
            Success => {
                todo!();
                // event_send.send(TuiEvent::MessageSendAck(packet.message_id)).await?;
                Ok(())
            }
            Failed => {
                error!("Failed to send media {:?}", packet.error_message);
                Ok(())
            }
            Notification => {
                info!("Got message notification from server TODO handle");
                Ok(())
            }
        },
    }
}
