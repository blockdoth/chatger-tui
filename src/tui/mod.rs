pub mod chat;
pub mod framework;
pub mod logs;
pub mod ui;
use std::collections::HashMap;

use anyhow::{Ok, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::Frame;
use tokio::sync::mpsc::{self, Sender};

use crate::tui::chat::{Channel, ChannelId, ChannelStatus, ChatMessage, ChatMessageStatus, CurrentUser, User};
use crate::tui::framework::{FromLog, Tui, TuiRunner};
use crate::tui::logs::LogEntry;
use crate::tui::ui::draw;

#[derive(Debug)]
pub enum TuiEvent {
    Log(LogEntry),
    Exit,
    ChannelUp,
    ChannelDown,
    FocusChange(Focus),
    InputRight,
    InputRightTab,
    InputLeft,
    InputLeftTab,
    InputChar(char),
    InputDelete,
    InputEnter,
}

impl FromLog for TuiEvent {
    fn from_log(log: logs::LogEntry) -> Self {
        TuiEvent::Log(log)
    }
}

#[derive(Debug, PartialEq)]
pub enum Focus {
    Channels,
    ChatHistory,
    ChatInput(usize),
    Users,
}

#[derive(Debug)]
pub enum Command {
    SendMessage(String),
}

pub struct State {
    should_quit: bool,
    logs: Vec<LogEntry>,
    logs_scroll_offset: usize,
    channels: Vec<Channel>,
    users: Vec<User>,
    chat_history: HashMap<ChannelId, Vec<ChatMessage>>,
    chat_input: String,
    active_channel_idx: usize,
    focus: Focus,
    current_user: CurrentUser,
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
            logs_scroll_offset: 0,
            logs: vec![],
            active_channel_idx: 0,
            focus: Focus::Channels,
            current_user: CurrentUser {
                id: 0,
                name: "blockdoth".to_owned(),
            },
            channels: vec![
                Channel {
                    id: 0,
                    name: "Balls".to_owned(),
                    status: ChannelStatus::Read,
                },
                Channel {
                    id: 1,
                    name: "penger pics".to_owned(),
                    status: ChannelStatus::Unread,
                },
                Channel {
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
impl Tui<TuiEvent, Command> for State {
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
                    _ => None,
                },
                Focus::ChatHistory => match key_event.code {
                    KeyCode::Left => Some(TuiEvent::FocusChange(Focus::Channels)),
                    KeyCode::Right => Some(TuiEvent::FocusChange(Focus::Users)),
                    KeyCode::Down | KeyCode::Enter => Some(TuiEvent::FocusChange(Focus::ChatInput(0))),
                    KeyCode::Char('q') | KeyCode::Char('Q') => Some(TuiEvent::Exit),
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
                    KeyCode::Left => Some(TuiEvent::FocusChange(Focus::ChatHistory)),
                    KeyCode::Char('q') | KeyCode::Char('Q') => Some(TuiEvent::Exit),
                    _ => None,
                },
            },
            _ => None,
        }
    }

    async fn handle_event(&mut self, event: TuiEvent, command_send: &Sender<Command>) -> Result<()> {
        match event {
            TuiEvent::Exit => __self.should_quit = true,
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
                command_send.send(Command::SendMessage(self.chat_input.clone())).await?;

                let message = ChatMessage {
                    id: 0,
                    author_name: self.current_user.name.to_owned(),
                    author_id: self.current_user.id,
                    timestamp: Utc::now(),
                    message: self.chat_input.clone(),
                    status: ChatMessageStatus::Sending,
                };

                self.chat_history
                    .entry(self.channels.get(self.active_channel_idx).unwrap().id)
                    .and_modify(|log| log.push(message));

                self.focus = Focus::ChatInput(0);
                self.chat_input = " ".to_owned();
            }
            TuiEvent::InputChar(chr) => {
                if let Focus::ChatInput(i) = self.focus {
                    self.chat_input.insert(i, chr);

                    self.focus = Focus::ChatInput(i + 1)
                }
            }
            _ => {}
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

pub async fn run() -> Result<()> {
    let (command_send, command_recv) = mpsc::channel::<Command>(10);
    let (even_send, even_recv) = mpsc::channel::<TuiEvent>(10);

    let update_send_clone = even_send.clone();

    // Showcases messages going both ways
    let other_task = vec![async move {
        // loop {
        //     update_send_clone.send(Update::Foo).await.unwrap();
        //     if command_recv.try_recv().is_ok() {
        //         warn!("Foo has been touched");
        //     }
        // }
    }];

    let tui = State::new();
    let tui_runner = TuiRunner::new(tui, command_send, even_recv, even_send, log::LevelFilter::Info);

    tui_runner.run(other_task).await
}
