pub mod chat;
pub mod framework;
pub mod logs;
pub mod ui;
use anyhow::{Ok, Result};
use async_trait::async_trait;
use crossterm::event::{Event, KeyCode};
use ratatui::Frame;
use tokio::sync::mpsc::{self, Sender};

use crate::tui::chat::{Channel, Chat, User};
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
pub enum Command {}

pub struct State {
    should_quit: bool,
    logs: Vec<LogEntry>,
    logs_scroll_offset: usize,
    channels: Vec<Channel>,
    users: Vec<User>,
    chat_log: Vec<Chat>,
    chat_input: String,
}

impl State {
    pub fn new() -> Self {
        State {
            should_quit: false,
            logs_scroll_offset: 0,
            logs: vec![],
            channels: vec![],
            users: vec![],
            chat_log: vec![],
            chat_input: "".to_owned(),
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
