use ratatui::style::{Color, Style};
use ratatui::symbols::{border, line};
use ratatui::widgets::Borders;

use crate::tui::screens::chat::state::ChatFocus;
use crate::tui::{ChatState, GlobalState};

pub fn borders_channel(state: &ChatState) -> (Borders, Style, border::Set) {
    match state.focus {
        ChatFocus::Channels => (
            Borders::ALL,
            Style::default().fg(Color::Cyan),
            border::Set {
                bottom_left: line::NORMAL.vertical_right,
                bottom_right: line::NORMAL.cross,
                top_right: line::NORMAL.horizontal_down,
                ..border::PLAIN
            },
        ),
        _ => (
            Borders::TOP | Borders::LEFT | Borders::BOTTOM,
            Style::default(),
            border::Set {
                bottom_left: line::NORMAL.vertical_right,
                ..border::PLAIN
            },
        ),
    }
}

pub fn borders_profile(state: &ChatState) -> (Borders, Style, border::Set) {
    match state.focus {
        ChatFocus::Channels => (
            Borders::LEFT | Borders::RIGHT | Borders::BOTTOM,
            Style::default(),
            border::Set {
                top_right: line::NORMAL.horizontal_down,
                top_left: line::NORMAL.vertical_right,
                bottom_left: line::NORMAL.vertical_right,
                bottom_right: line::NORMAL.horizontal_up,
                ..border::PLAIN
            },
        ),
        _ => (
            Borders::LEFT | Borders::BOTTOM,
            Style::default(),
            border::Set {
                top_right: line::NORMAL.horizontal_down,
                top_left: line::NORMAL.vertical_right,
                bottom_left: line::NORMAL.vertical_right,
                bottom_right: line::NORMAL.horizontal_up,
                ..border::PLAIN
            },
        ),
    }
}

pub fn borders_chat_history(global_state: &GlobalState, chat_state: &ChatState) -> (Borders, Style, border::Set) {
    match chat_state.focus {
        ChatFocus::Channels => (
            Borders::RIGHT | Borders::TOP | Borders::BOTTOM,
            Style::default(),
            border::Set {
                bottom_right: if global_state.show_logs {
                    line::NORMAL.horizontal_up
                } else {
                    line::NORMAL.cross
                },
                top_right: line::NORMAL.horizontal_down,
                ..border::PLAIN
            },
        ),
        ChatFocus::ChatHistory => (
            Borders::ALL,
            Style::default().fg(Color::Cyan),
            border::Set {
                bottom_left: line::NORMAL.cross,
                bottom_right: if global_state.show_logs {
                    line::NORMAL.horizontal_up
                } else {
                    line::NORMAL.cross
                },
                top_right: line::NORMAL.horizontal_down,
                top_left: line::NORMAL.horizontal_down,
                ..border::PLAIN
            },
        ),
        ChatFocus::Users => (
            Borders::TOP | Borders::LEFT | Borders::BOTTOM,
            Style::default(),
            border::Set {
                bottom_left: line::NORMAL.cross,
                bottom_right: line::NORMAL.vertical_left,
                top_right: line::NORMAL.horizontal_down,
                top_left: line::NORMAL.horizontal_down,
                ..border::PLAIN
            },
        ),
        ChatFocus::ChatInput(_) => (
            Borders::TOP | Borders::RIGHT | Borders::LEFT,
            Style::default(),
            border::Set {
                bottom_left: line::NORMAL.cross,
                bottom_right: line::NORMAL.horizontal_up,
                top_right: line::NORMAL.horizontal_down,
                top_left: line::NORMAL.horizontal_down,
                ..border::PLAIN
            },
        ),
        ChatFocus::Logs => (
            Borders::TOP | Borders::LEFT | Borders::BOTTOM,
            Style::default(),
            border::Set {
                bottom_left: line::NORMAL.cross,
                bottom_right: line::NORMAL.horizontal_up,
                top_right: line::NORMAL.horizontal_down,
                top_left: line::NORMAL.horizontal_down,
                ..border::PLAIN
            },
        ),
    }
}

pub fn borders_input(state: &ChatState) -> (Borders, Style, border::Set) {
    match state.focus {
        ChatFocus::Channels => (
            Borders::RIGHT | Borders::BOTTOM,
            Style::default(),
            border::Set {
                bottom_right: line::NORMAL.horizontal_up,
                top_right: line::NORMAL.vertical_left,
                ..border::PLAIN
            },
        ),
        ChatFocus::ChatHistory | ChatFocus::Logs => (
            Borders::LEFT | Borders::RIGHT | Borders::BOTTOM,
            Style::default(),
            border::Set {
                bottom_left: line::NORMAL.horizontal_up,
                bottom_right: line::NORMAL.horizontal_up,
                top_right: line::NORMAL.horizontal_down,
                top_left: line::NORMAL.horizontal_down,
                ..border::PLAIN
            },
        ),
        ChatFocus::ChatInput(_) => (
            Borders::ALL,
            Style::default().fg(Color::Cyan),
            border::Set {
                bottom_left: line::NORMAL.horizontal_up,
                bottom_right: line::NORMAL.horizontal_up,
                top_right: line::NORMAL.cross,
                top_left: line::NORMAL.cross,
                ..border::PLAIN
            },
        ),
        ChatFocus::Users => (
            Borders::BOTTOM | Borders::LEFT,
            Style::default(),
            border::Set {
                bottom_left: line::NORMAL.horizontal_up,
                top_left: line::NORMAL.vertical_right,
                ..border::PLAIN
            },
        ),
    }
}

pub fn borders_users(state: &ChatState) -> (Borders, Style, border::Set) {
    match state.focus {
        ChatFocus::ChatHistory => (
            Borders::RIGHT | Borders::TOP | Borders::BOTTOM,
            Style::default(),
            border::Set {
                bottom_right: line::NORMAL.vertical_left,
                ..border::PLAIN
            },
        ),
        ChatFocus::Logs => (
            Borders::RIGHT | Borders::TOP | Borders::BOTTOM,
            Style::default(),
            border::Set {
                bottom_right: line::NORMAL.vertical_left,
                ..border::PLAIN
            },
        ),
        ChatFocus::ChatInput(_) => (
            Borders::TOP | Borders::BOTTOM | Borders::RIGHT,
            Style::default(),
            border::Set {
                bottom_left: line::NORMAL.horizontal_up,
                bottom_right: line::NORMAL.vertical_left,
                top_left: line::NORMAL.horizontal_down,
                ..border::PLAIN
            },
        ),
        ChatFocus::Users => (
            Borders::ALL,
            Style::default().fg(Color::Cyan),
            border::Set {
                bottom_left: line::NORMAL.cross,
                bottom_right: line::NORMAL.vertical_left,
                top_left: line::NORMAL.horizontal_down,
                ..border::PLAIN
            },
        ),
        _ => (
            Borders::TOP | Borders::RIGHT | Borders::BOTTOM,
            Style::default(),
            border::Set {
                bottom_left: line::NORMAL.vertical_right,
                bottom_right: line::NORMAL.vertical_left,
                ..border::PLAIN
            },
        ),
    }
}

pub fn borders_logs(state: &ChatState) -> (Borders, Style, border::Set) {
    match state.focus {
        ChatFocus::Channels => (
            Borders::RIGHT | Borders::TOP | Borders::BOTTOM,
            Style::default(),
            border::Set {
                bottom_right: line::NORMAL.cross,
                top_right: line::NORMAL.horizontal_down,
                ..border::PLAIN
            },
        ),
        ChatFocus::ChatHistory => (
            Borders::TOP | Borders::BOTTOM | Borders::RIGHT,
            Style::default(),
            border::Set {
                bottom_left: line::NORMAL.cross,
                bottom_right: line::NORMAL.cross,
                top_right: line::NORMAL.horizontal_down,
                top_left: line::NORMAL.horizontal_down,
                ..border::PLAIN
            },
        ),
        ChatFocus::Users => (
            Borders::TOP | Borders::LEFT | Borders::BOTTOM,
            Style::default(),
            border::Set {
                bottom_left: line::NORMAL.horizontal_up,
                bottom_right: line::NORMAL.cross,
                top_right: line::NORMAL.horizontal_down,
                top_left: line::NORMAL.horizontal_down,
                ..border::PLAIN
            },
        ),
        ChatFocus::ChatInput(_) => (
            Borders::TOP | Borders::RIGHT,
            Style::default(),
            border::Set {
                bottom_left: line::NORMAL.horizontal_up,
                bottom_right: line::NORMAL.cross,
                top_right: line::NORMAL.horizontal_down,
                top_left: line::NORMAL.horizontal_down,
                ..border::PLAIN
            },
        ),
        ChatFocus::Logs => (
            Borders::ALL,
            Style::default().fg(Color::Cyan),
            border::Set {
                bottom_left: line::NORMAL.horizontal_up,
                bottom_right: line::NORMAL.cross,
                top_right: line::NORMAL.horizontal_down,
                top_left: line::NORMAL.horizontal_down,
                ..border::PLAIN
            },
        ),
    }
}

pub fn borders_server_status(state: &ChatState) -> (Borders, Style, border::Set) {
    match state.focus {
        ChatFocus::Users => (
            Borders::LEFT | Borders::RIGHT | Borders::BOTTOM,
            Style::default(),
            border::Set {
                top_right: line::NORMAL.vertical_left,
                top_left: line::NORMAL.cross,
                bottom_left: line::NORMAL.horizontal_up,
                bottom_right: line::NORMAL.vertical_left,
                ..border::PLAIN
            },
        ),
        _ => (
            Borders::RIGHT | Borders::BOTTOM,
            Style::default(),
            border::Set {
                top_right: line::NORMAL.vertical_left,
                top_left: line::NORMAL.cross,
                bottom_left: line::NORMAL.horizontal_up,
                bottom_right: line::NORMAL.vertical_left,
                ..border::PLAIN
            },
        ),
    }
}
