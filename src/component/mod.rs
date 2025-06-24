use std::any::Any;

mod component_helper;
mod instantiated_component;

pub trait Component: Any {
    type Props<'a>
    where
        Self: 'a;

    fn new(props: &Self::Props<'_>) -> Self;

    fn draw(&self, _frame: &mut ratatui::Frame<'_>, _area: ratatui::layout::Rect) {}
}

pub trait AnyComponent {
    fn draw(&self, frame: &mut ratatui::Frame<'_>, area: ratatui::layout::Rect);
}

// 为所有实现了 Component trait 的类型自动实现 AnyComponent trait
impl<T> AnyComponent for T
where
    T: Component,
{
    /// 调用具体组件的 draw 方法，实现多态分发
    fn draw(&self, frame: &mut ratatui::Frame<'_>, area: ratatui::layout::Rect) {
        Component::draw(self, frame, area);
    }
}
