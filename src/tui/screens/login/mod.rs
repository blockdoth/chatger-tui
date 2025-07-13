use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, Paragraph};
use std::fmt::format;
use std::fs;

use crate::tui::State;

pub fn draw_login(state: &State, frame: &mut Frame) {
    let main_area = frame.area();

    let (login_area, background_area) = split_login_area_background(state, main_area);

    render_background(state, frame, background_area);
    render_login(state, frame, login_area);
}

fn split_login_area_background(state: &State, area: Rect) -> (Rect, Rect) {
    let [horizontally_centered] = Layout::horizontal([Constraint::Percentage(20)]).flex(Flex::Center).areas(area);
    let [centered] = Layout::vertical([Constraint::Length(8)]).flex(Flex::Center).areas(horizontally_centered);
    (centered, area)
}

fn render_login(state: &State, frame: &mut Frame, area: Rect) {
    
    let text = Text::raw("Hello world!");

    let widget = Paragraph::new(text).block(
        Block::default().style(Style::default().bg(Color::Black)).borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)), // .title(Span::styled("Channels".to_string(), HEADER_STYLE)),
    );
    frame.render_widget(widget, area);
}

const PENGER_FILE_PATH:&str = "./assets/penger.txt"; // TODO hardcode in a better way

fn render_background(state: &State, frame: &mut Frame, area: Rect) {

    let penger = match fs::read_to_string(PENGER_FILE_PATH) {
        Ok(penger) => penger,
        Err(e) => format!("{e}"),
    };

  let widget =  Paragraph::new(penger).block(Block::default().style(Style::default().fg(Color::Gray))); 
  frame.render_widget(widget, area);

}