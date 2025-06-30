use futures::{FutureExt, future::select};
use std::io;

use crate::{
    component::{
        component_helper::ComponentHelperExt, instantiated_component::InstantiatedComponent,
    },
    element::{ElementExt, key::ElementKey},
    props::AnyProps,
    render::drawer::ComponentDrawer,
    terminal::Terminal,
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

    pub fn render(&mut self, terminal: &mut Terminal) -> io::Result<()> {
        self.root_component.update(self.props.borrow(), terminal);

        terminal.draw(|frame| {
            let area = frame.area();
            let mut drawer = ComponentDrawer::new(frame, area);
            self.root_component.draw(&mut drawer);
        })?;

        Ok(())
    }

    pub async fn render_loop(&mut self) -> io::Result<()> {
        let mut terminal = Terminal::new();

        loop {
            // 渲染 UI
            self.render(&mut terminal)?;

            if terminal.received_ctrl_c() {
                break;
            }

            select(self.root_component.wait().boxed(), terminal.wait().boxed()).await;

            if terminal.received_ctrl_c() {
                break;
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
