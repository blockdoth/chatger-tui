pub mod chat;
pub mod events;
pub mod framework;
pub mod logs;
pub mod screens;

use std::collections::HashMap;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use ratatui::Frame;
use ratatui::crossterm::event::Event;
use tokio::sync::mpsc::{self, Sender};

use crate::cli::AppConfig;
use crate::network::client::Client;
use crate::tui::events::TuiEvent;
use crate::tui::framework::{Tui, TuiRunner};
use crate::tui::logs::LogEntry;
use crate::tui::screens::chat::keys::handle_chat_key_event;
use crate::tui::screens::chat::ui::draw_main;
use crate::tui::screens::chat::{ChatState, ServerConnectionStatus, handle_chat_event};
use crate::tui::screens::login::keys::handle_login_key_event;
use crate::tui::screens::login::ui::draw_login;
use crate::tui::screens::login::{InputStatus, LoginFocus, LoginState, handle_login_event};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Screen {
    Chat(String, String, String),
    Login,
}

#[derive(Clone, Debug)]
pub enum AppState {
    Chat(Box<ChatState>),
    Login(LoginState),
}

#[derive(Clone)]
pub struct GlobalState {
    logs: Vec<LogEntry>,
    log_scroll_offset: usize,
    show_logs: bool,
    should_quit: bool,
}

#[derive(Clone)]
pub struct State {
    global_state: GlobalState,
    current_state: AppState,
    state_map: HashMap<Screen, AppState>,
}

impl State {
    pub fn new(initial_state: AppState) -> Self {
        State {
            global_state: GlobalState {
                should_quit: false,
                show_logs: false,
                log_scroll_offset: 0,
                logs: vec![],
            },
            current_state: initial_state.clone(),
            state_map: HashMap::new(),
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
        match &mut self.current_state {
            AppState::Login(login_state) => handle_login_key_event(event, login_state.focus),
            AppState::Chat(chat_state) => handle_chat_key_event(&self.global_state, event, chat_state.focus),
        }
    }

    async fn handle_event(&mut self, event: TuiEvent, client: &mut Client) -> Result<()> {
        match &mut self.current_state {
            AppState::Chat(_) => handle_chat_event(self, event, client).await,
            AppState::Login(_) => handle_login_event(self, event, client).await,
        }
    }

    async fn on_tick(&mut self, event_send: &Sender<TuiEvent>, client: &mut Client) -> Result<()> {
        if let AppState::Chat(state) = &mut self.current_state {
            if state.is_typing && state.time_since_last_typing.elapsed() > Duration::from_secs(2) {
                event_send.send(TuiEvent::TypingExpired).await?;
            }
            let connection_elapsed = client.time_since_last_transmit.elapsed();
            if connection_elapsed > Duration::from_secs(10) && client.connection_status == ServerConnectionStatus::Connected {
                event_send.send(TuiEvent::PossiblyUnhealthyConnection).await?;
            }
            if (connection_elapsed > Duration::from_secs(15)
                || client.connection_status == ServerConnectionStatus::Disconnected
                || client.connection_status == ServerConnectionStatus::Reconnecting)
                && client.time_since_last_reconnect.elapsed() > Duration::from_secs(5)
            {
                client.time_since_last_reconnect.update();
                event_send.send(TuiEvent::Reconnect).await?;
            }
        }

        Ok(())
    }

    fn should_quit(&self) -> bool {
        self.global_state.should_quit
    }
}

pub async fn run(config: AppConfig) -> Result<()> {
    let (event_send, event_recv) = mpsc::channel::<TuiEvent>(10);

    let client = Client::new(event_send.clone());

    let tasks = vec![async move {}];

    let login_state = AppState::Login(LoginState {
        username_input: config.username,
        password_input: config.password,
        server_address_input: config.address.to_string(),
        server_address: None,
        focus: LoginFocus::Nothing,
        input_status: InputStatus::AllFine,
    });

    let tui = State::new(login_state);

    if config.auto_login {
        event_send.send(TuiEvent::Login).await?;
    }
    let tui_runner = TuiRunner::new(tui, client, event_recv, event_send, config.loglevel);

    tui_runner.run(tasks).await
}
