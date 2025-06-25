use std::any::Any;

use ratatui::layout::{Direction, Layout};

use crate::{
    component::instantiated_component::Components,
    render::{drawer::ComponentDrawer, layout_style::LayoutStyle},
};

mod component_helper;
mod instantiated_component;

pub trait Component: Any {
    type Props<'a>
    where
        Self: 'a;

    fn new(props: &Self::Props<'_>) -> Self;

    fn draw(&self, drawer: &mut ComponentDrawer<'_, '_>) {}

    // 默认使用flex布局计算子组件的area
    fn calc_children_areas(
        &self,
        children: &Components,
        layout_style: &LayoutStyle,
        drawer: &mut ComponentDrawer<'_, '_>,
    ) -> Vec<ratatui::prelude::Rect> {
        let layout = layout_style
            .get_layout()
            .constraints(children.get_constraints(layout_style.flex_direction));

        let areas = layout.split(drawer.area);

        let mut children_areas: Vec<ratatui::prelude::Rect> = vec![];

        let rev_direction = match layout_style.flex_direction {
            Direction::Horizontal => Direction::Vertical,
            Direction::Vertical => Direction::Horizontal,
        };
        for (area, constraint) in areas.iter().zip(children.get_constraints(rev_direction)) {
            let area = Layout::new(rev_direction, [constraint]).split(*area)[0];
            children_areas.push(area);
        }

        children_areas
    }
}

pub trait AnyComponent {
    fn draw(&self, drawer: &mut ComponentDrawer<'_, '_>);

    fn calc_children_areas(
        &self,
        children: &Components,
        layout_style: &LayoutStyle,
        drawer: &mut ComponentDrawer<'_, '_>,
    ) -> Vec<ratatui::prelude::Rect>;
}

// 为所有实现了 Component trait 的类型自动实现 AnyComponent trait
impl<T> AnyComponent for T
where
    T: Component,
{
    /// 调用具体组件的 draw 方法，实现多态分发
    fn draw(&self, drawer: &mut ComponentDrawer<'_, '_>) {
        Component::draw(self, drawer);
    }

    fn calc_children_areas(
        &self,
        children: &Components,
        layout_style: &LayoutStyle,
        drawer: &mut ComponentDrawer<'_, '_>,
    ) -> Vec<ratatui::prelude::Rect> {
        Component::calc_children_areas(self, children, layout_style, drawer)
    }
}
