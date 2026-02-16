use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Tabs, Wrap};
use ratatui::Frame;

use crate::app::App;
use crate::ui::TAB_TITLES;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(9),
            Constraint::Length(2),
        ])
        .split(frame.area());

    let tabs = Tabs::new(TAB_TITLES.iter().map(|t| Line::from(*t)).collect::<Vec<_>>())
        .select(app.selected_tab)
        .block(Block::default().borders(Borders::ALL).title("niri-cast"))
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(tabs, chunks[0]);

    let main_content = super::views::main_content(app);
    let main = Paragraph::new(main_content)
        .block(Block::default().borders(Borders::ALL).title("Control"))
        .wrap(Wrap { trim: true });
    frame.render_widget(main, chunks[1]);

    let log_lines = app
        .log_lines
        .iter()
        .rev()
        .take(8)
        .rev()
        .map(|l| Line::from(l.as_str()))
        .collect::<Vec<_>>();
    let logs = Paragraph::new(log_lines)
        .block(Block::default().borders(Borders::ALL).title("Logs"))
        .wrap(Wrap { trim: false });
    frame.render_widget(logs, chunks[2]);

    let footer = Paragraph::new(vec![Line::from(vec![
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(" quit  "),
        Span::styled("Tab", Style::default().fg(Color::Yellow)),
        Span::raw(" switch tab  "),
        Span::styled("r", Style::default().fg(Color::Yellow)),
        Span::raw(" refresh  "),
        Span::styled("d", Style::default().fg(Color::Yellow)),
        Span::raw(" diagnostics  "),
        Span::styled("c/e/w/v/h/u", Style::default().fg(Color::Yellow)),
        Span::raw(" preflight/extendR/extendL/mirror/hdmi-only/restore  "),
        Span::styled("m/a/s/l", Style::default().fg(Color::Yellow)),
        Span::raw(" outputs/audio/save/load"),
    ])]);
    frame.render_widget(footer, chunks[3]);
}
