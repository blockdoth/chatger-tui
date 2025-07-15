use crossterm::event::{Event, KeyCode, KeyModifiers};

use crate::tui::LoginFocus;
use crate::tui::events::TuiEvent;

pub fn handle_login_key_event(event: Event, focus: LoginFocus) -> Option<TuiEvent> {
    use KeyCode::*;

    match event {
        Event::Key(key_event) => match focus {
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
                Char('l') | Char('L') => Some(TuiEvent::ToggleLogs),
                Char(chr) => Some(TuiEvent::InputChar(chr)),
                _ => None,
            },
            LoginFocus::Login => match key_event.code {
                Char('q') | Char('Q') => Some(TuiEvent::Exit),
                Char('l') | Char('L') => Some(TuiEvent::ToggleLogs),
                Up | BackTab => Some(TuiEvent::LoginFocusChange(LoginFocus::ServerAddressInput(0))),
                Esc => Some(TuiEvent::LoginFocusChange(LoginFocus::Nothing)),
                _ => None,
            },
            LoginFocus::Nothing => match key_event.code {
                Char('q') | Char('Q') => Some(TuiEvent::Exit),
                Char('l') | Char('L') => Some(TuiEvent::ToggleLogs),
                Char(_) | Tab | Up | Down | Left | Right | Enter => Some(TuiEvent::LoginFocusChange(LoginFocus::UsernameInput(0))),
                _ => None,
            },
        },
        _ => None,
    }
}
