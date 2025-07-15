mod borders;
pub mod keys;
pub mod state;

use chrono::{Duration, Utc};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Padding, Paragraph, Wrap};

use crate::network::protocol::UserStatus;
use crate::tui::chat::{ChannelStatus, ChatMessageStatus};
use crate::tui::screens::chat::borders::{
    borders_channel, borders_chat_history, borders_input, borders_logs, borders_profile, borders_server_status, borders_users,
};
use crate::tui::screens::chat::state::{ChatFocus, ChatState};
use crate::tui::GlobalState;

const HEADER_STYLE: Style = Style {
    fg: None,
    bg: None,
    underline_color: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

const PADDING: Padding = Padding::new(1, 1, 0, 0);

pub fn draw_main(global_state: &GlobalState, chat_state: &ChatState, frame: &mut Frame) {
    let main_area = frame.area();
    let (app_area, info_area) = split_app_info_areas(global_state, main_area);
    let (channels_area, chat_area, users_area) = split_channel_chat_user_areas(global_state, chat_state, app_area);
    let (users_area, server_status_area) = split_users_server_areas(global_state, chat_state, users_area);
    let (channels_area, profile_area) = split_channels_profile_areas(global_state, chat_state, channels_area);
    let (chat_log_area, chat_input_area) = split_chatlog_chatinput_areas(global_state, chat_state, chat_area);

    let chat_log_area = if global_state.show_logs {
        let (chat_log_area, logs_area) = split_chat_log_areas(global_state, chat_state, chat_log_area);
        render_logs(global_state, chat_state, frame, logs_area);
        chat_log_area
    } else {
        chat_log_area
    };

    render_channels(global_state, chat_state, frame, channels_area);
    render_profile(global_state, chat_state, frame, profile_area);
    render_chat_history(global_state, chat_state, frame, chat_log_area);
    render_chat_input(global_state, chat_state, frame, chat_input_area);
    render_users(global_state, chat_state, frame, users_area);
    render_server_status(global_state, chat_state, frame, server_status_area);
    render_info(global_state, chat_state, frame, info_area);
}

pub fn split_app_info_areas(global_state: &GlobalState, area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Fill(10), Constraint::Length(2)])
        .split(area);
    (chunks[0], chunks[1])
}

fn split_channel_chat_user_areas(global_state: &GlobalState, chat_state: &ChatState, area: Rect) -> (Rect, Rect, Rect) {
    let channel_width_offset = if chat_state.focus == ChatFocus::Channels { 0 } else { 1 };
    let users_width_offset = if chat_state.focus == ChatFocus::Users { 1 } else { 0 };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints([
            Constraint::Length(30 - channel_width_offset),
            Constraint::Fill(10),
            Constraint::Length(30 + users_width_offset),
        ])
        .split(area);
    (chunks[0], chunks[1], chunks[2])
}

fn split_channels_profile_areas(global_state: &GlobalState, chat_state: &ChatState, area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Fill(10), Constraint::Length(4)])
        .split(area);
    (chunks[0], chunks[1])
}

fn split_users_server_areas(global_state: &GlobalState, chat_state: &ChatState, area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Fill(10), Constraint::Length(4)])
        .split(area);
    (chunks[0], chunks[1])
}

fn split_chatlog_chatinput_areas(global_state: &GlobalState, chat_state: &ChatState, area: Rect) -> (Rect, Rect) {
    let input_height = if let ChatFocus::ChatInput(_) = chat_state.focus { 5 } else { 4 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Fill(10), Constraint::Length(input_height)])
        .split(area);
    (chunks[0], chunks[1])
}

// Done manually because of issues with border highlights creating small shifts
fn split_chat_log_areas(global_state: &GlobalState, chat_state: &ChatState, area: Rect) -> (Rect, Rect) {
    let left_width = area.width / 2 + (area.width % 2);
    let right_width = area.width - left_width;

    let offset = if let ChatFocus::ChatHistory | ChatFocus::ChatInput(_) = chat_state.focus {
        1
    } else {
        0
    };

    let left = Rect {
        x: area.x,
        y: area.y,
        width: left_width + offset,
        height: area.height,
    };
    let right = Rect {
        x: area.x + left_width + offset,
        y: area.y,
        width: right_width - offset,
        height: area.height,
    };
    (left, right)
}

fn render_channels(global_state: &GlobalState, chat_state: &ChatState, frame: &mut Frame, area: Rect) {
    let channels: Vec<Line> = chat_state
        .channels
        .iter()
        .map(|channel| {
            let mut style = match channel.status {
                ChannelStatus::Read => Style::default(),
                ChannelStatus::Unread => Style::default().add_modifier(Modifier::BOLD),
                ChannelStatus::Muted => Style::default().add_modifier(Modifier::DIM),
            };
            if channel.id == chat_state.channels.get(chat_state.active_channel_idx).unwrap().id {
                style = style.bg(Color::DarkGray);
            }

            Line::from(Span::styled(format!("# {:15}", channel.name.clone()), style))
        })
        .collect();

    let (borders, border_style, border_corners) = borders_channel(chat_state);
    let widget = Paragraph::new(Text::from(channels)).block(
        Block::default()
            .padding(PADDING)
            .border_set(border_corners)
            .borders(borders)
            .border_style(border_style)
            .title(Span::styled("Channels".to_string(), HEADER_STYLE)),
    );
    frame.render_widget(widget, area);
}

fn render_profile(global_state: &GlobalState, chat_state: &ChatState, frame: &mut Frame, area: Rect) {
    let (borders, border_style, border_corners) = borders_profile(chat_state);

    let username = Span::styled(
        chat_state.current_user.username.clone().to_string(),
        Style::default().fg(Color::LightGreen),
    );

    let lines = vec![Line::from(Span::from("")), Line::from(username)];

    let widget = Paragraph::new(Text::from(lines)).block(
        Block::default()
            .padding(PADDING)
            .border_set(border_corners)
            .borders(borders)
            .border_style(border_style),
    );
    frame.render_widget(widget, area);
}

fn render_server_status(global_state: &GlobalState, chat_state: &ChatState, frame: &mut Frame, area: Rect) {
    let (borders, border_style, border_corners) = borders_server_status(chat_state);
    let connection_status = if Utc::now() - chat_state.last_healthcheck < Duration::seconds(10) {
        Span::styled("Server: [Connected]".to_owned(), Style::default().fg(Color::Green))
    } else {
        Span::styled("Server: [Disconnected]".to_owned(), Style::default().fg(Color::LightRed))
    };

    let lines = vec![Line::from(Span::from("")), Line::from(connection_status)];

    let widget = Paragraph::new(Text::from(lines)).block(
        Block::default()
            .padding(PADDING)
            .border_set(border_corners)
            .borders(borders)
            .border_style(border_style),
    );
    frame.render_widget(widget, area);
}

fn render_chat_history(global_state: &GlobalState, chat_state: &ChatState, frame: &mut Frame, area: Rect) {
    // TODO make less ugly
    let empty = &vec![];

    let chatlog_lines: Vec<Line> = if chat_state.chat_history.is_empty() {
        vec![Line::from(Span::raw(""))]
    } else {
        let channel_id = if let Some(channel) = &chat_state.channels.get(chat_state.active_channel_idx) {
            channel.id
        } else {
            0
        };

        let chat_log = chat_state.chat_history.get(&channel_id).unwrap_or(empty);

        let current_message_line_count = chat_log.len();

        let start_index = current_message_line_count
            .saturating_sub(area.height.saturating_sub(2) as usize)
            .saturating_sub(chat_state.chat_scroll_offset);

        chat_log
            .iter()
            .skip(start_index)
            .flat_map(|chat_message| {
                let timestamp = chat_message.timestamp.format("%H:%M:%S").to_string();

                let header_style = match chat_message.status {
                    ChatMessageStatus::Send => Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    ChatMessageStatus::Sending => Style::default().fg(Color::Yellow).add_modifier(Modifier::DIM | Modifier::ITALIC),
                    ChatMessageStatus::FailedToSend => Style::default().fg(Color::LightRed).add_modifier(Modifier::DIM | Modifier::ITALIC),
                };

                let body_style = match chat_message.status {
                    ChatMessageStatus::Send => Style::default().fg(Color::Gray),
                    ChatMessageStatus::Sending => Style::default().fg(Color::Gray).add_modifier(Modifier::DIM | Modifier::ITALIC),
                    ChatMessageStatus::FailedToSend => Style::default().fg(Color::LightRed).add_modifier(Modifier::DIM | Modifier::ITALIC),
                };

                let timestamp_style = match chat_message.status {
                    ChatMessageStatus::Send => Style::default().fg(Color::DarkGray),
                    ChatMessageStatus::Sending | ChatMessageStatus::FailedToSend => {
                        Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)
                    }
                };

                let header = Line::from(vec![
                    Span::styled(format!("{} ", &chat_message.author_name), header_style),
                    Span::styled(format!(" [{timestamp}] "), timestamp_style),
                    (match chat_message.status {
                        ChatMessageStatus::Send => Span::raw(""),
                        ChatMessageStatus::Sending => Span::styled("sending...", Style::default().add_modifier(Modifier::DIM | Modifier::ITALIC)),
                        ChatMessageStatus::FailedToSend => Span::styled(
                            "failed to send",
                            Style::default().fg(Color::LightRed).add_modifier(Modifier::DIM | Modifier::ITALIC),
                        ),
                    }),
                ]);

                let body = Line::from(Span::styled(format!("\t{}", &chat_message.message), body_style));

                vec![header, body].into_iter()
            })
            .collect()
    };

    let (borders, border_style, border_corners) = borders_chat_history(global_state, chat_state);

    let widget = Paragraph::new(Text::from(chatlog_lines)).wrap(Wrap { trim: false }).block(
        Block::default()
            .padding(PADDING)
            .border_set(border_corners)
            .borders(borders)
            .border_style(border_style)
            .title(Span::styled("Chat Log".to_string(), HEADER_STYLE)),
    );
    frame.render_widget(widget, area);
}

fn render_chat_input(global_state: &GlobalState, chat_state: &ChatState, frame: &mut Frame, area: Rect) {
    let input_text: Vec<Span> = match chat_state.focus {
        ChatFocus::ChatInput(_) => chat_state
            .chat_input
            .chars()
            .enumerate()
            .map(|(idx, c)| {
                if let ChatFocus::ChatInput(focussed_idx) = chat_state.focus
                    && focussed_idx == idx
                {
                    Span::styled(c.to_string(), Modifier::UNDERLINED)
                } else {
                    Span::from(c.to_string())
                }
            })
            .collect(),
        _ => {
            if let Some(channel) = chat_state.channels.get(chat_state.active_channel_idx) {
                vec![Span::styled(
                    format!("Message #{}", channel.name),
                    Style::default().add_modifier(Modifier::DIM | Modifier::ITALIC),
                )]
            } else {
                vec![]
            }
        }
    };
    let input_line = vec![Line::from(Span::from("")), Line::from(input_text)];

    let (borders, border_style, border_corners) = borders_input(chat_state);
    let widget = Paragraph::new(Text::from(input_line)).block(
        Block::default()
            .padding(PADDING)
            .border_set(border_corners)
            .borders(borders)
            .border_style(border_style), // .title(Span::styled("Chat Input".to_string(), HEADER_STYLE)),
    );
    frame.render_widget(widget, area);
}

fn render_users(global_state: &GlobalState, chat_state: &ChatState, frame: &mut Frame, area: Rect) {
    let lines: Vec<Line> = chat_state
        .users
        .iter()
        .filter(|user| chat_state.current_user.username != user.name)
        .map(|user| {
            let (symbol, symbol_style) = match user.status {
                UserStatus::Offline => ("●", Style::default().fg(Color::Gray).add_modifier(Modifier::DIM)),
                UserStatus::Online => ("●", Style::default().fg(Color::Green)),
                UserStatus::Idle => ("●", Style::default().fg(Color::Yellow)),
                UserStatus::DoNotDisturb => ("●", Style::default().fg(Color::Red)),
            };
            let name_style = if let UserStatus::Offline = user.status {
                Style::default().fg(Color::Gray).add_modifier(Modifier::DIM)
            } else {
                Style::default()
            };

            Line::from(vec![
                Span::styled(format!("{symbol} "), symbol_style),
                Span::styled(user.name.clone(), name_style),
            ])
        })
        .collect();

    let (borders, border_style, border_corners) = borders_users(chat_state);

    let widget = Paragraph::new(Text::from(lines)).block(
        Block::default()
            .padding(PADDING)
            .border_set(border_corners)
            .borders(borders)
            .border_style(border_style)
            .title(Span::styled("Users".to_string(), HEADER_STYLE)),
    );
    frame.render_widget(widget, area);
}

fn render_info(global_state: &GlobalState, chat_state: &ChatState, frame: &mut Frame, area: Rect) {
    let info_text = match chat_state.focus {
        ChatFocus::Channels => "[↑↓] Change Channel | [→] Chat log | [L]ogs | [Q]uit",
        ChatFocus::ChatHistory => "[↓] Input | [←] Channels | [→] Chat log | [L]ogs | [Q]uit",
        ChatFocus::ChatInput(_) => {
            "[Enter] Send Message | [Backspace] Delete | [←→] Move Cursor | [Ctrl + ←→] Tab move Cursor | [↑] Chatlog | [L]ogs | [Q]uit"
        }
        ChatFocus::Users => "[←] Chat log | [L]ogs | [Q]uit",
        ChatFocus::Logs => "[L]ogs | [Q]uit",
    };

    let border_style = Style::default();
    let widget = Paragraph::new(Text::from(info_text)).block(
        Block::default()
            .padding(PADDING)
            .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
            .border_style(border_style),
    );
    frame.render_widget(widget, area);
}

fn render_logs(global_state: &GlobalState, chat_state: &ChatState, frame: &mut Frame, area: Rect) {
    let current_log_count = global_state.logs.len();
    let start_index = current_log_count
        .saturating_sub(area.height.saturating_sub(2) as usize)
        .saturating_sub(global_state.log_scroll_offset);

    let logs: Vec<Line> = global_state.logs.iter().skip(start_index).map(|entry| entry.format()).collect();

    let (borders, border_style, border_corners) = borders_logs(chat_state);

    let widget = Paragraph::new(Text::from(logs)).wrap(Wrap { trim: true }).block(
        Block::default()
            .padding(PADDING)
            .border_set(border_corners)
            .borders(borders)
            .border_style(border_style)
            .title(Span::styled("Log".to_string(), HEADER_STYLE)),
    );
    frame.render_widget(widget, area);
}
