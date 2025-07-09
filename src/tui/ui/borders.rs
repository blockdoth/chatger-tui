use ratatui::style::{Color, Style};
use ratatui::symbols::{border, line};
use ratatui::widgets::Borders;

use crate::tui::{Focus, State};

pub fn borders_channel(state: &State) -> (Borders, Style, border::Set) {
    match state.focus {
        Focus::Channels => (
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

pub fn borders_profile(state: &State) -> (Borders, Style, border::Set) {
    match state.focus {
        Focus::Channels => (
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

pub fn borders_chat_history(state: &State) -> (Borders, Style, border::Set) {
    match state.focus {
        Focus::Channels => (
            Borders::RIGHT | Borders::TOP | Borders::BOTTOM,
            Style::default(),
            border::Set {
                bottom_right: if state.show_logs {
                    line::NORMAL.horizontal_up
                } else {
                    line::NORMAL.cross
                },
                top_right: line::NORMAL.horizontal_down,
                ..border::PLAIN
            },
        ),
        Focus::ChatHistory => (
            Borders::ALL,
            Style::default().fg(Color::Cyan),
            border::Set {
                bottom_left: line::NORMAL.cross,
                bottom_right: if state.show_logs {
                    line::NORMAL.horizontal_up
                } else {
                    line::NORMAL.cross
                },
                top_right: line::NORMAL.horizontal_down,
                top_left: line::NORMAL.horizontal_down,
                ..border::PLAIN
            },
        ),
        Focus::Users => (
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
        Focus::ChatInput(_) => (
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
        Focus::Logs => (
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

pub fn borders_input(state: &State) -> (Borders, Style, border::Set) {
    match state.focus {
        Focus::Channels => (
            Borders::RIGHT | Borders::BOTTOM,
            Style::default(),
            border::Set {
                bottom_right: line::NORMAL.horizontal_up,
                top_right: line::NORMAL.vertical_left,
                ..border::PLAIN
            },
        ),
        Focus::ChatHistory | Focus::Logs => (
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
        Focus::ChatInput(_) => (
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
        Focus::Users => (
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

pub fn borders_users(state: &State) -> (Borders, Style, border::Set) {
    match state.focus {
        Focus::ChatHistory => (
            Borders::RIGHT | Borders::TOP | Borders::BOTTOM,
            Style::default(),
            border::Set {
                bottom_right: line::NORMAL.vertical_left,
                ..border::PLAIN
            },
        ),
        Focus::Logs => (
            Borders::RIGHT | Borders::TOP | Borders::BOTTOM,
            Style::default(),
            border::Set {
                bottom_right: line::NORMAL.vertical_left,
                ..border::PLAIN
            },
        ),
        Focus::ChatInput(_) => (
            Borders::TOP | Borders::BOTTOM | Borders::RIGHT,
            Style::default(),
            border::Set {
                bottom_left: line::NORMAL.horizontal_up,
                bottom_right: line::NORMAL.vertical_left,
                top_left: line::NORMAL.horizontal_down,
                ..border::PLAIN
            },
        ),
        Focus::Users => (
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

pub fn borders_logs(state: &State) -> (Borders, Style, border::Set) {
    match state.focus {
        Focus::Channels => (
            Borders::RIGHT | Borders::TOP | Borders::BOTTOM,
            Style::default(),
            border::Set {
                bottom_right: line::NORMAL.cross,
                top_right: line::NORMAL.horizontal_down,
                ..border::PLAIN
            },
        ),
        Focus::ChatHistory => (
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
        Focus::Users => (
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
        Focus::ChatInput(_) => (
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
        Focus::Logs => (
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

pub fn borders_server_status(state: &State) -> (Borders, Style, border::Set) {
    match state.focus {
        Focus::Users => (
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
