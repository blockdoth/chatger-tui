use anyhow::Result;
use tokio::sync::mpsc;

use crate::cli::AppConfig;
use crate::network::client::Client;
use crate::tui::events::TuiEvent;
use crate::tui::framework::TuiRunner;
use crate::tui::screens::login::{InputStatus, LoginFocus, LoginState};
use crate::tui::screens::{AppState, State};
pub mod chat;
pub mod events;
pub mod framework;
pub mod logs;
pub mod screens;

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
