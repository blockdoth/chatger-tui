pub mod keys;
pub mod ui;

use std::collections::{HashMap, VecDeque};
use std::io::{self, ErrorKind};
use std::net::SocketAddr;

use anyhow::Result;
use log::{debug, error, info};
use tokio::sync::mpsc::Sender;
use tokio::time::Instant;

use crate::network::client::Client;
use crate::network::protocol::UserStatus;
use crate::tui::events::TuiEvent;
use crate::tui::screens::chat::{ChatFocus, ServerConnectionStatus, UserProfile};
use crate::tui::{AppState, ChatState, Screen, State};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LoginFocus {
    UsernameInput(usize),
    PasswordInput(usize),
    ServerAddressInput(usize),
    LoginButton,
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

#[derive(Clone, Debug)]
pub struct LoginState {
    pub username_input: String,
    pub password_input: String,
    pub server_address_input: String,
    pub server_address: Option<SocketAddr>,
    pub focus: LoginFocus,
    pub input_status: InputStatus,
}

pub async fn handle_login_event(tui: &mut State, event: TuiEvent, client: &mut Client) -> Result<()> {
    let login_state = match &mut tui.current_state {
        AppState::Login(login_state) => login_state,
        _ => panic!("This function only handles the chat state"),
    };

    use TuiEvent::*;
    match event {
        LoginFocusChange(focus) => login_state.focus = focus,
        InputChar(chr) => match login_state.focus {
            LoginFocus::UsernameInput(i) if i < 129 => {
                login_state.username_input.insert(i, chr);
                login_state.focus = LoginFocus::UsernameInput(i + 1);
                login_state.input_status = InputStatus::AllFine;
            }
            LoginFocus::PasswordInput(i) if i < 1025 => {
                login_state.password_input.insert(i, chr);
                login_state.focus = LoginFocus::PasswordInput(i + 1);
                login_state.input_status = InputStatus::AllFine;
            }
            LoginFocus::ServerAddressInput(i) if i < 64 => {
                login_state.server_address_input.insert(i, chr);
                login_state.focus = LoginFocus::ServerAddressInput(i + 1);
                login_state.input_status = InputStatus::AllFine;
            }
            _ => {}
        },
        InputDelete => match login_state.focus {
            LoginFocus::UsernameInput(i) if i > 0 => {
                login_state.username_input.remove(i - 1);
                login_state.focus = LoginFocus::UsernameInput(i - 1);
                login_state.input_status = InputStatus::AllFine;
            }
            LoginFocus::PasswordInput(i) if i > 0 => {
                login_state.password_input.remove(i - 1);
                login_state.focus = LoginFocus::PasswordInput(i - 1);
                login_state.input_status = InputStatus::AllFine;
            }
            LoginFocus::ServerAddressInput(i) if i > 0 => {
                login_state.server_address_input.remove(i - 1);
                login_state.focus = LoginFocus::ServerAddressInput(i - 1);
                login_state.input_status = InputStatus::AllFine;
                login_state.input_status = InputStatus::AllFine;
            }
            _ => {}
        },
        InputLeft => match login_state.focus {
            LoginFocus::UsernameInput(i) if i > 0 => login_state.focus = LoginFocus::UsernameInput(i - 1),
            LoginFocus::PasswordInput(i) if i > 0 => login_state.focus = LoginFocus::PasswordInput(i - 1),
            LoginFocus::ServerAddressInput(i) if i > 0 => login_state.focus = LoginFocus::ServerAddressInput(i - 1),
            _ => {}
        },
        InputRight => match login_state.focus {
            LoginFocus::UsernameInput(i) if i < login_state.username_input.len() => login_state.focus = LoginFocus::UsernameInput(i + 1),
            LoginFocus::PasswordInput(i) if i < login_state.password_input.len() => login_state.focus = LoginFocus::PasswordInput(i + 1),
            LoginFocus::ServerAddressInput(i) if i < login_state.server_address_input.len() => {
                login_state.focus = LoginFocus::ServerAddressInput(i + 1)
            }
            _ => {}
        },
        InputLeftTab => match login_state.focus {
            LoginFocus::UsernameInput(_) => login_state.focus = LoginFocus::UsernameInput(0),
            LoginFocus::PasswordInput(_) => login_state.focus = LoginFocus::PasswordInput(0),
            LoginFocus::ServerAddressInput(_) => login_state.focus = LoginFocus::ServerAddressInput(0),
            _ => {}
        },
        InputRightTab => match login_state.focus {
            LoginFocus::UsernameInput(_) => login_state.focus = LoginFocus::UsernameInput(login_state.username_input.len()),
            LoginFocus::PasswordInput(_) => login_state.focus = LoginFocus::PasswordInput(login_state.password_input.len()),
            LoginFocus::ServerAddressInput(_) => login_state.focus = LoginFocus::ServerAddressInput(login_state.server_address_input.len()),
            _ => {}
        },
        Login => {
            if let Ok(server_address) = login_state.server_address_input.trim().parse::<SocketAddr>() {
                match client.connect(server_address).await {
                    Ok(_) => {
                        client
                            .login(login_state.username_input.clone(), login_state.password_input.clone())
                            .await?;
                        login_state.server_address = Some(server_address);
                        client.push_user_status(UserStatus::Online).await?;
                    }
                    Err(e) => {
                        if let Some(err) = e.downcast_ref::<io::Error>() {
                            match err.kind() {
                                ErrorKind::InvalidInput => login_state.input_status = InputStatus::ServerNotFound,
                                ErrorKind::ConnectionRefused => login_state.input_status = InputStatus::ServerNotFound,
                                e => {
                                    error!("Unhandled connection exception {e}");
                                    login_state.input_status = InputStatus::UnknownError
                                }
                            }
                        }
                    }
                }
            } else {
                login_state.input_status = InputStatus::AddressNotParsable
            };
        }
        LoginSuccess(user_id) => {
            if let Some(server_address) = login_state.server_address {
                // Save login state
                login_state.input_status = InputStatus::AllFine;
                tui.state_map.insert(Screen::Login, AppState::Login(login_state.clone()));

                let username = login_state.username_input.clone();
                let password = login_state.password_input.clone();

                debug!("{:?} {} {} {}", tui.state_map.keys(), username, password, server_address);
                if let Some(chat_state) = tui.state_map.get(&Screen::Chat(username, password, server_address.to_string())) {
                    tui.current_state = chat_state.clone();
                    info!("Restored a saved session");
                } else {
                    client.request_channel_ids().await?;
                    client.request_user_statuses().await?;
                    tui.current_state = AppState::Chat(Box::new(ChatState {
                        focus: ChatFocus::Channels,
                        channels: vec![],
                        users: vec![],
                        chat_history: HashMap::new(),
                        chat_inputs: HashMap::new(),
                        active_channel_idx: 0,
                        current_user: UserProfile {
                            user_id,
                            username: login_state.username_input.clone(),
                            password: login_state.password_input.clone(),
                        },
                        chat_scroll_offset: 0,
                        server_connection_status: ServerConnectionStatus::Connected,
                        server_address,
                        waiting_message_acks_id: VecDeque::new(),
                        incrementing_ack_id: 100000, // TODO better value
                        users_typing: HashMap::new(),
                        is_typing: false,
                        time_since_last_typing: Instant::now(),
                    }));
                };
            } else {
                panic!("Should be unreachable");
            }
        }
        LoginFail(message) => {
            match message.as_str() {
                "Incorrect username or password." => login_state.input_status = InputStatus::IncorrectUsernameOrPassword,
                _ => login_state.input_status = InputStatus::FailedToLogin,
            }

            client.disconnect()?; // TODO make it work properly
        }
        ToggleLogs => {
            tui.global_state.show_logs = !tui.global_state.show_logs;
        }
        Log(entry) => tui.global_state.logs.push(entry),
        Exit => tui.global_state.should_quit = true,
        _ => {}
    }
    Ok(())
}
