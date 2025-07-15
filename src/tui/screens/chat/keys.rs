use crossterm::event::{Event, KeyCode, KeyModifiers};

use crate::tui::events::TuiEvent;
use crate::tui::screens::chat::state::ChatFocus;
use crate::tui::{GlobalState};

pub fn handle_chat_key_event(global_state: &GlobalState, event: Event, focus: ChatFocus) -> Option<TuiEvent> {
    use KeyCode::*;
    match event {
        Event::Key(key_event) => match focus {
            ChatFocus::Channels => match key_event.code {
                Up => Some(TuiEvent::ChannelUp),
                Down => Some(TuiEvent::ChannelDown),
                Right => Some(TuiEvent::ChatFocusChange(ChatFocus::ChatHistory)),
                Char('q') | Char('Q') => Some(TuiEvent::Exit),
                Char('l') | Char('L') => Some(TuiEvent::ToggleLogs),
                Char(_) => Some(TuiEvent::ChatFocusChange(ChatFocus::ChatInput(0))),
                _ => None,
            },
            ChatFocus::ChatHistory => match key_event.code {
                Left => Some(TuiEvent::ChatFocusChange(ChatFocus::Channels)),
                Right if global_state.show_logs => Some(TuiEvent::ChatFocusChange(ChatFocus::Logs)),
                Right => Some(TuiEvent::ChatFocusChange(ChatFocus::Users)),
                Up => Some(TuiEvent::ScrollUp),
                Down => Some(TuiEvent::ScrollDown),
                Char('q') | Char('Q') => Some(TuiEvent::Exit),
                Char('l') | Char('L') => Some(TuiEvent::ToggleLogs),
                Char(_) => Some(TuiEvent::ChatFocusChange(ChatFocus::ChatInput(0))),
                _ => None,
            },
            ChatFocus::ChatInput(_) => match key_event.code {
                Up => Some(TuiEvent::ChatFocusChange(ChatFocus::ChatHistory)),
                Left if key_event.modifiers == KeyModifiers::CONTROL => Some(TuiEvent::InputLeftTab),
                Right if key_event.modifiers == KeyModifiers::CONTROL => Some(TuiEvent::InputRightTab),
                Left => Some(TuiEvent::InputLeft),
                Right => Some(TuiEvent::InputRight),
                Enter => Some(TuiEvent::InputEnter),
                Char(chr) => Some(TuiEvent::InputChar(chr)),
                Backspace => Some(TuiEvent::InputDelete),

                _ => None,
            },
            ChatFocus::Users => match key_event.code {
                Left if global_state.show_logs => Some(TuiEvent::ChatFocusChange(ChatFocus::Logs)),
                Left => Some(TuiEvent::ChatFocusChange(ChatFocus::ChatHistory)),
                Char('q') | Char('Q') => Some(TuiEvent::Exit),
                Char('l') | Char('L') => Some(TuiEvent::ToggleLogs),
                Char(_) => Some(TuiEvent::ChatFocusChange(ChatFocus::ChatInput(0))),
                _ => None,
            },
            ChatFocus::Logs => match key_event.code {
                Left => Some(TuiEvent::ChatFocusChange(ChatFocus::ChatHistory)),
                Right => Some(TuiEvent::ChatFocusChange(ChatFocus::Users)),
                Up => Some(TuiEvent::ScrollUp),
                Down => Some(TuiEvent::ScrollDown),
                Char('q') | Char('Q') => Some(TuiEvent::Exit),
                Char('l') | Char('L') => Some(TuiEvent::ToggleLogs),
                Char(_) => Some(TuiEvent::ChatFocusChange(ChatFocus::ChatInput(0))),

                _ => None,
            },
        },
        _ => None,
    }
}
