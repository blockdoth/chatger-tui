use ratatui::crossterm::event::{Event, KeyCode, KeyModifiers};

use crate::tui::events::TuiEvent;
use crate::tui::screens::login::LoginFocus;

pub fn handle_login_key_event(event: Event, focus: LoginFocus) -> Option<TuiEvent> {
    use KeyCode::*;
    use LoginFocus::*;

    match event {
        Event::Key(key_event) => match focus {
            UsernameInput(idx) => match key_event.code {
                Left if key_event.modifiers == KeyModifiers::CONTROL => Some(TuiEvent::InputLeftTab),
                Left => Some(TuiEvent::InputLeft),
                Right if key_event.modifiers == KeyModifiers::CONTROL => Some(TuiEvent::InputRightTab),
                Right => Some(TuiEvent::InputRight),
                Down | Tab | Enter => Some(TuiEvent::LoginFocusChange(LoginFocus::PasswordInput(idx))),
                Backspace => Some(TuiEvent::InputDelete),
                Esc => Some(TuiEvent::LoginFocusChange(LoginFocus::Nothing)),
                Char(chr) => Some(TuiEvent::InputChar(chr)),

                _ => None,
            },
            PasswordInput(idx) => match key_event.code {
                Up | BackTab => Some(TuiEvent::LoginFocusChange(LoginFocus::UsernameInput(idx))),
                Down | Tab | Enter => Some(TuiEvent::LoginFocusChange(LoginFocus::ServerAddressInput(idx))),
                Left if key_event.modifiers == KeyModifiers::CONTROL => Some(TuiEvent::InputLeftTab),
                Left => Some(TuiEvent::InputLeft),
                Right if key_event.modifiers == KeyModifiers::CONTROL => Some(TuiEvent::InputRightTab),
                Right => Some(TuiEvent::InputRight),
                Backspace => Some(TuiEvent::InputDelete),
                Esc => Some(TuiEvent::LoginFocusChange(LoginFocus::Nothing)),
                Char(chr) => Some(TuiEvent::InputChar(chr)),
                _ => None,
            },
            ServerAddressInput(idx) => match key_event.code {
                Up | BackTab => Some(TuiEvent::LoginFocusChange(LoginFocus::PasswordInput(idx))),
                Down | Tab | Enter => Some(TuiEvent::LoginFocusChange(LoginFocus::LoginButton)),
                Left if key_event.modifiers == KeyModifiers::CONTROL => Some(TuiEvent::InputLeftTab),
                Left => Some(TuiEvent::InputLeft),
                Right if key_event.modifiers == KeyModifiers::CONTROL => Some(TuiEvent::InputRightTab),
                Right => Some(TuiEvent::InputRight),
                Backspace => Some(TuiEvent::InputDelete),
                Esc => Some(TuiEvent::LoginFocusChange(LoginFocus::Nothing)),
                Char(chr) => Some(TuiEvent::InputChar(chr)),
                _ => None,
            },
            LoginButton => match key_event.code {
                Char('q') | Char('Q') => Some(TuiEvent::Exit),
                Char('l') | Char('L') => Some(TuiEvent::ToggleLogs),
                Up | BackTab => Some(TuiEvent::LoginFocusChange(LoginFocus::ServerAddressInput(0))),
                Esc => Some(TuiEvent::LoginFocusChange(LoginFocus::Nothing)),
                Enter => Some(TuiEvent::Login),
                _ => None,
            },
            Nothing => match key_event.code {
                Char('q') | Char('Q') => Some(TuiEvent::Exit),
                Char('l') | Char('L') => Some(TuiEvent::ToggleLogs),
                Char(_) | Tab | Up | Down | Left | Right | Enter => Some(TuiEvent::LoginFocusChange(LoginFocus::UsernameInput(0))),
                _ => None,
            },
        },
        _ => None,
    }
}
