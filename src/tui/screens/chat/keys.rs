use log::info;
use ratatui::crossterm::event::{Event, KeyCode, KeyModifiers};

use crate::tui::events::TuiEvent;
use crate::tui::screens::GlobalState;
use crate::tui::screens::chat::ChatFocus;

pub fn handle_chat_key_event(event: Event, focus: ChatFocus, global_state: &GlobalState) -> Option<TuiEvent> {
    use KeyCode::*;
    match event {
        Event::Key(key_event) => match focus {
            ChatFocus::Channels => match key_event.code {
                Up => Some(TuiEvent::ChannelUp),
                Down => Some(TuiEvent::ChannelDown),
                Right | Enter => Some(TuiEvent::ChatFocusChange(ChatFocus::ChatHistory)),
                Char('q') | Char('Q') => Some(TuiEvent::Exit),
                Char('l') | Char('L') => Some(TuiEvent::ToggleLogs),
                Char('x') | Char('X') => Some(TuiEvent::Logout),
                _ => None,
            },
            ChatFocus::ChatHistory => match key_event.code {
                Left => Some(TuiEvent::ChatFocusChange(ChatFocus::Channels)),
                Right if global_state.show_logs => Some(TuiEvent::ChatFocusChange(ChatFocus::Logs)),
                Right => Some(TuiEvent::ChatFocusChange(ChatFocus::Users(0))),
                Up => Some(TuiEvent::ScrollUp),
                Down => Some(TuiEvent::ScrollDown),
                Char('s') | Char('S') => Some(TuiEvent::ChatFocusChange(ChatFocus::ChatHistorySelection)),
                Char('q') | Char('Q') => Some(TuiEvent::Exit),
                Char('l') | Char('L') => Some(TuiEvent::ToggleLogs),
                Char('x') | Char('X') => Some(TuiEvent::Logout),
                Char(_) | Enter => Some(TuiEvent::ChatFocusChange(ChatFocus::ChatInput(0))),
                _ => None,
            },
            ChatFocus::ChatHistorySelection => match key_event.code {
                Left => Some(TuiEvent::ChatFocusChange(ChatFocus::Channels)),
                Right if global_state.show_logs => Some(TuiEvent::ChatFocusChange(ChatFocus::Logs)),
                Right => Some(TuiEvent::ChatFocusChange(ChatFocus::Users(0))),
                Up => Some(TuiEvent::ScrollUp),
                Down => Some(TuiEvent::ScrollDown),
                Char('s') | Char('S') | Esc => Some(TuiEvent::ChatFocusChange(ChatFocus::ChatHistory)),
                Char('r') | Char('R') => Some(TuiEvent::Reply),
                Char('q') | Char('Q') => Some(TuiEvent::Exit),
                Char('l') | Char('L') => Some(TuiEvent::ToggleLogs),
                Char('x') | Char('X') => Some(TuiEvent::Logout),
                Char(_) | Enter => Some(TuiEvent::ChatFocusChange(ChatFocus::ChatInput(0))),
                _ => None,
            },
            ChatFocus::ChatInput(_) => match key_event.code {
                Up | Esc => Some(TuiEvent::ChatFocusChange(ChatFocus::ChatHistory)),
                Left if key_event.modifiers == KeyModifiers::CONTROL => Some(TuiEvent::InputLeftTab),
                Right if key_event.modifiers == KeyModifiers::CONTROL => Some(TuiEvent::InputRightTab),
                Left => Some(TuiEvent::InputLeft),
                Right => Some(TuiEvent::InputRight),
                Enter => Some(TuiEvent::MessageSend),
                Char(chr) => Some(TuiEvent::InputChar(chr)),
                Backspace => Some(TuiEvent::InputDelete),

                _ => None,
            },
            ChatFocus::Users(_) => match key_event.code {
                Left if global_state.show_logs => Some(TuiEvent::ChatFocusChange(ChatFocus::Logs)),
                Left => Some(TuiEvent::ChatFocusChange(ChatFocus::ChatHistory)),
                Up => Some(TuiEvent::ScrollUp),
                Down => Some(TuiEvent::ScrollDown),
                Char('v') | Char('V') => Some(TuiEvent::ViewUsers),
                Char('q') | Char('Q') => Some(TuiEvent::Exit),
                Char('l') | Char('L') => Some(TuiEvent::ToggleLogs),
                Char(_) => Some(TuiEvent::ChatFocusChange(ChatFocus::ChatInput(0))),
                _ => None,
            },
            ChatFocus::Logs => match key_event.code {
                Left => Some(TuiEvent::ChatFocusChange(ChatFocus::ChatHistory)),
                Right => Some(TuiEvent::ChatFocusChange(ChatFocus::Users(0))),
                Up => Some(TuiEvent::ScrollUp),
                Down => Some(TuiEvent::ScrollDown),
                Char('q') | Char('Q') => Some(TuiEvent::Exit),
                Char('l') | Char('L') => Some(TuiEvent::ToggleLogs),
                Char('x') | Char('X') => Some(TuiEvent::Logout),
                Char(_) => Some(TuiEvent::ChatFocusChange(ChatFocus::ChatInput(0))),

                _ => None,
            },
        },
        Event::FocusLost => Some(TuiEvent::FocusLost),
        Event::FocusGained => Some(TuiEvent::FocusGained),
        _ => None,
    }
}
