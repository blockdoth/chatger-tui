pub mod chat;
pub mod events;
pub mod framework;
pub mod logs;
pub mod ui;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;

use anyhow::{Ok, Result, anyhow};
use async_trait::async_trait;
use chrono::{DateTime, Days, NaiveDateTime, Utc};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use log::{debug, error, info, warn};
use ratatui::Frame;
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::watch::error;
use tokio::time::sleep;

use crate::cli::{AppConfig, CliArgs};
use crate::network::client::{self, Client};
use crate::network::protocol::UserStatus;
use crate::tui::chat::{ChannelId, ChannelStatus, ChatMessage, ChatMessageStatus, CurrentUser, DisplayChannel, User};
use crate::tui::events::TuiEvent;
use crate::tui::framework::{FromLog, Tui, TuiRunner};
use crate::tui::logs::LogEntry;
use crate::tui::ui::draw;

pub struct UserProfile {
    id: u64,
    username: String,
    password: String,
    is_logged_in: bool,
}

#[derive(Debug, PartialEq)]
pub enum Focus {
    Channels,
    ChatHistory,
    ChatInput(usize),
    Users,
    Logs,
}

#[derive(Debug, PartialEq)]
pub enum ServerState {
    Connected,
    Unhealhty,
    Disconnected,
    Reconnecting,
}

pub struct State {
    should_quit: bool,
    logs: Vec<LogEntry>,
    channels: Vec<DisplayChannel>,
    users: Vec<User>,
    chat_history: HashMap<ChannelId, Vec<ChatMessage>>,
    chat_input: String,
    active_channel_idx: usize,
    focus: Focus,
    current_user: Option<UserProfile>,
    last_healthcheck: DateTime<Utc>,
    show_logs: bool,
    log_scroll_offset: usize,
    chat_scroll_offset: usize,
    server_address: Option<SocketAddr>,
    server_connection_state: ServerState,
}

impl State {
    pub fn new() -> Self {
        State {
            should_quit: false,
            last_healthcheck: Utc::now().checked_sub_days(Days::new(1)).unwrap(),
            show_logs: true,
            log_scroll_offset: 0,
            chat_scroll_offset: 0,
            logs: vec![],
            active_channel_idx: 0,
            server_connection_state: ServerState::Disconnected,
            server_address: None,
            focus: Focus::Channels,
            current_user: None,
            channels: vec![],
            users: vec![],
            chat_history: HashMap::new(),
            chat_input: " ".to_owned(),
        }
    }
}

#[async_trait]
impl Tui<TuiEvent> for State {
    /// Draws the UI layout and content.
    fn draw_ui(&self, frame: &mut Frame) {
        draw(self, frame);
    }

    fn process_event(&self, event: Event) -> Option<TuiEvent> {
        match event {
            Event::Key(key_event) => match self.focus {
                Focus::Channels => match key_event.code {
                    KeyCode::Up => Some(TuiEvent::ChannelUp),
                    KeyCode::Down => Some(TuiEvent::ChannelDown),
                    KeyCode::Right => Some(TuiEvent::FocusChange(Focus::ChatHistory)),
                    KeyCode::Char('q') | KeyCode::Char('Q') => Some(TuiEvent::Exit),
                    KeyCode::Char('l') | KeyCode::Char('L') => Some(TuiEvent::ToggleLogs),
                    KeyCode::Char(_) => Some(TuiEvent::FocusChange(Focus::ChatInput(0))),
                    _ => None,
                },
                Focus::ChatHistory => match key_event.code {
                    KeyCode::Left => Some(TuiEvent::FocusChange(Focus::Channels)),
                    KeyCode::Right if self.show_logs => Some(TuiEvent::FocusChange(Focus::Logs)),
                    KeyCode::Right => Some(TuiEvent::FocusChange(Focus::Users)),
                    KeyCode::Up => Some(TuiEvent::ScrollUp),
                    KeyCode::Down => Some(TuiEvent::ScrollDown),
                    KeyCode::Char('q') | KeyCode::Char('Q') => Some(TuiEvent::Exit),
                    KeyCode::Char('l') | KeyCode::Char('L') => Some(TuiEvent::ToggleLogs),
                    KeyCode::Char(_) => Some(TuiEvent::FocusChange(Focus::ChatInput(0))),
                    _ => None,
                },
                Focus::ChatInput(_) => match key_event.code {
                    KeyCode::Up => Some(TuiEvent::FocusChange(Focus::ChatHistory)),
                    KeyCode::Left if key_event.modifiers == KeyModifiers::CONTROL => Some(TuiEvent::InputLeftTab),
                    KeyCode::Right if key_event.modifiers == KeyModifiers::CONTROL => Some(TuiEvent::InputRightTab),
                    KeyCode::Left => Some(TuiEvent::InputLeft),
                    KeyCode::Right => Some(TuiEvent::InputRight),
                    KeyCode::Enter => Some(TuiEvent::InputEnter),
                    KeyCode::Char(chr) => Some(TuiEvent::InputChar(chr)),
                    KeyCode::Backspace => Some(TuiEvent::InputDelete),

                    _ => None,
                },
                Focus::Users => match key_event.code {
                    KeyCode::Left if self.show_logs => Some(TuiEvent::FocusChange(Focus::Logs)),
                    KeyCode::Left => Some(TuiEvent::FocusChange(Focus::ChatHistory)),
                    KeyCode::Char('q') | KeyCode::Char('Q') => Some(TuiEvent::Exit),
                    KeyCode::Char('l') | KeyCode::Char('L') => Some(TuiEvent::ToggleLogs),
                    KeyCode::Char(_) => Some(TuiEvent::FocusChange(Focus::ChatInput(0))),
                    _ => None,
                },
                Focus::Logs => match key_event.code {
                    KeyCode::Left => Some(TuiEvent::FocusChange(Focus::ChatHistory)),
                    KeyCode::Right => Some(TuiEvent::FocusChange(Focus::Users)),
                    KeyCode::Up => Some(TuiEvent::ScrollUp),
                    KeyCode::Down => Some(TuiEvent::ScrollDown),
                    KeyCode::Char('q') | KeyCode::Char('Q') => Some(TuiEvent::Exit),
                    KeyCode::Char('l') | KeyCode::Char('L') => Some(TuiEvent::ToggleLogs),
                    KeyCode::Char(_) => Some(TuiEvent::FocusChange(Focus::ChatInput(0))),

                    _ => None,
                },
            },
            _ => None,
        }
    }

    async fn handle_event(&mut self, event: TuiEvent, event_send: &Sender<TuiEvent>, client: &mut Client) -> Result<()> {
        match event {
            TuiEvent::Exit => self.should_quit = true,
            TuiEvent::ToggleLogs => {
                self.show_logs = !self.show_logs;
                self.focus = Focus::ChatHistory;
            }
            TuiEvent::Log(entry) => self.logs.push(entry),
            TuiEvent::ChannelUp => {
                if self.active_channel_idx == 0 {
                    self.active_channel_idx = self.channels.len().saturating_sub(1);
                } else {
                    self.active_channel_idx -= 1;
                }
            }
            TuiEvent::ChannelDown => {
                self.active_channel_idx = (self.active_channel_idx + 1) % self.channels.len();
            }
            TuiEvent::FocusChange(focus) => self.focus = focus,
            TuiEvent::InputLeft => {
                if let Focus::ChatInput(i) = self.focus
                    && i > 0
                {
                    self.focus = Focus::ChatInput(i - 1)
                }
            }
            TuiEvent::InputRight => {
                if let Focus::ChatInput(i) = self.focus
                    && i + 1 < self.chat_input.len()
                {
                    self.focus = Focus::ChatInput(i + 1)
                }
            }
            TuiEvent::InputLeftTab => {
                if let Focus::ChatInput(i) = self.focus
                    && i > 0
                {
                    let idx = self
                        .chat_input
                        .char_indices()
                        .take(i)
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .skip_while(|(_, c)| *c != ' ')
                        // .skip_while(|(_, c)| *c == ' ')
                        .map(|(idx, _)| idx)
                        .next()
                        .unwrap_or_else(|| 0);

                    self.focus = Focus::ChatInput(idx)
                }
            }
            TuiEvent::InputRightTab => {
                if let Focus::ChatInput(i) = self.focus
                    && i + 1 < self.chat_input.len()
                {
                    let idx = self
                        .chat_input
                        .char_indices()
                        .skip(i + 1)
                        .skip_while(|(_, c)| *c != ' ')
                        // .skip_while(|(_, c)| *c == ' ')
                        .map(|(idx, _)| idx)
                        .next()
                        .unwrap_or_else(|| self.chat_input.len());
                    self.focus = Focus::ChatInput(idx)
                }
            }
            TuiEvent::InputDelete => {
                if let Focus::ChatInput(i) = self.focus
                    && i > 0
                {
                    self.chat_input.remove(i - 1);
                    self.focus = Focus::ChatInput(i - 1)
                }
            }
            TuiEvent::InputEnter if self.chat_input.len() > 1 => {
                if let Some(user) = &self.current_user {
                    // command_send.send(Command::SendMessage(self.chat_input.clone())).await?;
                    let message = ChatMessage {
                        id: None,
                        author_name: user.username.to_owned(),
                        author_id: user.id,
                        reply_id: 0, // TODO replies
                        timestamp: Utc::now(),
                        message: self.chat_input.clone(),
                        status: ChatMessageStatus::Sending,
                    };
                    let channel_id = self.channels.get(self.active_channel_idx).unwrap().id; // TODO better

                    self.chat_history.entry(channel_id).and_modify(|log| log.push(message));

                    client.send_chat_message(channel_id, 0, self.chat_input.clone(), vec![]).await; // TODO improve
                    self.focus = Focus::ChatInput(0);
                    self.chat_input = " ".to_owned();
                } else {
                    todo!("tui notification handling for trying to send a message while not logged in")
                }
            }
            TuiEvent::InputEnter => {} // Do nothing if above case falls through
            TuiEvent::ScrollDown => match self.focus {
                Focus::ChatHistory => {
                    self.chat_scroll_offset = self.chat_scroll_offset.saturating_sub(1);
                }
                Focus::Logs => {
                    self.log_scroll_offset = self.log_scroll_offset.saturating_sub(1);
                }
                _ => {}
            },
            TuiEvent::ScrollUp => match self.focus {
                Focus::ChatHistory => {
                    self.chat_scroll_offset = self.chat_scroll_offset.saturating_add(1);
                }
                Focus::Logs => {
                    self.log_scroll_offset = self.log_scroll_offset.saturating_add(1);
                }
                _ => {}
            },
            TuiEvent::InputChar(chr) => {
                if let Focus::ChatInput(i) = self.focus {
                    self.chat_input.insert(i, chr);

                    self.focus = Focus::ChatInput(i + 1)
                }
            }
            TuiEvent::SetUserNamePassword(username, password) => {
                self.current_user = Some(UserProfile {
                    id: 0,
                    username,
                    password,
                    is_logged_in: false,
                })
            }
            TuiEvent::ConnectAndLogin(address, username, password) => {
                client.connect(address).await?;
                client.login(username.clone(), password.clone()).await?;
                self.server_connection_state = ServerState::Connected;
                event_send.send(TuiEvent::SetUserNamePassword(username, password)).await;
                self.server_address = Some(address);
            }
            TuiEvent::LoggedIn => {
                if let Some(user) = &mut self.current_user {
                    user.is_logged_in = true;
                    client.request_channel_ids().await?;
                    client.request_user_statuses().await?;
                }
            }
            TuiEvent::ChannelIDs(channel_ids) => {
                if !channel_ids.is_empty() {
                    debug!("received channel ids {channel_ids:?}");
                    client.request_channels(channel_ids).await?
                }
            }
            TuiEvent::HealthCheck => {
                self.last_healthcheck = Utc::now();
                client.request_user_statuses().await?;
            }

            TuiEvent::Channels(channels) => {
                debug!("received {channels:?}");
                for channel in channels {
                    // I want to add the channel first and only then request
                    // if I requested first to make the borrow checker happy it could fail and end up in a broken state
                    // history would be incoming for a channel which is not added
                    let channel_id = channel.channel_id;

                    self.channels.push(channel.into());
                    client.request_history_by_timestamp(channel_id, Utc::now(), 50).await?;
                }
            }
            TuiEvent::UserStatusesUpdate(status_updates) => {
                // TODO what happens if a new user comes online? We dont get their name
                debug!("received statuses{status_updates:?}");

                let mut users_not_found = vec![];
                'outer: for status_update in status_updates {
                    for user in &mut self.users {
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
                    client.request_users(users_not_found).await;
                }
            }
            TuiEvent::Users(users) => {
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
                for user in &mut self.users {
                    if let Some(new_user) = new_users_map.remove(&user.id) {
                        user.status = new_user.status;
                    }
                }
                self.users.extend(new_users_map.into_values());
            }
            TuiEvent::HistoryUpdate(messages) => {
                for message in messages {
                    let author_name = self
                        .users
                        .iter()
                        .find(|user| user.id == message.user_id)
                        .map(|user| user.name.clone())
                        .unwrap_or_else(|| "Unknown".to_string());

                    let timestamp = DateTime::from_timestamp(message.sent_timestamp as i64, 0).ok_or_else(|| anyhow!("Invalid timestamp"))?;

                    let display_message = ChatMessage {
                        id: Some(message.message_id),
                        reply_id: message.message_id,
                        author_name,
                        author_id: message.user_id,
                        timestamp,
                        message: message.message_text,
                        status: ChatMessageStatus::Send,
                    };

                    let channel_id = message.channel_id;
                    // TODO figure out what to do when we get message from channels we dont know the name off
                    let display_messages = self.chat_history.entry(channel_id).or_default();

                    if !display_messages.iter().any(|m| m.id == display_message.id) {
                        debug!("inserting {display_message:?} into history of channel {channel_id}");
                        display_messages.push(display_message);
                    }
                }
            }
            TuiEvent::MessageSendAck(message_id) => {
                info!("{message_id}");
                if let Some(message) = self
                    .chat_history
                    .iter_mut()
                    .find_map(|(channel_id, messages)| messages.iter_mut().find(|m| m.id == Some(message_id)))
                {
                    // Update the message status
                    message.status = ChatMessageStatus::Send;
                } else {
                    info!("Message with id {message_id} not found in chat history");
                }
            }

            TuiEvent::Disconnected => {
                self.server_connection_state = ServerState::Disconnected;
                error!("TOOD reconnect logic");

                // TOOD reconnect logic

                // loop {
                //   if let Some(address) = self.server_address && let Some(UserProfile{username, password, .. }) = &self.current_user {
                //     if self.server_connection_state != ServerState::Reconnecting {
                //       self.server_connection_state = ServerState::Reconnecting;
                //       event_send.send(TuiEvent::ConnectAndLogin(address, username.clone(), password.clone())).await;
                //     }
                //   }
                //   sleep(Duration::from_secs(5)).await;
                // }
            }
        }
        Ok(())
    }

    async fn on_tick(&mut self) -> Result<()> {
        Ok(())
    }

    fn should_quit(&self) -> bool {
        self.should_quit
    }
}

pub async fn run(config: AppConfig) -> Result<()> {
    let (event_send, event_recv) = mpsc::channel::<TuiEvent>(10);

    let mut client = Client::new(event_send.clone());

    let username = config.username.clone();
    let password = config.password.clone();
    event_send.send(TuiEvent::ConnectAndLogin(config.address, username, password)).await?;

    let tasks = vec![async move {}];

    let tui = State::new();
    let tui_runner = TuiRunner::new(tui, client, event_recv, event_send, config.loglevel);

    tui_runner.run(tasks).await
}
