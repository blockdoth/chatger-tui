pub mod chat;
pub mod events;
pub mod framework;
pub mod logs;
pub mod ui;
use std::collections::HashMap;

use anyhow::{Ok, Result};
use async_trait::async_trait;
use chrono::{DateTime, Days, Duration, Utc};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use log::info;
use ratatui::Frame;
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::watch::error;

use crate::cli::{AppConfig, CliArgs};
use crate::network::client::{self, Client};
use crate::tui::chat::{ChannelId, ChannelStatus, ChatMessage, ChatMessageStatus, CurrentUser, DisplayChannel, User};
use crate::tui::events::TuiEvent;
use crate::tui::framework::{FromLog, Tui, TuiRunner};
use crate::tui::logs::LogEntry;
use crate::tui::ui::draw;

pub struct UserProfile {
    id: u64,
    name: String,
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
}

impl State {
    pub fn new() -> Self {
        let mut chatlogs = HashMap::new();
        chatlogs.insert(0, vec![
              ChatMessage {
                id: 1,
                author_id: 1,
                author_name: "ballman 1".to_owned(),
                timestamp: Utc::now(),
                message: "balls".to_owned(),
                status: ChatMessageStatus::Send,
              },
              ChatMessage {
                id: 2,
                author_id: 2,
                author_name: "ballman 2".to_owned(),
                timestamp: Utc::now(),
                message: "also balls".to_owned(),
                status: ChatMessageStatus::Send,
              },
              ChatMessage {
                id: 3,
                author_id: 3,
                author_name: "ballman 3".to_owned(),
                timestamp: Utc::now(),
                message: "more balls".to_owned(),
                status: ChatMessageStatus::FailedToSend,
              },
              ChatMessage {
                id: 4,
                author_id: 4,
                author_name: "ballman 4".to_owned(),
                timestamp: Utc::now(),
                message: "What the fuck did you just fucking say about me, you little bitch? I'll have you know I graduated top of my class in the Navy Seals, and I've been involved in numerous secret raids on Al-Quaeda, and I have over 300 confirmed kills. I am trained in gorilla warfare and I'm the top sniper in the entire ".to_owned(),
                status: ChatMessageStatus::Sending,
            }]);

        State {
            should_quit: false,
            last_healthcheck: Utc::now().checked_sub_days(Days::new(1)).unwrap(),
            show_logs: true,
            log_scroll_offset: 0,
            chat_scroll_offset: 0,
            logs: vec![],
            active_channel_idx: 0,
            focus: Focus::Channels,
            current_user: None,
            channels: vec![
                DisplayChannel {
                    id: 0,
                    name: "Balls".to_owned(),
                    status: ChannelStatus::Read,
                },
                DisplayChannel {
                    id: 1,
                    name: "penger pics".to_owned(),
                    status: ChannelStatus::Unread,
                },
                DisplayChannel {
                    id: 2,
                    name: "capi".to_owned(),
                    status: ChannelStatus::Muted,
                },
            ],
            users: vec![
                User {
                    id: 1,
                    name: "ballman 1".to_owned(),
                    status: chat::UserStatus::Online,
                },
                User {
                    id: 2,
                    name: "ballman 2".to_owned(),
                    status: chat::UserStatus::Offline,
                },
                User {
                    id: 3,
                    name: "ballman 3".to_owned(),
                    status: chat::UserStatus::Online,
                },
            ],
            chat_history: chatlogs,
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
                        id: 0,
                        author_name: user.name.to_owned(),
                        author_id: user.id,
                        timestamp: Utc::now(),
                        message: self.chat_input.clone(),
                        status: ChatMessageStatus::Sending,
                    };
                    self.chat_history
                        .entry(self.channels.get(self.active_channel_idx).unwrap().id)
                        .and_modify(|log| log.push(message));

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
            TuiEvent::SetUserNamePassword(name, password) => {
                self.current_user = Some(UserProfile {
                    id: 0,
                    name,
                    password,
                    is_logged_in: false,
                })
            }
            TuiEvent::ConnectAndLogin(address, username, password) => {
                client.connect(address).await?;
                client.login(username.clone(), password.clone()).await?;
                event_send.send(TuiEvent::SetUserNamePassword(username, password)).await;
            }
            TuiEvent::LoggedIn => {
                if let Some(user) = &mut self.current_user {
                    user.is_logged_in = true;
                    client.request_channel_ids().await?;
                }
            }
            TuiEvent::ChannelIDs(channel_ids) => {
                if !channel_ids.is_empty() {
                    info!("received channel ids {channel_ids:?}");
                    client.request_channels(channel_ids).await?
                }
            }
            TuiEvent::HealthCheck => self.last_healthcheck = Utc::now(),
            TuiEvent::Channels(channels) => {
                info!("{channels:?}");
                for channel in channels {
                    self.channels.push(channel.into());
                }
            }
            TuiEvent::ChannelIDs(channel_ids) => {
                info!("{channel_ids:?}")
            }
            TuiEvent::Disconnected => {
                info!("TODO reconnect logic");
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

    let tui = State::new();
    let tui_runner = TuiRunner::new(tui, client, event_recv, event_send, config.loglevel);

    tui_runner.run().await
}
