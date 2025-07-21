use anyhow::{Result, anyhow};
use log::{error, info};
use tokio::sync::mpsc::Sender;

use crate::network::protocol::server::{HealthKind, ReturnStatus, ServerPayload};
use crate::tui::chat::MediaMessage;
use crate::tui::events::TuiEvent;
pub mod client;
pub mod protocol;

pub async fn handle_message(payload: ServerPayload, event_send: Sender<TuiEvent>) -> Result<()> {
    use ServerPayload::*;

    use self::ReturnStatus::*;

    match payload {
        Health(packet) => match packet.kind {
            HealthKind::Ping => {
                event_send.send(TuiEvent::HealthCheckRecv).await?;
                Ok(())
            }
            HealthKind::Pong => Err(anyhow!("Received Pong packet, which are meant for the server only")),
        },
        Login(packet) => match packet.status {
            Success => {
                info!("Succefully logged in");
                event_send.send(TuiEvent::LoginSuccess(0)).await?; // TODO user id handling
                Ok(())
            }
            Failed => {
                if let Some(message) = packet.error_message {
                    event_send.send(TuiEvent::LoginFail(message.clone())).await?; // TODO distinction between username and password fail
                    Err(anyhow!("Failed to login: {message}"))
                } else {
                    Err(anyhow!("Failed to login"))
                }
            }
            Notification => Err(anyhow!("Malformed packet, notification bit should not be set")),
        },
        Channels(packet) => match packet.status {
            Success => {
                event_send.send(TuiEvent::Channels(packet.channels)).await?;
                Ok(())
            }
            Failed => {
                if let Some(message) = packet.error_message {
                    Err(anyhow!("Failed to retrieve channels: {message}"))
                } else {
                    Err(anyhow!("Failed to retrieve channels"))
                }
            }
            Notification => Err(anyhow!("Malformed packet, notification bit should not be set")),
        },
        ChannelsList(packet) => match packet.status {
            Success => {
                event_send.send(TuiEvent::ChannelIDs(packet.channel_ids)).await?;
                Ok(())
            }
            Failed => {
                if let Some(message) = packet.error_message {
                    Err(anyhow!("Failed to retrieve channels list: {message}"))
                } else {
                    Err(anyhow!("Failed to retrieve channels list"))
                }
            }
            Notification => Err(anyhow!("Malformed packet, notification bit should not be set")),
        },
        UserStatuses(packet) => match packet.status {
            Success => {
                event_send.send(TuiEvent::UserStatusesUpdate(packet.users)).await?;
                Ok(())
            }
            Failed => {
                if let Some(message) = packet.error_message {
                    Err(anyhow!("Failed to retrieve user statuses: {message}"))
                } else {
                    Err(anyhow!("Failed to retrieve user statuses"))
                }
            }
            Notification => Err(anyhow!("Malformed packet, notification bit should not be set")),
        },
        Users(packet) => match packet.status {
            Success => {
                event_send.send(TuiEvent::Users(packet.users)).await?;
                Ok(())
            }
            Failed => {
                if let Some(message) = packet.error_message {
                    Err(anyhow!("Failed to retrieve users: {message}"))
                } else {
                    Err(anyhow!("Failed to retrieve users"))
                }
            }
            Notification => Err(anyhow!("Malformed packet, notification bit should not be set")),
        },
        History(packet) => match packet.status {
            Success | Notification => {
                event_send.send(TuiEvent::HistoryUpdate(packet.messages)).await?;
                Ok(())
            }
            Failed => {
                if let Some(message) = packet.error_message {
                    Err(anyhow!("Failed to retrieve history: {message}"))
                } else {
                    Err(anyhow!("Failed to retrieve history"))
                }
            }
        },
        SendMessageAck(packet) => match packet.status {
            Success => {
                event_send.send(TuiEvent::MessageSendAck(packet.message_id)).await?;
                Ok(())
            }
            Failed => {
                if let Some(message) = packet.error_message {
                    Err(anyhow!("Failed to send message: {message}"))
                } else {
                    Err(anyhow!("Failed to send message-but"))
                }
            }
            Notification => Err(anyhow!("Malformed packet, notification bit should not be set")),
        },
        SendMediaAck(packet) => match packet.status {
            Success => {
                event_send.send(TuiEvent::MessageMediaAck(packet.media_id)).await?;
                Ok(())
            }
            Failed => {
                if let Some(message) = packet.error_message {
                    Err(anyhow!("Failed to send media: {message}"))
                } else {
                    Err(anyhow!("Failed to send media"))
                }
            }
            Notification => Err(anyhow!("Malformed packet, notification bit should not be set")),
        },
        Media(packet) => match packet.status {
            Success => {
                event_send
                    .send(TuiEvent::Media(MediaMessage {
                        filename: packet.filename,
                        media_type: packet.media_type,
                        media_data: packet.media_data,
                    }))
                    .await?;
                Ok(())
            }
            Failed => {
                if let Some(message) = packet.error_message {
                    Err(anyhow!("Failed to retrieve media: {message}"))
                } else {
                    Err(anyhow!("Failed to retrieve media"))
                }
            }
            Notification => Err(anyhow!("Malformed packet, notification bit should not be set")),
        },
        Typing(packet) => {
            event_send
                .send(TuiEvent::Typing(packet.channel_id, packet.user_id, packet.is_typing))
                .await?;
            Ok(())
        }
        Status(packet) => {
            event_send.send(TuiEvent::UserStatusUpdate(packet.user_id, packet.status)).await?;
            Ok(())
        }
    }
}
