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
            // 渲染 UI
            self.render(&mut terminal)?;

            // 等待组件树有状态变更（如 Hook、子组件等），避免无效刷新，提高性能
            self.root_component.wait().await;

            // 监听并处理用户输入事件
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
