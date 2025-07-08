use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::symbols::{border, line};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Padding, Paragraph, Wrap};

use crate::tui::State;

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
    let (channels_area, chat_area, users_area) = split_channel_chat_user_areas(main_area);
    let (chat_log, chat_input) = split_chatlog_chatinput_areas(chat_area);

    // render_logs(state, frame, frame_area);
    render_channels(state, frame, channels_area);
    render_chat_log(state, frame, chat_log);
    render_chat_input(state, frame, chat_input);
    render_users(state, frame, users_area);
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
        .constraints([Constraint::Fill(10), Constraint::Length(6)])
        .split(area);
    (chunks[0], chunks[1])
}

fn render_channels(state: &State, frame: &mut Frame, area: Rect) {
    let border_style = Style::default();
    let widget = Paragraph::new(Text::from("test")).block(
        Block::default()
            .padding(PADDING)
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

    let border_style = Style::default();
    let widget = Paragraph::new(Text::from("test")).block(
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

    let border_style = Style::default();
    let widget = Paragraph::new(Text::from("test")).block(
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
    let border_style = Style::default();
    let widget = Paragraph::new(Text::from("test")).block(
        Block::default()
            .padding(PADDING)
            .borders(Borders::TOP | Borders::RIGHT | Borders::BOTTOM)
            .border_style(border_style)
            .title(Span::styled("Users".to_string(), HEADER_STYLE)),
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
            .padding(PADDING)
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(Span::styled(format!(" Log ({current_log_count})"), HEADER_STYLE)),
    );
    frame.render_widget(widget, area);
}
