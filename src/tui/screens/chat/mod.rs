pub mod borders;
pub mod keys;
pub mod ui;

use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;

use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use log::{debug, error, info};
use tokio::sync::mpsc::Sender;

use crate::network::client::Client;
use crate::tui::chat::{ChatMessage, ChatMessageStatus, DisplayChannel, User};
use crate::tui::events::{ChannelId, MessageId, TuiEvent, UserId};
use crate::tui::{AppState, Screen, State};

#[derive(Clone, Debug)]
pub struct ChatState {
    pub focus: ChatFocus,
    pub channels: Vec<DisplayChannel>,
    pub users: Vec<User>,
    pub chat_history: HashMap<ChannelId, Vec<ChatMessage>>,
    pub chat_inputs: HashMap<ChannelId, String>,
    pub active_channel_idx: usize,
    pub current_user: UserProfile,
    pub chat_scroll_offset: usize,
    pub last_healthcheck: DateTime<Utc>,
    pub server_address: SocketAddr,
    pub server_connection_state: ServerState,
    pub waiting_message_acks_id: VecDeque<MessageId>,
    pub incrementing_ack_id: MessageId,
}

#[derive(Clone, Debug)]
pub struct UserProfile {
    pub user_id: UserId,
    pub username: String,
    pub password: String,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ChatFocus {
    Channels,
    ChatHistory,
    ChatInput(usize),
    Users,
    Logs,
}
#[derive(Debug, PartialEq, Clone)]
pub enum ServerState {
    Connected,
    Unhealthy,
    Disconnected,
    Reconnecting,
}

pub async fn handle_chat_event(tui: &mut State, event: TuiEvent, event_send: &Sender<TuiEvent>, client: &mut Client) -> Result<()> {
    let mut chat_state = match &mut tui.current_state {
        AppState::Chat(chat_state) => chat_state,
        _ => panic!("This function only handles the chat state"),
    };

    use TuiEvent::*;

    match event {
        Exit => tui.global_state.should_quit = true,
        ToggleLogs => {
            tui.global_state.show_logs = !tui.global_state.show_logs;
            chat_state.focus = ChatFocus::ChatHistory;
        }
        Log(entry) => tui.global_state.logs.push(entry),
        ChannelUp => {
            if chat_state.active_channel_idx == 0 {
                chat_state.active_channel_idx = chat_state.channels.len().saturating_sub(1);
            } else {
                chat_state.active_channel_idx -= 1;
            }
        }
        ChannelDown => {
            chat_state.active_channel_idx = (chat_state.active_channel_idx + 1) % chat_state.channels.len();
        }
        ChatFocusChange(focus) => chat_state.focus = focus,
        InputLeft => {
            if let ChatFocus::ChatInput(i) = chat_state.focus
                && i > 0
            {
                chat_state.focus = ChatFocus::ChatInput(i - 1)
            }
        }
        InputRight => {
            if let ChatFocus::ChatInput(i) = chat_state.focus
                && i + 1 < chat_state.chat_inputs.len()
            {
                chat_state.focus = ChatFocus::ChatInput(i + 1)
            }
        }
        InputLeftTab => {
            if let ChatFocus::ChatInput(i) = chat_state.focus
                && i > 0
                && let Some(channel_id) = chat_state.channels.get(chat_state.active_channel_idx)
                && let Some(input_line) = chat_state.chat_inputs.get(&channel_id.id)
            {
                let idx = input_line
                    .char_indices()
                    .take(i)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .skip_while(|(_, c)| *c != ' ')
                    .map(|(idx, _)| idx)
                    .next()
                    .unwrap_or(0);

                chat_state.focus = ChatFocus::ChatInput(idx)
            }
        }
        InputRightTab => {
            if let ChatFocus::ChatInput(i) = chat_state.focus
                && i + 1 < chat_state.chat_inputs.len()
                && let Some(channel_id) = chat_state.channels.get(chat_state.active_channel_idx)
                && let Some(input_line) = chat_state.chat_inputs.get(&channel_id.id)
            {
                let idx = input_line
                    .char_indices()
                    .skip(i + 1)
                    .skip_while(|(_, c)| *c != ' ')
                    // .skip_while(|(_, c)| *c == ' ')
                    .map(|(idx, _)| idx)
                    .next()
                    .unwrap_or(chat_state.chat_inputs.len());
                chat_state.focus = ChatFocus::ChatInput(idx)
            }
        }
        InputDelete => {
            if let ChatFocus::ChatInput(i) = chat_state.focus
                && i > 0
                && let Some(channel_id) = chat_state.channels.get(chat_state.active_channel_idx)
                && let Some(input_line) = chat_state.chat_inputs.get_mut(&channel_id.id)
            {
                input_line.remove(i - 1);
                chat_state.focus = ChatFocus::ChatInput(i - 1)
            }
        }

        MessageSend => {
            let channel_id = chat_state
                .channels
                .get(chat_state.active_channel_idx)
                .map(|c| c.id)
                .ok_or_else(|| anyhow!("channel not found based on index"))?;

            let input_line = chat_state
                .chat_inputs
                .get_mut(&channel_id)
                .ok_or_else(|| anyhow!("No input found for channel {}", channel_id))?;

            if !input_line.trim().is_empty() {
                // Don't send empty or whitespace-only messages

                let temp_message_id = chat_state.incrementing_ack_id;
                let message = ChatMessage {
                    message_id: temp_message_id,
                    author_name: chat_state.current_user.username.to_owned(),
                    author_id: chat_state.current_user.user_id,
                    reply_id: 0, // TODO replies
                    timestamp: Utc::now(),
                    message: input_line.clone(),
                    status: ChatMessageStatus::Sending,
                };
                chat_state.waiting_message_acks_id.push_back(temp_message_id);
                chat_state.incrementing_ack_id += 1;

                chat_state
                    .chat_history
                    .entry(channel_id)
                    .and_modify(|message_history| message_history.push(message));

                client.send_chat_message(channel_id, 0, input_line.clone(), vec![]).await?; // TODO improve
                chat_state.focus = ChatFocus::ChatInput(0);
                *input_line = "".to_owned();
            }
        }
        MessageSendAck(message_id) => {
            if let Some(temp_message_id) = chat_state.waiting_message_acks_id.pop_back() {
                if let Some(message) = chat_state
                    .chat_history
                    .values_mut()
                    .flat_map(|messages| messages.iter_mut())
                    .find(|m| m.message_id == temp_message_id)
                {
                    message.status = ChatMessageStatus::Send;
                    message.message_id = message_id;
                } else {
                    chat_state.waiting_message_acks_id.push_front(temp_message_id);
                }
            } else {
                // TODO more logic maybe
                error!("No message is waiting for ack");
            }
        }
        ScrollDown => match chat_state.focus {
            ChatFocus::ChatHistory => {
                chat_state.chat_scroll_offset = chat_state.chat_scroll_offset.saturating_sub(1);
            }
            ChatFocus::Logs => {
                tui.global_state.log_scroll_offset = tui.global_state.log_scroll_offset.saturating_sub(1);
            }
            _ => {}
        },
        ScrollUp => match chat_state.focus {
            ChatFocus::ChatHistory => {
                chat_state.chat_scroll_offset = chat_state.chat_scroll_offset.saturating_add(1);
            }
            ChatFocus::Logs => {
                tui.global_state.log_scroll_offset = tui.global_state.log_scroll_offset.saturating_add(1);
            }
            _ => {}
        },
        InputChar(chr) => {
            if let ChatFocus::ChatInput(i) = chat_state.focus
                && let Some(channel_id) = chat_state.channels.get(chat_state.active_channel_idx)
                && let Some(input_line) = chat_state.chat_inputs.get_mut(&channel_id.id)
            {
                input_line.insert(i, chr);
                chat_state.focus = ChatFocus::ChatInput(i + 1)
            }
        }

        ChannelIDs(channel_ids) => {
            if !channel_ids.is_empty() {
                debug!("received channel ids {channel_ids:?}");
                client.request_channels(channel_ids).await?
            }
        }
        HealthCheck => {
            chat_state.last_healthcheck = Utc::now();
            client.request_user_statuses().await?;
        }

        Channels(channels) => {
            debug!("received {channels:?}");
            for channel in channels {
                // I want to add the channel first and only then request
                // if I requested first to make the borrow checker happy it could fail and end up in a broken state
                // history would be incoming for a channel which is not added
                let channel_id = channel.channel_id;
                chat_state.chat_inputs.insert(channel_id, "".to_owned());
                chat_state.channels.push(channel.into());
                client.request_history_by_timestamp(channel_id, Utc::now(), 50).await?;
            }
        }
        UserStatusesUpdate(status_updates) => {
            // TODO what happens if a new user comes online? We dont get their name
            debug!("received statuses{status_updates:?}");

            let mut users_not_found = vec![];
            'outer: for status_update in status_updates {
                for user in &mut chat_state.users {
                    if user.id == status_update.0 {
                        user.status = status_update.1.clone();
                        continue 'outer;
                    }
                }
                // User not found in current users
                users_not_found.push(status_update.0);
            }
            if !users_not_found.is_empty() {
                debug!("New users added, requesting names of users ids {users_not_found:?}");
                client.request_users(users_not_found).await?;
            }
        }
        Users(users) => {
            let mut new_users: Vec<User> = users
                .iter()
                .map(|user| User {
                    id: user.user_id,
                    name: user.username.clone(),
                    status: user.status.clone(),
                })
                .collect();

            let mut new_users_map: HashMap<u64, User> = new_users.drain(..).map(|user| (user.id, user)).collect();

            // Update existing users
            for user in &mut chat_state.users {
                if let Some(new_user) = new_users_map.remove(&user.id) {
                    user.status = new_user.status;
                }
            }
            chat_state.users.extend(new_users_map.into_values());
        }
        HistoryUpdate(messages) => {
            for message in messages {
                let author_name = chat_state
                    .users
                    .iter()
                    .find(|user| user.id == message.user_id)
                    .map(|user| user.name.clone())
                    .unwrap_or_else(|| "Unknown".to_string());

                let timestamp = DateTime::from_timestamp(message.sent_timestamp as i64, 0).ok_or_else(|| anyhow!("Invalid timestamp"))?;

                let display_message = ChatMessage {
                    message_id: message.message_id,
                    reply_id: 0,
                    author_name,
                    author_id: message.user_id,
                    timestamp,
                    message: message.message_text,
                    status: ChatMessageStatus::Send,
                };

                let channel_id = message.channel_id;
                // TODO figure out what to do when we get message from channels we dont know the name off
                let display_messages = chat_state.chat_history.entry(channel_id).or_default();

                if !display_messages.iter().any(|m| m.message_id == display_message.message_id) {
                    display_messages.push(display_message);
                }
            }
        }
        Logout => {
            if let Some(login_state) = tui.state_map.get(&Screen::Login).cloned() {
                chat_state.chat_history.values_mut().for_each(|messages| {
                    messages.iter_mut().for_each(|msg| {
                        if msg.status == ChatMessageStatus::Sending {
                            msg.status = ChatMessageStatus::FailedToSend;
                        }
                    });
                });
                chat_state.waiting_message_acks_id.clear();

                client.disconnect()?;
                let user = &chat_state.current_user;
                tui.state_map.insert(
                    Screen::Chat(
                        user.username.trim().to_string(),
                        user.password.trim().to_string(),
                        chat_state.server_address.to_string(),
                    ),
                    AppState::Chat(chat_state.clone()),
                );
                tui.current_state = login_state;
                info!("Logging out");
            } else {
                tui.global_state.should_quit = true;
            }
        }

        MessageMediaAck(media_id) => {
            todo!()
        }
        Media(media_message) => {
            todo!()
        }
        Typing(channel_id, user_id, is_typing) => {
            todo!()
        }
        UserStatusUpdate(user_id, status) => {
            todo!()
        }
        Disconnected => {
            chat_state.chat_history.values_mut().for_each(|messages| {
                messages.iter_mut().for_each(|msg| {
                    if msg.status == ChatMessageStatus::Sending {
                        msg.status = ChatMessageStatus::FailedToSend;
                    }
                });
            });
            chat_state.waiting_message_acks_id.clear();

            client.disconnect()?;
            chat_state.server_connection_state = ServerState::Disconnected;
            error!("TOOD reconnect logic");

            // TOOD reconnect logic

            // loop {
            //   if let Some(address) = tui.server_address && let Some(UserProfile{username, password, .. }) = &tui.current_user {
            //     if tui.server_connection_state != ServerState::Reconnecting {
            //       tui.server_connection_state = ServerState::Reconnecting;
            //       event_send.send(TuiEvent::ConnectAndLogin(address, username.clone(), password.clone())).await;
            //     }
            //   }
            //   sleep(Duration::from_secs(5)).await;
            // }
        }
        _ => {}
    }
    Ok(())
}
