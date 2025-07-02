use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
// 引入 ratatui 相关模块
use ratatui::{
    layout::{Constraint, Direction, Flex, Margin, Offset},
    style::{Style, Stylize},
    widgets::{Block, Paragraph},
};
use ratatui_kit_macros::element;
// 引入 ratatui-kit-principle 组件系统相关模块
use ratatui_kit_principle::{
    component::Component,
    element::{AnyElement, Element, ElementExt, key::ElementKey},
    hooks::{self, use_events::UseEvents, use_state::UseState},
    render::{drawer::ComponentDrawer, layout_style::LayoutStyle},
};

use std::io;

// 文本组件，负责渲染一段文本
pub struct Text {
    pub text: String,
    pub style: Style,
    pub alignment: ratatui::layout::Alignment,
}

// 文本组件的 Props
#[derive(Default)]
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

    fn update(
        &mut self,
        props: &mut Self::Props<'_>,
        _hooks: hooks::Hooks,
        _updater: &mut ratatui_kit_principle::render::updater::ComponentUpdater<'_, '_>,
    ) {
        *self = Self {
            text: props.text.to_string(),
            style: props.style,
            alignment: props.alignment,
        };
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

    fn update(
        &mut self,
        props: &mut Self::Props<'_>,
        _hooks: hooks::Hooks,
        _updater: &mut ratatui_kit_principle::render::updater::ComponentUpdater<'_, '_>,
    ) {
        self.border_style = props.clone();
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

#[derive(Default)]
pub struct ViewProps<'a> {
    /// 主轴方向（横向/纵向）
    pub flex_direction: Direction,
    /// 主轴对齐方式（如 Start, End, Center, SpaceBetween 等）
    pub justify_content: Flex,
    /// 子项间距
    pub gap: i32,
    /// 外边距
    pub margin: Margin,
    /// 偏移量
    pub offset: Offset,
    /// 宽度约束
    pub width: Constraint,
    /// 高度约束
    pub height: Constraint,

    pub children: Vec<AnyElement<'a>>,
}

pub struct View;

impl Component for View {
    type Props<'a> = ViewProps<'a>;

    fn new(_props: &Self::Props<'_>) -> Self {
        Self
    }

    fn update(
        &mut self,
        props: &mut Self::Props<'_>,
        _hooks: hooks::Hooks,
        updater: &mut ratatui_kit_principle::render::updater::ComponentUpdater<'_, '_>,
    ) {
        updater.set_layout_style(LayoutStyle {
            flex_direction: props.flex_direction,
            justify_content: props.justify_content,
            gap: props.gap,
            margin: props.margin,
            offset: props.offset,
            width: props.width,
            height: props.height,
        });

        updater.update_children(props.children.iter_mut(), None);
    }
}

// 主程序入口，构建组件树并启动渲染循环
#[tokio::main]
async fn main() -> io::Result<()> {
    let mut element = Element::<Counter> {
        key: ElementKey::new("counter_app"),
        props: (),
    };

    element.render_loop().await?;
    Ok(())
}

pub struct Counter;

impl Component for Counter {
    type Props<'a> = ();
    fn new(_props: &Self::Props<'_>) -> Self {
        Counter
    }

    fn update(
        &mut self,
        _props: &mut Self::Props<'_>,
        mut hooks: hooks::Hooks,
        updater: &mut ratatui_kit_principle::render::updater::ComponentUpdater<'_, '_>,
    ) {
        let mut state = hooks.use_state(|| 0);

        hooks.use_events(move |event| match event {
            Event::Key(KeyEvent {
                kind: KeyEventKind::Press,
                code,
                ..
            }) => match code {
                KeyCode::Up => {
                    state.set(state.get() + 1);
                }
                KeyCode::Down => {
                    state.set(state.get() - 1);
                }
                _ => {}
            },
            _ => {}
        });

        let counter_text = format!("Count: {}", state.get());

        let element = element! {
            View(flex_direction: Direction::Vertical,gap: 3,){
                View(height: Constraint::Length(1),){
                    Text(
                        text: "Welcome to the Counter App", style: Style::default().bold().light_blue(), alignment: ratatui::layout::Alignment::Center)
                }
                View(height: Constraint::Fill(1),){
                    Text(text: counter_text.as_str(), style: Style::default().light_green(), alignment: ratatui::layout::Alignment::Center)
                }
                View(height: Constraint::Length(1),){
                    Text(text: "Press q or Ctrl+C to quit, + to increase, - to decrease", style: Style::default().yellow(), alignment: ratatui::layout::Alignment::Center)
                }
            }
        };
        // let element = Element::<View> {
        //     key: ElementKey::new("root"),
        //     props: ViewProps {
        //         children: vec![
        //             Element::<View> {
        //                 key: ElementKey::new("header"),
        //                 props: ViewProps {
        //                     children: vec![
        //                         Element::<Text> {
        //                             key: ElementKey::new("title"),
        //                             props: TextProps {
        //                                 text: "Welcome to the Counter App",
        //                                 style: Style::default().bold().light_blue(),
        //                                 alignment: ratatui::layout::Alignment::Center,
        //                             },
        //                         }
        //                         .into(),
        //                     ],
        //                     height: Constraint::Length(1),
        //                     ..Default::default()
        //                 },
        //             }
        //             .into(),
        //             Element::<View> {
        //                 key: ElementKey::new("body"),
        //                 props: ViewProps {
        //                     children: vec![
        //                         Element::<Text> {
        //                             key: ElementKey::new("number"),
        //                             props: TextProps {
        //                                 text: counter_text.as_str(),
        //                                 style: Style::default().light_green(),
        //                                 alignment: ratatui::layout::Alignment::Center,
        //                             },
        //                         }
        //                         .into(),
        //                     ],
        //                     height: Constraint::Fill(1),
        //                     ..Default::default()
        //                 },
        //             }
        //             .into(),
        //             Element::<View> {
        //                 key: ElementKey::new("footer"),
        //                 props: ViewProps {
        //                     children: vec![
        //                     Element::<Text> {
        //                         key: ElementKey::new("info"),
        //                         props: TextProps {
        //                             text: "Press q or Ctrl+C to quit, + to increase, - to decrease",
        //                             style: Style::default().yellow(),
        //                             alignment: ratatui::layout::Alignment::Center,
        //                         },
        //                     }
        //                     .into(),
        //                 ],
        //                     height: Constraint::Length(1),
        //                     ..Default::default()
        //                 },
        //             }
        //             .into(),
        //         ],
        //         flex_direction: Direction::Vertical,
        //         gap: 3,
        //         ..Default::default()
        //     },
        // };

        updater.update_children([element], None);
    }
}
