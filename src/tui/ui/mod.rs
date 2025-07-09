mod borders;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::{border, line};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Padding, Paragraph, Wrap};

use crate::tui::chat::{ChannelStatus, ChatMessageStatus, UserStatus};
use crate::tui::ui::borders::{
    borders_channel, borders_chat_history, borders_input, borders_logs, borders_profile, borders_server_status, borders_users,
};
use crate::tui::{Focus, State};

const HEADER_STYLE: Style = Style {
    fg: None,
    bg: None,
    underline_color: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

const PADDING: Padding = Padding::new(1, 1, 0, 0);

pub fn draw(state: &State, frame: &mut Frame) {
    let main_area = frame.area();
    let (app_area, info_area) = split_app_info_areas(state, main_area);
    let (channels_area, chat_area, users_area) = split_channel_chat_user_areas(state, app_area);
    let (users_area, server_status_area) = split_users_server_areas(state, users_area);
    let (channels_area, profile_area) = split_channels_profile_areas(state, channels_area);
    let (chat_log_area, chat_input_area) = split_chatlog_chatinput_areas(state, chat_area);

    let chat_log_area = if state.show_logs {
        let (chat_log_area, logs_area) = split_chat_log_areas(state, chat_log_area);
        render_logs(state, frame, logs_area);
        chat_log_area
    } else {
        chat_log_area
    };

    render_channels(state, frame, channels_area);
    render_profile(state, frame, profile_area);
    render_chat_history(state, frame, chat_log_area);
    render_chat_input(state, frame, chat_input_area);
    render_users(state, frame, users_area);
    render_server_status(state, frame, server_status_area);
    render_info(state, frame, info_area);
}

fn split_app_info_areas(state: &State, area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Fill(10), Constraint::Length(2)])
        .split(area);
    (chunks[0], chunks[1])
}

fn split_channel_chat_user_areas(state: &State, area: Rect) -> (Rect, Rect, Rect) {
    let channel_width_offset = if state.focus == Focus::Channels { 0 } else { 1 };
    let users_width_offset = if state.focus == Focus::Users { 1 } else { 0 };

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

fn split_channels_profile_areas(state: &State, area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Fill(10), Constraint::Length(4)])
        .split(area);
    (chunks[0], chunks[1])
}

fn split_users_server_areas(state: &State, area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Fill(10), Constraint::Length(4)])
        .split(area);
    (chunks[0], chunks[1])
}

fn split_chatlog_chatinput_areas(state: &State, area: Rect) -> (Rect, Rect) {
    let input_height = if let Focus::ChatInput(_) = state.focus { 5 } else { 4 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Fill(10), Constraint::Length(input_height)])
        .split(area);
    (chunks[0], chunks[1])
}

// Done manually because of issues with border highlights creating small shifts
fn split_chat_log_areas(state: &State, area: Rect) -> (Rect, Rect) {
    let left_width = area.width / 2 + (area.width % 2);
    let right_width = area.width - left_width;

    let offset = if let Focus::ChatHistory | Focus::ChatInput(_) = state.focus {
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

fn render_channels(state: &State, frame: &mut Frame, area: Rect) {
    let channels: Vec<Line> = state
        .channels
        .iter()
        .map(|channel| {
            let mut style = match channel.status {
                ChannelStatus::Read => Style::default(),
                ChannelStatus::Unread => Style::default().add_modifier(Modifier::BOLD),
                ChannelStatus::Muted => Style::default().add_modifier(Modifier::DIM),
            };
            if channel.id == state.channels.get(state.active_channel_idx).unwrap().id {
                style = style.bg(Color::DarkGray);
            }

            Line::from(Span::styled(format!("# {:15}", channel.name.clone()), style))
        })
        .collect();

    let (borders, border_style, border_corners) = borders_channel(state);
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

fn render_profile(state: &State, frame: &mut Frame, area: Rect) {
    let (borders, border_style, border_corners) = borders_profile(state);
    let current_user = if let Some(user) = &state.current_user {
        format!("user: {}", user.name.clone())
    } else {
        "Not logged in :(".to_owned()
    };

    let lines = vec![Line::from(Span::from("")), Line::from(current_user)];

    let border_style = Style::default();
    let widget = Paragraph::new(Text::from(lines)).block(
        Block::default()
            .padding(PADDING)
            .border_set(border_corners)
            .borders(borders)
            .border_style(border_style),
    );
    frame.render_widget(widget, area);
}

fn render_server_status(state: &State, frame: &mut Frame, area: Rect) {
    let (borders, border_style, border_corners) = borders_server_status(state);
    let connection_status = if state.connected_with_server {
        Span::styled("Server: [Connected]".to_owned(), Style::default().fg(Color::Green))
    } else {
        Span::styled("Server: [Disconnected]".to_owned(), Style::default().fg(Color::LightRed))
    };

    let lines = vec![Line::from(Span::from("")), Line::from(connection_status)];

    let border_style = Style::default();
    let widget = Paragraph::new(Text::from(lines)).block(
        Block::default()
            .padding(PADDING)
            .border_set(border_corners)
            .borders(borders)
            .border_style(border_style),
    );
    frame.render_widget(widget, area);
}

fn render_chat_history(state: &State, frame: &mut Frame, area: Rect) {
    let border_corners = border::Set {
        top_left: line::NORMAL.horizontal_down,
        top_right: line::NORMAL.horizontal_down,
        ..border::PLAIN
    };

    let chatlog: Vec<Line> = state
        .chat_history
        .get(&state.channels.get(state.active_channel_idx).unwrap().id)
        .unwrap_or(&vec![])
        .iter()
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
                ChatMessageStatus::Sending | ChatMessageStatus::FailedToSend => Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
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
        .collect();

    let (borders, border_style, border_corners) = borders_chat_history(state);

    let widget = Paragraph::new(Text::from(chatlog)).wrap(Wrap { trim: false }).block(
        Block::default()
            .padding(PADDING)
            .border_set(border_corners)
            .borders(borders)
            .border_style(border_style)
            .title(Span::styled("Chat Log".to_string(), HEADER_STYLE)),
    );
    frame.render_widget(widget, area);
}

fn render_chat_input(state: &State, frame: &mut Frame, area: Rect) {
    let border_corners = border::Set {
        top_left: line::NORMAL.vertical_right,
        top_right: line::NORMAL.vertical_left,
        bottom_left: line::NORMAL.horizontal_up,
        bottom_right: line::NORMAL.horizontal_up,
        ..border::PLAIN
    };

    let input_text: Vec<Span> = state
        .chat_input
        .chars()
        .enumerate()
        .map(|(idx, c)| {
            if let Focus::ChatInput(focussed_idx) = state.focus
                && focussed_idx == idx
            {
                Span::styled(c.to_string(), Modifier::UNDERLINED)
            } else {
                Span::from(c.to_string())
            }
        })
        .collect();

    let input_line = vec![Line::from(Span::from("")), Line::from(input_text)];

    let (borders, border_style, border_corners) = borders_input(state);
    let widget = Paragraph::new(Text::from(input_line)).block(
        Block::default()
            .padding(PADDING)
            .border_set(border_corners)
            .borders(borders)
            .border_style(border_style), // .title(Span::styled("Chat Input".to_string(), HEADER_STYLE)),
    );
    frame.render_widget(widget, area);
}

fn render_users(state: &State, frame: &mut Frame, area: Rect) {
    let online: Vec<Line> = state
        .users
        .iter()
        .filter(|user| user.status == UserStatus::Online)
        .map(|user| Line::from(format!(" {}", user.name)))
        .collect();
    let mut lines: Vec<Line> = vec![Line::from(Span::styled(
        format!("Online - {}", online.len()),
        Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    ))];
    lines.extend(online);

    let offline: Vec<Line> = state
        .users
        .iter()
        .filter(|user| user.status == UserStatus::Offline)
        .map(|user| Line::from(Span::styled(format!(" {}", user.name), Style::default().add_modifier(Modifier::DIM))))
        .collect();
    lines.push(Line::from(Span::styled(
        format!("Offline - {}", offline.len()),
        Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    )));
    lines.extend(offline);

    let border_corners = border::Set {
        bottom_left: line::NORMAL.horizontal_up,
        bottom_right: line::NORMAL.vertical_left,
        ..border::PLAIN
    };

    let (borders, border_style, border_corners) = borders_users(state);

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

fn render_info(state: &State, frame: &mut Frame, area: Rect) {
    let info_text = match state.focus {
        Focus::Channels => "[↑↓] Change Channel | [→] Chat log | [L]ogs | [Q]uit",
        Focus::ChatHistory => "[↓] Input | [←] Channels | [→] Chat log | [L]ogs | [Q]uit",
        Focus::ChatInput(_) => {
            "[Enter] Send Message | [Backspace] Delete | [←→] Move Cursor | [Ctrl + ←→] Tab move Cursor | [↑] Chatlog | [L]ogs | [Q]uit"
        }
        Focus::Users => "[←] Chat log | [L]ogs | [Q]uit",
        Focus::Logs => "[L]ogs | [Q]uit",
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

fn render_logs(state: &State, frame: &mut Frame, area: Rect) {
    let current_log_count = state.logs.len();
    let start_index = current_log_count
        .saturating_sub(area.height.saturating_sub(2) as usize)
        .saturating_sub(state.logs_scroll_offset);

    let logs: Vec<Line> = state.logs.iter().skip(start_index).map(|entry| entry.format()).collect();

    let (borders, border_style, border_corners) = borders_logs(state);

    let widget = Paragraph::new(Text::from(logs)).wrap(Wrap { trim: true }).block(
        Block::default()
            .padding(PADDING)
            .border_set(border_corners)
            .borders(borders)
            .border_style(border_style)
            .title(Span::styled("Logs".to_string(), HEADER_STYLE)),
    );
    frame.render_widget(widget, area);
}
