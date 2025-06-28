use std::io;

use crossterm::event::{Event, EventStream, KeyCode, KeyModifiers};
use futures::StreamExt;

use crate::{
    component::{
        component_helper::ComponentHelperExt, instantiated_component::InstantiatedComponent,
    },
    element::{ElementExt, key::ElementKey},
    props::AnyProps,
    render::drawer::ComponentDrawer,
};

pub struct Tree<'a> {
    root_component: InstantiatedComponent,
    props: AnyProps<'a>,
}

impl<'a> Tree<'a> {
    pub fn new(mut props: AnyProps<'a>, helper: Box<dyn ComponentHelperExt>) -> Self {
        Self {
            root_component: InstantiatedComponent::new(
                ElementKey::new("__root__"),
                props.borrow(),
                helper,
            ),
            props,
        }
    }

    pub fn render(&mut self, terminal: &mut ratatui::DefaultTerminal) -> io::Result<()> {
        self.root_component.update(self.props.borrow());

        terminal.draw(|frame| {
            let area = frame.area();
            let mut drawer = ComponentDrawer::new(frame, area);
            self.root_component.draw(&mut drawer);
        })?;

        Ok(())
    }

    pub async fn render_loop(&mut self) -> io::Result<()> {
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

pub(crate) async fn render_loop<E: ElementExt>(mut element: E) -> io::Result<()> {
    let helper = element.helper();
    let mut tree = Tree::new(element.props_mut(), helper);

    tree.render_loop().await?;
    Ok(())
}
