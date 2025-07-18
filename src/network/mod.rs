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
            HealthKind::Pong => panic!("todo"),
        },
        Login(packet) => match packet.status {
            Success => {
                info!("Succefully logged in");
                event_send.send(TuiEvent::LoginSuccess(0)).await?; // TODO user id handling
                Ok(())
            }
            Failed => match packet.error_message {
                Some(message) => {
                    event_send.send(TuiEvent::LoginFail(message.clone())).await?; // TODO distinction between username and password fail
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
            Success | Notification => {
                event_send.send(TuiEvent::HistoryUpdate(packet.messages)).await?;
                Ok(())
            }
            Failed => todo!(),
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
                event_send.send(TuiEvent::MessageMediaAck(packet.media_id)).await?;
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
                error!("Failed to send media {:?}", packet.error_message);
                Ok(())
            }
            Notification => {
                info!("Got message notification from server TODO handle");
                Ok(())
            }
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
