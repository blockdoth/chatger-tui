pub mod chat;
pub mod framework;
pub mod logs;
pub mod ui;
use std::collections::HashMap;

use anyhow::{Ok, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crossterm::event::{Event, KeyCode};
use ratatui::Frame;
use tokio::sync::mpsc::{self, Sender};

use crate::tui::chat::{Channel, ChannelId, ChannelStatus, ChatMessage, User};
use crate::tui::framework::{FromLog, Tui, TuiRunner};
use crate::tui::logs::LogEntry;
use crate::tui::ui::draw;

#[derive(Debug)]
pub enum TuiEvent {
    Log(LogEntry),
    Exit,
}

impl FromLog for TuiEvent {
    fn from_log(log: logs::LogEntry) -> Self {
        TuiEvent::Log(log)
    }
}

#[derive(Debug)]
pub enum Focus {
    Channels,
    ChatHistory,
    Input,
    Users,
    None,
}

#[derive(Debug)]
pub enum Command {}

pub struct State {
    should_quit: bool,
    logs: Vec<LogEntry>,
    logs_scroll_offset: usize,
    channels: Vec<Channel>,
    users: Vec<User>,
    chat_history: HashMap<ChannelId, Vec<ChatMessage>>,
    chat_input: String,
    active_channel: ChannelId,
    focus: Focus,
}

impl State {
    pub fn new() -> Self {
        let mut chatlogs = HashMap::new();
        chatlogs.insert(0, vec![
              ChatMessage {
                id: 1,
                author: User {
                    id: 1,
                    name: "ballman 1".to_owned(),
                    status: chat::UserStatus::Online,
                },
                timestamp: Utc::now(),
                message: "balls".to_owned(),
            },
              ChatMessage {
                id: 2,
                author: User {
                    id: 2,
                    name: "ballman 2".to_owned(),
                    status: chat::UserStatus::Online,
                },
                timestamp: Utc::now(),
                message: "also balls".to_owned(),
            },
              ChatMessage {
                id: 3,
                author: User {
                    id: 3,
                    name: "ballman 3".to_owned(),
                    status: chat::UserStatus::Online,
                },
                timestamp: Utc::now(),
                message: "more balls".to_owned(),
            },
              ChatMessage {
                id: 4,
                author: User {
                    id: 4,
                    name: "ballman 4".to_owned(),
                    status: chat::UserStatus::Online,
                },
                timestamp: Utc::now(),
                message: "What the fuck did you just fucking say about me, you little bitch? I'll have you know I graduated top of my class in the Navy Seals, and I've been involved in numerous secret raids on Al-Quaeda, and I have over 300 confirmed kills. I am trained in gorilla warfare and I'm the top sniper in the entire ".to_owned(),
            }]);

        State {
            should_quit: false,
            logs_scroll_offset: 0,
            logs: vec![],
            active_channel: 0,
            focus: Focus::None,
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
            chat_input: "We're no strangers to love".to_owned(),
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
            Event::Key(key_event) => match key_event.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => Some(TuiEvent::Exit),
                _ => None,
            },
            _ => None,
        }
    }

    async fn handle_event(&mut self, event: TuiEvent, command_send: &Sender<Command>) -> Result<()> {
        match event {
            TuiEvent::Exit => self.should_quit = true,
            TuiEvent::Log(entry) => self.logs.push(entry),
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
