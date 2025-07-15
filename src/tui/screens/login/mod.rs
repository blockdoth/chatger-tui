use std::fmt::format;
use std::fs;
use std::iter::repeat;

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::{border, line};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::tui::screens::chat::split_app_info_areas;
use crate::tui::{ChatFocus, GlobalState, InputStatus, LoginFocus, LoginState, State};

pub fn draw_login(global_state: &GlobalState, login_state: &LoginState, frame: &mut Frame) {
    let main_area = frame.area();
    let (form_area, info_area) = split_app_info_areas(global_state, main_area);
    let (login_area, background_area) = split_login_area_background(global_state, login_state, form_area);

    let background_area = if global_state.show_logs {
        let (background_area, logs_area) = split_background_log_areas(global_state, background_area);
        render_logs(global_state, frame, logs_area);
        background_area
    } else {
        background_area
    };
    render_background(global_state, login_state, frame, background_area);

    render_login(global_state, login_state, frame, login_area);
    render_info(frame, info_area);
}

fn split_background_log_areas(global_state: &GlobalState, area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);
    (chunks[0], chunks[1])
}

fn split_login_area_background(global_state: &GlobalState, login_state: &LoginState, area: Rect) -> (Rect, Rect) {
    let [horizontally_centered] = Layout::horizontal([Constraint::Percentage(15)]).flex(Flex::Center).areas(area);
    let [centered] = Layout::vertical([Constraint::Length(16)]).flex(Flex::Center).areas(horizontally_centered);
    (centered, area)
}

pub enum LineSelected {
    Username,
    Password,
    ServerAddress,
}

fn input_line(login_state: &'_ LoginState, line_selected: LineSelected, input_length: usize) -> Vec<Span<'_>> {
    let (input, focus_index) = match line_selected {
        LineSelected::Username => (
            &login_state.username_input,
            if let LoginFocus::UsernameInput(idx) = login_state.focus {
                idx
            } else {
                usize::MAX
            },
        ),
        LineSelected::Password => (
            &login_state.password_input,
            if let LoginFocus::PasswordInput(idx) = login_state.focus {
                idx
            } else {
                usize::MAX
            },
        ),
        LineSelected::ServerAddress => (
            &login_state.server_address_input,
            if let LoginFocus::ServerAddressInput(idx) = login_state.focus {
                idx
            } else {
                usize::MAX
            },
        ),
    };

    let mut selected_style = match (&line_selected, &login_state.focus) {
        (LineSelected::Username, LoginFocus::UsernameInput(_)) => Style::default().fg(Color::Cyan),
        (LineSelected::Password, LoginFocus::PasswordInput(_)) => Style::default().fg(Color::Cyan),
        (LineSelected::Username, _) if login_state.input_status == InputStatus::IncorrectUsernameOrPassword => Style::default().fg(Color::Red),
        (LineSelected::Password, _) if login_state.input_status == InputStatus::IncorrectUsernameOrPassword => Style::default().fg(Color::Red),
        (LineSelected::ServerAddress, _) if login_state.input_status == InputStatus::AddressNotParsable => Style::default().fg(Color::Red),
        (LineSelected::ServerAddress, LoginFocus::ServerAddressInput(_)) => Style::default().fg(Color::Cyan),
        _ => Style::default(),
    };
    selected_style = selected_style.add_modifier(Modifier::UNDERLINED);

    let mut spans: Vec<Span> = input
        .chars()
        .enumerate()
        .map(|(idx, c)| {
            if idx == focus_index {
                Span::styled(c.to_string(), selected_style.add_modifier(Modifier::DIM))
            } else {
                Span::styled(c.to_string(), selected_style)
            }
        })
        .collect();

    let current_len = spans.len();
    if current_len < input_length {
        let padding = " ".repeat(input_length - current_len);
        spans.push(Span::styled(padding, selected_style));
    }

    spans
}

fn render_login(global_state: &GlobalState, login_state: &LoginState, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(12), Constraint::Length(2)])
        .split(area);
    let (login_title_area, login_form_area, login_button_area) = (chunks[0], chunks[1], chunks[2]);

    let obscured_password = "*".repeat(login_state.password_input.chars().filter(|c| *c != ' ').count());

    let side_padding_len = 2;

    let input_length = login_form_area.width.saturating_sub(3 * side_padding_len) as usize;

    let username_input = input_line(login_state, LineSelected::Username, input_length);
    let password_input = input_line(login_state, LineSelected::Password, input_length);
    let server_input = input_line(login_state, LineSelected::ServerAddress, input_length);

    let side_padding = " ".repeat(side_padding_len as usize);

    let error_message = match &login_state.input_status {
        InputStatus::AllFine => Span::raw(""),
        status => Span::styled(format!("  {status:?}"), Modifier::ITALIC),
    };

    let lines = Text::from(vec![
        Line::from(vec![Span::styled(
            " Username",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )]),
        Line::from({
            let mut spans = Vec::new();
            spans.push(Span::raw(&side_padding));
            spans.extend(username_input.into_iter());
            spans.push(Span::raw(&side_padding));
            spans
        }),
        Line::from(""),
        Line::from(vec![Span::styled(
            " Password",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )]),
        Line::from({
            let mut spans = Vec::new();
            spans.push(Span::raw(&side_padding));
            spans.extend(password_input.into_iter());
            spans.push(Span::raw(&side_padding));
            spans
        }),
        Line::from(""),
        Line::from(vec![Span::styled(
            " Server Address",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )]),
        Line::from({
            let mut spans = Vec::new();
            spans.push(Span::raw(&side_padding));
            spans.extend(server_input.into_iter());
            spans.push(Span::raw(&side_padding));
            spans
        }),
        Line::from(error_message),
    ]);

    let login_button_style = if LoginFocus::Login == login_state.focus {
        if InputStatus::AllFine == login_state.input_status {
            Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD)
        } else {
            Style::default().bg(Color::Red).fg(Color::Black).add_modifier(Modifier::BOLD)
        }
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    };

    let title_block = Paragraph::new(Text::from(Span::styled(
        "Welcome to Chatger!",
        Style::default().add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
            .border_style(Style::default())
            .style(Style::default()),
    );

    let form_block = Paragraph::new(lines).block(
        Block::default()
            .style(Style::default())
            .border_set(border::Set {
                bottom_right: line::NORMAL.vertical_left,
                bottom_left: line::NORMAL.vertical_right,
                top_right: line::NORMAL.vertical_left,
                top_left: line::NORMAL.vertical_right,
                ..border::PLAIN
            })
            .borders(Borders::ALL)
            .border_style(Style::default()),
    );

    let login_block = Paragraph::new(Span::styled(" Login ", login_button_style))
        .block(
            Block::default()
                .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                .border_style(Style::default()),
        )
        .alignment(Alignment::Center);

    frame.render_widget(title_block, login_title_area);
    frame.render_widget(form_block, login_form_area);
    frame.render_widget(login_block, login_button_area);
}

const PENGER_FILE_PATH: &str = "./assets/penger.txt"; // TODO hardcode in a better way
const PENGER_TITLE_FILE_PATH: &str = "./assets/penger_title.txt";

fn render_background(global_state: &GlobalState, login_state: &LoginState, frame: &mut Frame, area: Rect) {
    let penger = match fs::read_to_string(PENGER_FILE_PATH) {
        Ok(penger) => penger,
        Err(e) => format!("{e}"),
    };
    let penger_title = match fs::read_to_string(PENGER_TITLE_FILE_PATH) {
        Ok(penger) => penger,
        Err(e) => format!("{e}"),
    };

    let penger_lines: Vec<&str> = penger.lines().collect();
    let title_lines: Vec<&str> = penger_title.lines().collect();

    let max_lines = penger_lines.len().max(title_lines.len());

    let padded_penger = penger_lines.iter().cloned().chain(repeat("")).take(max_lines);
    let padded_title = title_lines.iter().cloned().chain(repeat("")).take(max_lines);

    let merged_lines: Vec<String> = padded_penger
        .clone()
        .zip(padded_title)
        .map(|(left, right)| format!("{left:<30}{right}"))
        .zip(padded_penger)
        .map(|(left, right)| format!("{left:<30}{right}"))
        .collect();

    let merged_text = merged_lines.join("\n");

    let widget = Paragraph::new(merged_text).alignment(Alignment::Center);
    frame.render_widget(widget, area);
}

fn render_info(frame: &mut Frame, area: Rect) {
    let info_text =
        "[Enter] Login | [Backspace] Delete | [←→] Move Cursor | [Ctrl + ←→] Tab move Cursor | [↑↓] Move Field | [L]ogs | [Q]uit".to_owned();

    let widget = Paragraph::new(Text::from(info_text)).alignment(Alignment::Center);

    frame.render_widget(widget, area);
}

fn render_logs(global_state: &GlobalState, frame: &mut Frame, area: Rect) {
    let current_log_count = global_state.logs.len();
    let start_index = current_log_count
        .saturating_sub(area.height.saturating_sub(2) as usize)
        .saturating_sub(global_state.log_scroll_offset);

    let logs: Vec<Line> = global_state.logs.iter().skip(start_index).map(|entry| entry.format()).collect();

    let widget = Paragraph::new(Text::from(logs)).wrap(Wrap { trim: true });
    frame.render_widget(widget, area);
}
