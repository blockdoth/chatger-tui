pub mod chat;
pub mod events;
pub mod framework;
pub mod logs;
pub mod screens;
use core::panic;
use std::collections::HashMap;
use std::io::{self, ErrorKind};
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use chrono::{DateTime, Days, Utc};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use log::{debug, error, info};
use ratatui::Frame;
use tokio::sync::mpsc::{self, Sender};

use crate::cli::AppConfig;
use crate::network::client::Client;
use crate::tui::chat::{ChatMessage, ChatMessageStatus, DisplayChannel, MediaMessage, User};
use crate::tui::events::{ChannelId, TuiEvent, UserId};
use crate::tui::framework::{Tui, TuiRunner};
use crate::tui::logs::LogEntry;
use crate::tui::screens::chat::draw_main;
use crate::tui::screens::login::draw_login;

#[derive(Clone)]
pub struct UserProfile {
    user_id: UserId,
    username: String,
    password: String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ChatFocus {
    Channels,
    ChatHistory,
    ChatInput(usize),
    Users,
    Logs,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Screen {
    Chat(SocketAddr),
    Login,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ServerState {
    Connected,
    Unhealthy,
    Disconnected,
    Reconnecting,
}

#[derive(Clone)]
pub struct GlobalState {
    logs: Vec<LogEntry>,
    log_scroll_offset: usize,
    show_logs: bool,
}

#[derive(Clone)]
pub struct State {
    should_quit: bool,
    global_state: GlobalState,
    current_state: AppState,
    state_map: HashMap<Screen, AppState>,
}

#[derive(Clone)]
pub enum AppState {
    Chat(ChatState),
    Login(LoginState),
}

#[derive(Clone)]
pub struct ChatState {
    focus: ChatFocus,
    channels: Vec<DisplayChannel>,
    users: Vec<User>,
    chat_history: HashMap<ChannelId, Vec<ChatMessage>>,
    chat_input: String,
    active_channel_idx: usize,
    current_user: UserProfile,
    chat_scroll_offset: usize,
    last_healthcheck: DateTime<Utc>,
    server_address: SocketAddr,
    server_connection_state: ServerState,
}

#[derive(Debug, PartialEq, Clone)]
pub enum LoginFocus {
    UsernameInput(usize),
    PasswordInput(usize),
    ServerAddressInput(usize),
    Login,
    Nothing,
}

#[derive(Clone, PartialEq, Debug)]
pub enum InputStatus {
    AllFine,
    FailedToLogin, // Temp fix until server gets better
    UserNotFound,
    IncorrectPassword,
    IncorrectUsernameOrPassword,
    ServerNotFound,
    AddressNotParsable,
    UnknownError,
}

#[derive(Clone)]
pub struct LoginState {
    username_input: String,
    password_input: String,
    server_address_input: String,
    server_address: Option<SocketAddr>,
    focus: LoginFocus,
    input_status: InputStatus,
}

impl State {
    pub fn new() -> Self {
        let login_state = AppState::Login(LoginState {
            username_input: "penger ".to_owned(),
            password_input: "epicpass4 ".to_owned(),
            server_address_input: "0.0.0.0:4348 ".to_owned(),
            server_address: None,
            focus: LoginFocus::Nothing,
            input_status: InputStatus::AllFine,
        });

        State {
            should_quit: false,
            global_state: GlobalState {
                show_logs: false,
                log_scroll_offset: 0,
                logs: vec![],
            },
            current_state: login_state.clone(),
            state_map: HashMap::from([(Screen::Login, login_state)]),
        }
    }
}

#[async_trait]
impl Tui<TuiEvent> for State {
    /// Draws the UI layout and content.
    fn draw_ui(&mut self, frame: &mut Frame) {
        match &mut self.current_state {
            AppState::Chat(chat_state) => draw_main(&self.global_state, chat_state, frame),
            AppState::Login(login_state) => draw_login(&self.global_state, login_state, frame),
        }
    }

    fn process_event(&mut self, event: Event) -> Option<TuiEvent> {
        use KeyCode::*;

        match &mut self.current_state {
            AppState::Chat(chat_state) => match event {
                Event::Key(key_event) => match chat_state.focus {
                    ChatFocus::Channels => match key_event.code {
                        Up => Some(TuiEvent::ChannelUp),
                        Down => Some(TuiEvent::ChannelDown),
                        Right => Some(TuiEvent::ChatFocusChange(ChatFocus::ChatHistory)),
                        Char('q') | Char('Q') => Some(TuiEvent::Exit),
                        Char('l') | Char('L') => Some(TuiEvent::ToggleLogs),
                        Char(_) => Some(TuiEvent::ChatFocusChange(ChatFocus::ChatInput(0))),
                        _ => None,
                    },
                    ChatFocus::ChatHistory => match key_event.code {
                        Left => Some(TuiEvent::ChatFocusChange(ChatFocus::Channels)),
                        Right if self.global_state.show_logs => Some(TuiEvent::ChatFocusChange(ChatFocus::Logs)),
                        Right => Some(TuiEvent::ChatFocusChange(ChatFocus::Users)),
                        Up => Some(TuiEvent::ScrollUp),
                        Down => Some(TuiEvent::ScrollDown),
                        Char('q') | Char('Q') => Some(TuiEvent::Exit),
                        Char('l') | Char('L') => Some(TuiEvent::ToggleLogs),
                        Char(_) => Some(TuiEvent::ChatFocusChange(ChatFocus::ChatInput(0))),
                        _ => None,
                    },
                    ChatFocus::ChatInput(_) => match key_event.code {
                        Up => Some(TuiEvent::ChatFocusChange(ChatFocus::ChatHistory)),
                        Left if key_event.modifiers == KeyModifiers::CONTROL => Some(TuiEvent::InputLeftTab),
                        Right if key_event.modifiers == KeyModifiers::CONTROL => Some(TuiEvent::InputRightTab),
                        Left => Some(TuiEvent::InputLeft),
                        Right => Some(TuiEvent::InputRight),
                        Enter => Some(TuiEvent::InputEnter),
                        Char(chr) => Some(TuiEvent::InputChar(chr)),
                        Backspace => Some(TuiEvent::InputDelete),

                        _ => None,
                    },
                    ChatFocus::Users => match key_event.code {
                        Left if self.global_state.show_logs => Some(TuiEvent::ChatFocusChange(ChatFocus::Logs)),
                        Left => Some(TuiEvent::ChatFocusChange(ChatFocus::ChatHistory)),
                        Char('q') | Char('Q') => Some(TuiEvent::Exit),
                        Char('l') | Char('L') => Some(TuiEvent::ToggleLogs),
                        Char(_) => Some(TuiEvent::ChatFocusChange(ChatFocus::ChatInput(0))),
                        _ => None,
                    },
                    ChatFocus::Logs => match key_event.code {
                        Left => Some(TuiEvent::ChatFocusChange(ChatFocus::ChatHistory)),
                        Right => Some(TuiEvent::ChatFocusChange(ChatFocus::Users)),
                        Up => Some(TuiEvent::ScrollUp),
                        Down => Some(TuiEvent::ScrollDown),
                        Char('q') | Char('Q') => Some(TuiEvent::Exit),
                        Char('l') | Char('L') => Some(TuiEvent::ToggleLogs),
                        Char(_) => Some(TuiEvent::ChatFocusChange(ChatFocus::ChatInput(0))),

                        _ => None,
                    },
                },
                _ => None,
            },
            AppState::Login(login_state) => match event {
                Event::Key(key_event) => match login_state.focus {
                    LoginFocus::UsernameInput(idx) => match key_event.code {
                        Left if key_event.modifiers == KeyModifiers::CONTROL => Some(TuiEvent::InputLeftTab),
                        Left => Some(TuiEvent::InputLeft),
                        Right if key_event.modifiers == KeyModifiers::CONTROL => Some(TuiEvent::InputRightTab),
                        Right => Some(TuiEvent::InputRight),
                        Down | Tab | Enter => Some(TuiEvent::LoginFocusChange(LoginFocus::PasswordInput(idx))),
                        Backspace => Some(TuiEvent::InputDelete),
                        Esc => Some(TuiEvent::LoginFocusChange(LoginFocus::Nothing)),
                        Char('l') | Char('L') => Some(TuiEvent::ToggleLogs),
                        Char(chr) => Some(TuiEvent::InputChar(chr)),

                        _ => None,
                    },
                    LoginFocus::PasswordInput(idx) => match key_event.code {
                        Up | BackTab => Some(TuiEvent::LoginFocusChange(LoginFocus::UsernameInput(idx))),
                        Down | Tab | Enter => Some(TuiEvent::LoginFocusChange(LoginFocus::ServerAddressInput(idx))),
                        Left if key_event.modifiers == KeyModifiers::CONTROL => Some(TuiEvent::InputLeftTab),
                        Left => Some(TuiEvent::InputLeft),
                        Right if key_event.modifiers == KeyModifiers::CONTROL => Some(TuiEvent::InputRightTab),
                        Right => Some(TuiEvent::InputRight),
                        Backspace => Some(TuiEvent::InputDelete),
                        Esc => Some(TuiEvent::LoginFocusChange(LoginFocus::Nothing)),
                        Char('l') | Char('L') => Some(TuiEvent::ToggleLogs),
                        Char(chr) => Some(TuiEvent::InputChar(chr)),
                        _ => None,
                    },
                    LoginFocus::ServerAddressInput(idx) => match key_event.code {
                        Up | BackTab => Some(TuiEvent::LoginFocusChange(LoginFocus::PasswordInput(idx))),
                        Down | Tab | Enter => Some(TuiEvent::LoginFocusChange(LoginFocus::Login)),
                        Left if key_event.modifiers == KeyModifiers::CONTROL => Some(TuiEvent::InputLeftTab),
                        Left => Some(TuiEvent::InputLeft),
                        Right if key_event.modifiers == KeyModifiers::CONTROL => Some(TuiEvent::InputRightTab),
                        Right => Some(TuiEvent::InputRight),
                        Backspace => Some(TuiEvent::InputDelete),
                        Esc => Some(TuiEvent::LoginFocusChange(LoginFocus::Nothing)),
                        Enter => Some(TuiEvent::InputEnter),
                        Char('l') | Char('L') => Some(TuiEvent::ToggleLogs),
                        Char(chr) => Some(TuiEvent::InputChar(chr)),
                        _ => None,
                    },
                    LoginFocus::Login => match key_event.code {
                        Char('q') | Char('Q') => Some(TuiEvent::Exit),
                        Char('l') | Char('L') => Some(TuiEvent::ToggleLogs),
                        Up | BackTab => Some(TuiEvent::LoginFocusChange(LoginFocus::ServerAddressInput(0))),
                        Esc => Some(TuiEvent::LoginFocusChange(LoginFocus::Nothing)),
                        Enter => Some(TuiEvent::Login),
                        _ => None,
                    },
                    LoginFocus::Nothing => match key_event.code {
                        Char('q') | Char('Q') => Some(TuiEvent::Exit),
                        Char('l') | Char('L') => Some(TuiEvent::ToggleLogs),
                        Char(_) | Tab | Up | Down | Left | Right | Enter => Some(TuiEvent::LoginFocusChange(LoginFocus::UsernameInput(0))),
                        Enter => Some(TuiEvent::InputEnter),
                        _ => None,
                    },
                },
                _ => None,
            },
        }
    }

    async fn handle_event(&mut self, event: TuiEvent, event_send: &Sender<TuiEvent>, client: &mut Client) -> Result<()> {
        use TuiEvent::*;
        match &mut self.current_state {
            AppState::Chat(state) => {
                match event {
                    Exit => self.should_quit = true,
                    ToggleLogs => {
                        self.global_state.show_logs = !self.global_state.show_logs;
                        state.focus = ChatFocus::ChatHistory;
                    }
                    Log(entry) => self.global_state.logs.push(entry),
                    ChannelUp => {
                        if state.active_channel_idx == 0 {
                            state.active_channel_idx = state.channels.len().saturating_sub(1);
                        } else {
                            state.active_channel_idx -= 1;
                        }
                    }
                    ChannelDown => {
                        state.active_channel_idx = (state.active_channel_idx + 1) % state.channels.len();
                    }
                    ChatFocusChange(focus) => state.focus = focus,
                    InputLeft => {
                        if let ChatFocus::ChatInput(i) = state.focus
                            && i > 0
                        {
                            state.focus = ChatFocus::ChatInput(i - 1)
                        }
                    }
                    InputRight => {
                        if let ChatFocus::ChatInput(i) = state.focus
                            && i + 1 < state.chat_input.len()
                        {
                            state.focus = ChatFocus::ChatInput(i + 1)
                        }
                    }
                    InputLeftTab => {
                        if let ChatFocus::ChatInput(i) = state.focus
                            && i > 0
                        {
                            let idx = state
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
                                .unwrap_or(0);

                            state.focus = ChatFocus::ChatInput(idx)
                        }
                    }
                    InputRightTab => {
                        if let ChatFocus::ChatInput(i) = state.focus
                            && i + 1 < state.chat_input.len()
                        {
                            let idx = state
                                .chat_input
                                .char_indices()
                                .skip(i + 1)
                                .skip_while(|(_, c)| *c != ' ')
                                // .skip_while(|(_, c)| *c == ' ')
                                .map(|(idx, _)| idx)
                                .next()
                                .unwrap_or(state.chat_input.len());
                            state.focus = ChatFocus::ChatInput(idx)
                        }
                    }
                    InputDelete => {
                        if let ChatFocus::ChatInput(i) = state.focus
                            && i > 0
                        {
                            state.chat_input.remove(i - 1);
                            state.focus = ChatFocus::ChatInput(i - 1)
                        }
                    }

                    InputEnter if state.chat_input.len() > 1 => {
                        // command_send.send(Command::SendMessage(self.chat_input.clone())).await?;
                        let message = ChatMessage {
                            message_id: None,
                            author_name: state.current_user.username.to_owned(),
                            author_id: state.current_user.user_id,
                            reply_id: 0, // TODO replies
                            timestamp: Utc::now(),
                            message: state.chat_input.clone(),
                            status: ChatMessageStatus::Sending,
                        };
                        let channel_id = state.channels.get(state.active_channel_idx).unwrap().id; // TODO better

                        state.chat_history.entry(channel_id).and_modify(|log| log.push(message));

                        client.send_chat_message(channel_id, 0, state.chat_input.clone(), vec![]).await?; // TODO improve
                        state.focus = ChatFocus::ChatInput(0);
                        state.chat_input = " ".to_owned();
                    }
                    ScrollDown => match state.focus {
                        ChatFocus::ChatHistory => {
                            state.chat_scroll_offset = state.chat_scroll_offset.saturating_sub(1);
                        }
                        ChatFocus::Logs => {
                            self.global_state.log_scroll_offset = self.global_state.log_scroll_offset.saturating_sub(1);
                        }
                        _ => {}
                    },
                    ScrollUp => match state.focus {
                        ChatFocus::ChatHistory => {
                            state.chat_scroll_offset = state.chat_scroll_offset.saturating_add(1);
                        }
                        ChatFocus::Logs => {
                            self.global_state.log_scroll_offset = self.global_state.log_scroll_offset.saturating_add(1);
                        }
                        _ => {}
                    },
                    InputChar(chr) => {
                        if let ChatFocus::ChatInput(i) = state.focus {
                            state.chat_input.insert(i, chr);

                            state.focus = ChatFocus::ChatInput(i + 1)
                        }
                    }

                    ChannelIDs(channel_ids) => {
                        if !channel_ids.is_empty() {
                            debug!("received channel ids {channel_ids:?}");
                            client.request_channels(channel_ids).await?
                        }
                    }
                    HealthCheck => {
                        state.last_healthcheck = Utc::now();
                        client.request_user_statuses().await?;
                    }

                    Channels(channels) => {
                        debug!("received {channels:?}");
                        for channel in channels {
                            // I want to add the channel first and only then request
                            // if I requested first to make the borrow checker happy it could fail and end up in a broken state
                            // history would be incoming for a channel which is not added
                            let channel_id = channel.channel_id;

                            state.channels.push(channel.into());
                            client.request_history_by_timestamp(channel_id, Utc::now(), 50).await?;
                        }
                    }
                    UserStatusesUpdate(status_updates) => {
                        // TODO what happens if a new user comes online? We dont get their name
                        debug!("received statuses{status_updates:?}");

                        let mut users_not_found = vec![];
                        'outer: for status_update in status_updates {
                            for user in &mut state.users {
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
                        for user in &mut state.users {
                            if let Some(new_user) = new_users_map.remove(&user.id) {
                                user.status = new_user.status;
                            }
                        }
                        state.users.extend(new_users_map.into_values());
                    }
                    HistoryUpdate(messages) => {
                        for message in messages {
                            let author_name = state
                                .users
                                .iter()
                                .find(|user| user.id == message.user_id)
                                .map(|user| user.name.clone())
                                .unwrap_or_else(|| "Unknown".to_string());

                            let timestamp = DateTime::from_timestamp(message.sent_timestamp as i64, 0).ok_or_else(|| anyhow!("Invalid timestamp"))?;

                            let display_message = ChatMessage {
                                message_id: Some(message.message_id),
                                reply_id: message.message_id,
                                author_name,
                                author_id: message.user_id,
                                timestamp,
                                message: message.message_text,
                                status: ChatMessageStatus::Send,
                            };

                            let channel_id = message.channel_id;
                            // TODO figure out what to do when we get message from channels we dont know the name off
                            let display_messages = state.chat_history.entry(channel_id).or_default();

                            if !display_messages.iter().any(|m| m.message_id == display_message.message_id) {
                                debug!("inserting {display_message:?} into history of channel {channel_id}");
                                display_messages.push(display_message);
                            }
                        }
                    }
                    MessageSendAck(message_id) => {
                        // Never passes because local display messages do not have an id yet
                        if let Some(message) = state
                            .chat_history
                            .iter_mut()
                            .find_map(|(_, messages)| messages.iter_mut().find(|m| m.message_id == Some(message_id)))
                        {
                            // Update the message status
                            message.status = ChatMessageStatus::Send;
                        } else {
                            debug!("Message with id {message_id} not found in chat history");
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
                        state.server_connection_state = ServerState::Disconnected;
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
                    _ => {}
                }
            }
            AppState::Login(state) => match event {
                LoginFocusChange(focus) => state.focus = focus,
                InputChar(chr) => match state.focus {
                    LoginFocus::UsernameInput(i) if i < 129 => {
                        state.username_input.insert(i, chr);
                        state.focus = LoginFocus::UsernameInput(i + 1);
                        state.input_status = InputStatus::AllFine;
                    }
                    LoginFocus::PasswordInput(i) if i < 1025 => {
                        state.password_input.insert(i, chr);
                        state.focus = LoginFocus::PasswordInput(i + 1);
                        state.input_status = InputStatus::AllFine;
                    }
                    LoginFocus::ServerAddressInput(i) if i < 64 => {
                        state.server_address_input.insert(i, chr);
                        state.focus = LoginFocus::ServerAddressInput(i + 1);
                        state.input_status = InputStatus::AllFine;
                    }
                    _ => {}
                },
                InputDelete => match state.focus {
                    LoginFocus::UsernameInput(i) if i > 0 => {
                        state.username_input.remove(i - 1);
                        state.focus = LoginFocus::UsernameInput(i - 1);
                        state.input_status = InputStatus::AllFine;
                    }
                    LoginFocus::PasswordInput(i) if i > 0 => {
                        state.password_input.remove(i - 1);
                        state.focus = LoginFocus::PasswordInput(i - 1);
                        state.input_status = InputStatus::AllFine;
                    }
                    LoginFocus::ServerAddressInput(i) if i > 0 => {
                        state.server_address_input.remove(i - 1);
                        state.focus = LoginFocus::ServerAddressInput(i - 1);
                        state.input_status = InputStatus::AllFine;
                        state.input_status = InputStatus::AllFine;
                    }
                    _ => {}
                },
                InputLeft => match state.focus {
                    LoginFocus::UsernameInput(i) if i > 0 => state.focus = LoginFocus::UsernameInput(i - 1),
                    LoginFocus::PasswordInput(i) if i > 0 => state.focus = LoginFocus::PasswordInput(i - 1),
                    LoginFocus::ServerAddressInput(i) if i > 0 => state.focus = LoginFocus::ServerAddressInput(i - 1),
                    _ => {}
                },
                InputRight => match state.focus {
                    LoginFocus::UsernameInput(i) if i + 1 < state.username_input.len() => state.focus = LoginFocus::UsernameInput(i + 1),
                    LoginFocus::PasswordInput(i) if i + 1 < state.password_input.len() => state.focus = LoginFocus::PasswordInput(i + 1),
                    LoginFocus::ServerAddressInput(i) if i + 1 < state.server_address_input.len() => {
                        state.focus = LoginFocus::ServerAddressInput(i + 1)
                    }
                    _ => {}
                },
                InputLeftTab => match state.focus {
                    LoginFocus::UsernameInput(i) => state.focus = LoginFocus::UsernameInput(0),
                    LoginFocus::PasswordInput(i) => state.focus = LoginFocus::PasswordInput(0),
                    LoginFocus::ServerAddressInput(i) => state.focus = LoginFocus::ServerAddressInput(0),
                    _ => {}
                },
                InputRightTab => match state.focus {
                    LoginFocus::UsernameInput(i) => state.focus = LoginFocus::UsernameInput(state.username_input.len() - 1),
                    LoginFocus::PasswordInput(i) => state.focus = LoginFocus::PasswordInput(state.password_input.len() - 1),
                    LoginFocus::ServerAddressInput(i) => state.focus = LoginFocus::ServerAddressInput(state.server_address_input.len() - 1),
                    _ => {}
                },
                Login => {
                    if let Ok(server_address) = state.server_address_input.trim().parse::<SocketAddr>() {
                        match client.connect(server_address).await {
                            Ok(_) => {
                                client
                                    .login(state.username_input.trim().to_string(), state.password_input.clone().trim().to_string())
                                    .await?;
                                state.server_address = Some(server_address);
                            }
                            Err(e) => {
                                if let Some(err) = e.downcast_ref::<io::Error>()
                                    && err.kind() == ErrorKind::InvalidInput
                                {
                                    state.input_status = InputStatus::ServerNotFound;
                                } else {
                                    state.input_status = InputStatus::UnknownError;
                                }
                            }
                        }
                    } else {
                        state.input_status = InputStatus::AddressNotParsable
                    };
                }
                LoginSuccess(user_id) => {
                    if let Some(server_address) = state.server_address {
                        // Save login state
                        self.state_map.insert(Screen::Login, AppState::Login(state.clone()));
                        self.current_state = AppState::Chat(ChatState {
                            focus: ChatFocus::Channels,
                            channels: vec![],
                            users: vec![],
                            chat_history: HashMap::new(),
                            chat_input: " ".to_owned(),
                            active_channel_idx: 0,
                            current_user: UserProfile {
                                user_id,
                                username: state.username_input.clone(),
                                password: state.password_input.clone(),
                            },
                            chat_scroll_offset: 0,
                            last_healthcheck: Utc::now(),
                            server_connection_state: ServerState::Connected,
                            server_address,
                        });
                        client.request_channel_ids().await?;
                        client.request_user_statuses().await?;
                    } else {
                        panic!("Should be unreachable");
                    }
                }
                LoginFail(message) => {
                    match message.as_str() {
                        "Incorrect username or password." => state.input_status = InputStatus::IncorrectUsernameOrPassword,
                        _ => state.input_status = InputStatus::FailedToLogin,
                    }

                    client.disconnect(); // TODO make it work properly
                }
                ToggleLogs => {
                    self.global_state.show_logs = !self.global_state.show_logs;
                }
                Log(entry) => self.global_state.logs.push(entry),
                Exit => self.should_quit = true,
                _ => {}
            },
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

    let client = Client::new(event_send.clone());

    let username = config.username.clone();
    let password = config.password.clone();
    event_send.send(TuiEvent::ConnectAndLogin(config.address, username, password)).await?;

    let tasks = vec![async move {}];

    let tui = State::new();
    let tui_runner = TuiRunner::new(tui, client, event_recv, event_send, config.loglevel);

    tui_runner.run(tasks).await
}
