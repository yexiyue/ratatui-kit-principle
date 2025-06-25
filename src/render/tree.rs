use std::io;

use crossterm::event::{Event, EventStream, KeyCode, KeyModifiers};
use futures::StreamExt;

use crate::{
    component::instantiated_component::InstantiatedComponent, render::drawer::ComponentDrawer,
};

pub struct Tree {
    root_component: InstantiatedComponent,
}

impl Tree {
    pub fn new(root_component: InstantiatedComponent) -> Self {
        Self { root_component }
    }

    pub fn render(&self, terminal: &mut ratatui::DefaultTerminal) -> io::Result<()> {
        terminal.draw(|frame| {
            let area = frame.area();
            let mut drawer = ComponentDrawer::new(frame, area);
            self.root_component.draw(&mut drawer);
        })?;

        Ok(())
    }

    pub async fn render_loop(&self) -> io::Result<()> {
        let mut terminal = ratatui::init();
        let mut event_stream = EventStream::new();
        loop {
            self.render(&mut terminal)?;

            if let Some(Ok(event)) = event_stream.next().await {
                if let Event::Key(key) = event {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
        ratatui::restore();
        Ok(())
    }
}
