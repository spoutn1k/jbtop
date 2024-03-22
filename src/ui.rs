use ratatui::{
    layout::Alignment,
    prelude::*,
    style::{Color, Style},
    widgets::*,
    Frame,
};

use crate::app::{App, HostState};

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui-org/ratatui/tree/master/examples

    let header = Row::new(vec!["host", "load"]);
    let widths = [Constraint::Percentage(25), Constraint::Fill(1)];

    let content: Vec<Row> = app
        .hosts
        .iter()
        .map(|(host, status)| match status {
            HostState::Connecting => Row::new(vec![
                Cell::from(host.as_str()).style(Style::default().fg(Color::Yellow)),
                Cell::from("Connecting ...").style(Style::default()),
            ]),
            HostState::Up(content) => Row::new(vec![
                Cell::from(host.as_str()).style(Style::default().fg(Color::Green)),
                Cell::from(content.as_str()).style(Style::default()),
            ]),
            HostState::Down(content) => Row::new(vec![
                Cell::from(host.as_str()).style(Style::default().fg(Color::Red)),
                Cell::from(content.as_str()).style(Style::default()),
            ]),
        })
        .collect();

    let load_table = Table::new(content, widths)
        .column_spacing(1)
        .header(header.style(Style::new().bold()));

    frame.render_widget(load_table, frame.size())
}
