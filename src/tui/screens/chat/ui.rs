use std::collections::HashMap;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Padding, Paragraph, Wrap};

use crate::network::client::ServerConnectionStatus;
use crate::network::protocol::UserStatus;
use crate::tui::chat::{ChannelStatus, ChatMessageStatus, User};
use crate::tui::screens::GlobalState;
use crate::tui::screens::chat::borders::{
    borders_channel, borders_chat_history, borders_input, borders_logs, borders_profile, borders_server_status, borders_users,
};
use crate::tui::screens::chat::{ChatFocus, ChatState};

const HEADER_STYLE: Style = Style {
    fg: None,
    bg: None,
    underline_color: None,
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};

const PADDING: Padding = Padding::new(1, 1, 0, 0);

pub fn draw_main(global_state: &GlobalState, chat_state: &ChatState, frame: &mut Frame) {
    let main_area = frame.area();
    let (app_area, info_area) = split_app_info_areas(global_state, main_area);
    let (channels_area, chat_area, users_area) = split_channel_chat_user_areas(global_state, chat_state, app_area);
    let (users_area, server_status_area) = split_users_server_areas(global_state, chat_state, users_area);
    let (channels_area, profile_area) = split_channels_profile_areas(global_state, chat_state, channels_area);
    let (chat_history_area, chat_input_area) = split_chatlog_chatinput_areas(global_state, chat_state, chat_area);

    let chat_history_area = if global_state.show_logs {
        let (chat_history_area, logs_area) = split_chat_log_areas(global_state, chat_state, chat_history_area);
        render_logs(global_state, chat_state, frame, logs_area);
        chat_history_area
    } else {
        chat_history_area
    };

    render_channels(global_state, chat_state, frame, channels_area);
    render_profile(global_state, chat_state, frame, profile_area);
    render_chat_history(global_state, chat_state, frame, chat_history_area);
    render_chat_input(global_state, chat_state, frame, chat_input_area);
    render_users(global_state, chat_state, frame, users_area);
    render_server_status(global_state, chat_state, frame, server_status_area);
    render_info(global_state, chat_state, frame, info_area);
}

pub fn split_app_info_areas(_global_state: &GlobalState, area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Fill(10), Constraint::Length(2)])
        .split(area);
    (chunks[0], chunks[1])
}

fn split_channel_chat_user_areas(_global_state: &GlobalState, chat_state: &ChatState, area: Rect) -> (Rect, Rect, Rect) {
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

fn split_channels_profile_areas(_global_state: &GlobalState, _chat_state: &ChatState, area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Fill(10), Constraint::Length(4)])
        .split(area);
    (chunks[0], chunks[1])
}

fn split_users_server_areas(_global_state: &GlobalState, _chat_state: &ChatState, area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Fill(10), Constraint::Length(4)])
        .split(area);
    (chunks[0], chunks[1])
}

fn split_chatlog_chatinput_areas(_global_state: &GlobalState, chat_state: &ChatState, area: Rect) -> (Rect, Rect) {
    let input_height = if let ChatFocus::ChatInput(_) = chat_state.focus { 5 } else { 4 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Fill(10), Constraint::Length(input_height)])
        .split(area);
    (chunks[0], chunks[1])
}

// Done manually because of issues with border highlights creating small shifts
fn split_chat_log_areas(_global_state: &GlobalState, chat_state: &ChatState, area: Rect) -> (Rect, Rect) {
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

fn render_channels(_global_state: &GlobalState, chat_state: &ChatState, frame: &mut Frame, area: Rect) {
    let channels: Vec<Line> = if chat_state.channels.is_empty() {
        vec![Line::from(Span::styled(
            "This server has no channels",
            Style::default().add_modifier(Modifier::DIM | Modifier::ITALIC),
        ))]
    } else {
        chat_state
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
            .collect()
    };

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

fn render_profile(_global_state: &GlobalState, chat_state: &ChatState, frame: &mut Frame, area: Rect) {
    let (borders, border_style, border_corners) = borders_profile(chat_state);

    let (symbol, user_status_style) = user_status(&chat_state.current_user.status);

    let username = Span::styled(format!("{symbol} {}", chat_state.current_user.username), user_status_style);

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

fn render_server_status(_global_state: &GlobalState, chat_state: &ChatState, frame: &mut Frame, area: Rect) {
    let (borders, border_style, border_corners) = borders_server_status(chat_state);
    let connection_status = match chat_state.server_connection_status {
        ServerConnectionStatus::Connected => Span::styled("Server: [Connected]".to_owned(), Style::default().fg(Color::Green)),
        ServerConnectionStatus::Unhealthy => Span::styled("Server: [Unhealthy]".to_owned(), Style::default().fg(Color::LightYellow)),
        ServerConnectionStatus::Disconnected => Span::styled("Server: [Disconnected]".to_owned(), Style::default().fg(Color::LightRed)),
        ServerConnectionStatus::Reconnecting => Span::styled("Server: [Reconnecting]".to_owned(), Style::default().fg(Color::LightYellow)),
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

    let (channel_id, channel_name) = if let Some(channel) = &chat_state.channels.get(chat_state.active_channel_idx) {
        (channel.id, channel.name.clone())
    } else {
        (0, "Should not be shown".to_string())
    };

    let chat_log = chat_state.chat_history.get(&channel_id).unwrap_or(empty);

    let chatlog_lines: Vec<Line> = if chat_log.is_empty() {
        vec![Line::from(Span::styled(
            format!("Be the first to message in #{channel_name}"),
            Style::default().add_modifier(Modifier::DIM | Modifier::ITALIC),
        ))]
    } else {
        let current_message_line_count = chat_log.len();

        let start_index = current_message_line_count
            .saturating_sub((area.height.div_ceil(2)).saturating_sub(1) as usize)
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

    //     .title(
    //     Title::from(Span::styled(
    //         "Bottom Title",
    //         Style::default().add_modifier(Modifier::ITALIC),
    //     ))
    //     .position(ratatui::widgets::TitlePosition::Bottom),
    // );

    let mut block = Block::default()
        .padding(PADDING)
        .border_set(border_corners)
        .borders(borders)
        .border_style(border_style)
        .title(Span::styled(format!("Chat Log [{}]", &channel_name), HEADER_STYLE));

    let users_typing = match chat_state.focus {
        ChatFocus::ChatInput(_) => "".to_owned(),
        _ => is_typing(
            &chat_state
                .users_typing
                .get(&channel_id)
                .unwrap_or(&HashMap::new())
                .values()
                .cloned()
                .collect(),
        ),
    };

    if !users_typing.is_empty() {
        block = block.title_bottom(Span::styled(users_typing, Modifier::ITALIC | Modifier::DIM));
    };

    let widget = Paragraph::new(Text::from(chatlog_lines)).wrap(Wrap { trim: false }).block(block);
    frame.render_widget(widget, area);
}

fn render_chat_input(_global_state: &GlobalState, chat_state: &ChatState, frame: &mut Frame, area: Rect) {
    let (channel_id, channel_name) = match chat_state.channels.get(chat_state.active_channel_idx) {
        Some(channel) => (channel.id, channel.name.clone()),
        None => (0, "Should not be seen".to_owned()),
    };

    let input_line = match chat_state.chat_inputs.get(&channel_id) {
        Some(line) if !line.is_empty() => {
            if matches!(chat_state.focus, ChatFocus::ChatInput(_)) {
                format!("{line} ")
                    .char_indices()
                    .map(|(idx, chr)| {
                        if let ChatFocus::ChatInput(focussed_idx) = chat_state.focus
                            && focussed_idx == idx
                        {
                            Span::styled(chr.to_string(), Modifier::UNDERLINED)
                        } else {
                            Span::from(chr.to_string())
                        }
                    })
                    .collect()
            } else {
                vec![Span::from(line)]
            }
        }
        _ => {
            vec![Span::styled(
                format!("Message #{channel_name}"),
                Style::default().add_modifier(Modifier::DIM | Modifier::ITALIC),
            )]
        }
    };

    let users_typing = match chat_state.focus {
        ChatFocus::ChatInput(_) => is_typing(
            &chat_state
                .users_typing
                .get(&channel_id)
                .unwrap_or(&HashMap::new())
                .values()
                .cloned()
                .collect(),
        ),
        _ => "".to_owned(),
    };

    let (borders, border_style, border_corners) = borders_input(chat_state);
    let mut block = Block::default()
        .padding(PADDING)
        .border_set(border_corners)
        .borders(borders)
        .border_style(border_style);

    let input_text = if users_typing.is_empty() {
        vec![Line::raw(""), Line::from(input_line)]
    } else {
        block = block.title(Span::styled(users_typing, Modifier::ITALIC | Modifier::DIM));
        vec![Line::raw(""), Line::from(input_line)]
    };

    let widget = Paragraph::new(Text::from(input_text)).block(block);
    frame.render_widget(widget, area);
}

fn render_users(_global_state: &GlobalState, chat_state: &ChatState, frame: &mut Frame, area: Rect) {
    let (mut online_users, mut offline_users): (Vec<&User>, Vec<&User>) = chat_state
        .users
        .iter()
        .filter(|user| chat_state.current_user.username != user.name)
        .partition(|user| matches!(user.status, UserStatus::Online | UserStatus::Idle | UserStatus::DoNotDisturb));

    online_users.sort_by_key(|user| &user.name);
    offline_users.sort_by_key(|user| &user.name);

    let format_user_line = |user: &User| {
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
            Span::styled(format!(" {symbol} "), symbol_style),
            Span::styled(user.name.clone(), name_style),
        ])
    };

    let mut lines = vec![];

    if !online_users.is_empty() {
        lines.push(Line::from(Span::styled(
            "Online",
            Style::default().fg(Color::Green).add_modifier(Modifier::UNDERLINED),
        )));
        for user in &online_users {
            lines.push(format_user_line(user));
        }
        lines.push(Line::from(""));
    }

    if !offline_users.is_empty() {
        lines.push(Line::from(Span::styled(
            "Offline",
            Style::default().fg(Color::Gray).add_modifier(Modifier::UNDERLINED),
        )));
        for user in &offline_users {
            lines.push(format_user_line(user));
        }
    }
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
        ChatFocus::Channels => "[↑↓] Change Channel | [Enter | →] Chat log | [L]ogs | [Q]uit",
        ChatFocus::ChatHistory if global_state.show_logs => "[↓] Input | [←] Channels | [→] Logs | [L]ogs | [Q]uit",
        ChatFocus::ChatHistory => "[Enter | Space ] Input | [←] Channels | [→] Users | [L]ogs | [Q]uit",
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

#[allow(clippy::ptr_arg)] // TODO fix
fn is_typing(is_typing: &Vec<String>) -> String {
    match is_typing.len() {
        0 => String::new(),
        typers if typers > 4 => " Several people are typing... ".to_owned(),
        typers => {
            let mut string = String::new();
            string.push(' ');
            for (idx, user) in is_typing.iter().enumerate() {
                string.push_str(user);

                match idx {
                    i if typers > 1 && i == typers - 2 => string.push_str(" and "),
                    i if typers > 1 && i < typers - 2 => string.push_str(", "),
                    _ => {}
                }
            }

            if typers == 1 {
                string.push_str(" is typing... ");
            } else {
                string.push_str(" are typing... ");
            }

            string
        }
    }
}

fn user_status(status: &UserStatus) -> (String, Style) {
    match status {
        UserStatus::Offline => ("●".to_owned(), Style::default().fg(Color::Gray).add_modifier(Modifier::DIM)),
        UserStatus::Online => ("●".to_owned(), Style::default().fg(Color::Green)),
        UserStatus::Idle => ("●".to_owned(), Style::default().fg(Color::Yellow)),
        UserStatus::DoNotDisturb => ("●".to_owned(), Style::default().fg(Color::Red)),
    }
}
