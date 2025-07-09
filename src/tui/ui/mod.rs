use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::{border, line};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Padding, Paragraph, Wrap};

use crate::tui::chat::{ChannelStatus, UserStatus};
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
    let (app_area, info_area) = split_app_info(main_area);
    let (channels_area, chat_area, users_area) = split_channel_chat_user_areas(app_area);
    let (chat_log, chat_input) = split_chatlog_chatinput_areas(chat_area);

    // render_logs(state, frame, frame_area);
    render_channels(state, frame, channels_area);
    render_chat_log(state, frame, chat_log);
    render_chat_input(state, frame, chat_input);
    render_users(state, frame, users_area);
    render_info(state, frame, info_area);
}

fn split_app_info(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Fill(10), Constraint::Length(2)])
        .split(area);
    (chunks[0], chunks[1])
}

fn split_channel_chat_user_areas(area: Rect) -> (Rect, Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints([Constraint::Min(20), Constraint::Fill(10), Constraint::Min(20)])
        .split(area);
    (chunks[0], chunks[1], chunks[2])
}

fn split_chatlog_chatinput_areas(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Fill(10), Constraint::Length(5)])
        .split(area);
    (chunks[0], chunks[1])
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
            if channel.id == state.active_channel {
                style = style.bg(Color::DarkGray);
            }

            Line::from(Span::styled(format!("# {:15}", channel.name.clone()), style))
        })
        .collect();
    let border_corners = border::Set {
        bottom_left: line::NORMAL.vertical_right,
        ..border::PLAIN
    };

    let border_style = Style::default();
    let widget = Paragraph::new(Text::from(channels)).block(
        Block::default()
            .padding(PADDING)
            .border_set(border_corners)
            .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
            .border_style(border_style)
            .title(Span::styled("Channels".to_string(), HEADER_STYLE)),
    );
    frame.render_widget(widget, area);
}

fn render_chat_log(state: &State, frame: &mut Frame, area: Rect) {
    let border_corners = border::Set {
        top_left: line::NORMAL.horizontal_down,
        top_right: line::NORMAL.horizontal_down,
        ..border::PLAIN
    };

    let chatlog: Vec<Line> = state
        .chat_history
        .get(&state.active_channel)
        .unwrap_or(&vec![])
        .iter()
        .flat_map(|chat_message| {
            let timestamp = chat_message.timestamp.format("%H:%M:%S").to_string();

            let header = Line::from(vec![
                Span::styled(
                    format!("{} ", &chat_message.author.name),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!(" [{timestamp}] "), Style::default().fg(Color::DarkGray)),
            ]);

            let body = Line::from(Span::raw(format!("\t{}", &chat_message.message)));

            vec![header, body].into_iter()
        })
        .collect();

    let border_style = Style::default();
    let widget = Paragraph::new(Text::from(chatlog)).wrap(Wrap { trim: false }).block(
        Block::default()
            .padding(PADDING)
            .border_set(border_corners)
            .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
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

    let input = vec![Line::from(Span::from("")), Line::from(Span::from(state.chat_input.clone().to_string()))];
    let border_style = Style::default();
    let widget = Paragraph::new(Text::from(input)).block(
        Block::default()
            .padding(PADDING)
            .border_set(border_corners)
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(Span::styled("Chat Input".to_string(), HEADER_STYLE)),
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

    let border_style = Style::default();
    let widget = Paragraph::new(Text::from(lines)).block(
        Block::default()
            .padding(PADDING)
            .border_set(border_corners)
            .borders(Borders::TOP | Borders::RIGHT | Borders::BOTTOM)
            .border_style(border_style)
            .title(Span::styled("Users".to_string(), HEADER_STYLE)),
    );
    frame.render_widget(widget, area);
}

fn render_info(state: &State, frame: &mut Frame, area: Rect) {
    let info_text = match state.focus {
        Focus::Channels => "[↑↓] Move | [L]ogs | [Q]uit",
        Focus::ChatHistory => "[L]ogs | [Q]uit",
        Focus::Input => "[L]ogs | [Q]uit",
        Focus::Users => "[L]ogs | [Q]uit",
        Focus::None => "[C]hannels | [Space] Input | [U]sers | [L]ogs | [Q]uit",
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

    let border_style = Style::default();
    let widget = Paragraph::new(Text::from(logs)).wrap(Wrap { trim: true }).block(
        Block::default()
            // .padding(PADDING)
            .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
            .border_style(border_style)
            .title(Span::styled(format!(" Log ({current_log_count})"), HEADER_STYLE)),
    );
    frame.render_widget(widget, area);
}
