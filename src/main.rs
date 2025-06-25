// 引入 ratatui 相关模块
use ratatui::{
    layout::{Constraint, Direction},
    style::{Style, Stylize},
    widgets::{Block, Paragraph},
};
// 引入 ratatui-kit-principle 组件系统相关模块
use ratatui_kit_principle::{
    component::{
        Component,
        component_helper::ComponentHelper,
        instantiated_component::{Components, InstantiatedComponent},
    },
    props::AnyProps,
    render::{drawer::ComponentDrawer, layout_style::LayoutStyle, tree::Tree},
};

use std::io;

// 文本组件，负责渲染一段文本
pub struct Text {
    pub text: String,
    pub style: Style,
    pub alignment: ratatui::layout::Alignment,
}

// 文本组件的 Props
pub struct TextProps<'a> {
    pub text: &'a str,
    pub style: Style,
    pub alignment: ratatui::layout::Alignment,
}

// Text 组件实现 Component 协议
impl Component for Text {
    type Props<'a> = TextProps<'a>;

    fn new(props: &Self::Props<'_>) -> Self {
        Self {
            text: props.text.to_string(),
            style: props.style,
            alignment: props.alignment,
        }
    }

    fn draw(&self, drawer: &mut ComponentDrawer<'_, '_>) {
        // 渲染段落文本
        let paragraph = Paragraph::new(self.text.clone())
            .style(self.style)
            .alignment(self.alignment);
        drawer.render_widget(paragraph, drawer.area);
    }
}

// 边框组件，负责为内容添加边框
pub struct Border {
    pub border_style: Style,
}

// Border 组件实现 Component 协议
impl Component for Border {
    type Props<'a> = Style;

    fn new(props: &Self::Props<'_>) -> Self {
        Self {
            border_style: props.clone(),
        }
    }

    fn draw(&self, drawer: &mut ComponentDrawer<'_, '_>) {
        // 绘制带样式的边框
        let block = Block::bordered().border_style(self.border_style);
        let inner_area = block.inner(drawer.area);

        drawer.render_widget(block, drawer.area);

        // 更新 drawer 的可用区域为边框内部
        drawer.area = inner_area;
    }
}

// 主程序入口，构建组件树并启动渲染循环
#[tokio::main]
async fn main() -> io::Result<()> {
    let count = 0;

    // 构建根组件（带边框的容器），并嵌套多个子组件
    let instantiated_component = InstantiatedComponent::new(
        AnyProps::owned(Style::default().blue()),
        ComponentHelper::<Border>::boxed(),
        LayoutStyle {
            gap: 3,
            flex_direction: Direction::Vertical,
            ..Default::default()
        },
        Components {
            components: vec![
                // 标题文本
                InstantiatedComponent::new(
                    AnyProps::owned(TextProps {
                        text: "Welcome to the Counter App",
                        style: Style::default().bold().light_blue(),
                        alignment: ratatui::layout::Alignment::Center,
                    }),
                    ComponentHelper::<Text>::boxed(),
                    LayoutStyle {
                        height: Constraint::Length(1),
                        ..Default::default()
                    },
                    Components::default(),
                ),
                // 计数显示
                InstantiatedComponent::new(
                    AnyProps::owned(TextProps {
                        text: &format!("Count: {}", count),
                        style: Style::default().light_green(),
                        alignment: ratatui::layout::Alignment::Center,
                    }),
                    ComponentHelper::<Text>::boxed(),
                    LayoutStyle {
                        height: Constraint::Fill(1),
                        ..Default::default()
                    },
                    Components::default(),
                ),
                // 操作提示
                InstantiatedComponent::new(
                    AnyProps::owned(TextProps {
                        text: "Press q or Ctrl+C to quit, + to increase, - to decrease",
                        style: Style::default().yellow(),
                        alignment: ratatui::layout::Alignment::Center,
                    }),
                    ComponentHelper::<Text>::boxed(),
                    LayoutStyle {
                        height: Constraint::Length(1),
                        ..Default::default()
                    },
                    Components::default(),
                ),
            ],
        },
    );

    // 启动组件树的渲染主循环
    Tree::new(instantiated_component).render_loop().await?;
    Ok(())
}
