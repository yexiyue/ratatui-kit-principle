use crossterm::event::{Event, EventStream, KeyCode, KeyModifiers};
use futures::StreamExt;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Borders, Paragraph},
};
use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let mut event_stream = EventStream::new();
    let mut count = 0;

    loop {
        terminal.draw(|f| {
            let area = f.area();

            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().blue());

            let inner_area = block.inner(area);

            let title = Line::from("Counter Application")
                .style(Style::default().bold().light_blue())
                .centered();

            let text = Paragraph::new(format!("Count: {}", count).light_green()).centered();

            let info = Line::from("Press q or Ctrl+C to quit, + to increase, - to decrease")
                .style(Style::default().yellow())
                .centered();

            let [top, body, bottom] = Layout::vertical([
                Constraint::Length(1),
                Constraint::Fill(1),
                Constraint::Length(1),
            ])
            .spacing(1)
            .areas(inner_area);
            f.render_widget(block, area);
            f.render_widget(title, top);
            f.render_widget(text, body);
            f.render_widget(info, bottom);
        })?;

        if let Some(Ok(event)) = event_stream.next().await {
            if let Event::Key(key) = event {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('+') => count += 1,
                    KeyCode::Char('-') => count -= 1,
                    KeyCode::Left => count -= 1,
                    KeyCode::Right => count += 1,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    _ => {}
                }
            }
        }
    }
    ratatui::restore();
    Ok(())
}
