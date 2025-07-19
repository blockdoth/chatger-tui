pub mod chat;
pub mod login;
use std::collections::HashMap;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use ratatui::Frame;
use ratatui::crossterm::event::Event;
use tokio::sync::mpsc::{self, Sender};

use crate::cli::AppConfig;
use crate::network::client::{Client, ServerConnectionStatus};
use crate::tui::events::TuiEvent;
use crate::tui::framework::{Tui, TuiRunner};
use crate::tui::logs::LogEntry;
use crate::tui::screens::chat::keys::handle_chat_key_event;
use crate::tui::screens::chat::ui::draw_main;
use crate::tui::screens::chat::{ChatState, handle_chat_event};
use crate::tui::screens::login::keys::handle_login_key_event;
use crate::tui::screens::login::ui::draw_login;
use crate::tui::screens::login::{InputStatus, LoginFocus, LoginState, handle_login_event};

const USER_TIME_UNTIL_IDLE: u64 = 60;

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
            AppState::Chat(chat_state) => handle_chat_key_event(event, chat_state.focus, &self.global_state),
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

            if let Some(time) = state.time_since_last_focused
                && time.elapsed() > Duration::from_secs(USER_TIME_UNTIL_IDLE)
            {
                event_send.send(TuiEvent::IdleUser).await?;
                state.time_since_last_focused = None;
            }
        }

        Ok(())
    }

    fn should_quit(&self) -> bool {
        self.global_state.should_quit
    }
}
