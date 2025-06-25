use ratatui::layout::{Constraint, Direction, Layout};

use super::component_helper::ComponentHelperExt;
use crate::{component::AnyComponent, props::AnyProps, render::layout_style::LayoutStyle};
use std::ops::{Deref, DerefMut};

pub struct InstantiatedComponent {
    component: Box<dyn AnyComponent>,
    children: Components,
    helper: Box<dyn ComponentHelperExt>,
    layout_style: LayoutStyle,
}

impl InstantiatedComponent {
    pub fn new(mut props: AnyProps, helper: Box<dyn ComponentHelperExt>) -> Self {
        let component = helper.new_component(props.borrow());

        Self {
            component,
            children: Components::default(),
            helper,
            layout_style: LayoutStyle::default(),
        }
    }

    /// 渲染当前组件及其子组件，自动进行布局区域划分
    ///
    /// 1. 先根据自身 layout_style 计算出当前组件的实际绘制区域（应用 margin/offset）
    /// 2. 绘制当前组件内容
    /// 3. 根据主轴方向，获取所有子组件的布局约束，生成主布局
    /// 4. 将主区域划分为多个子区域
    /// 5. 对每个子区域再按交叉轴方向进一步细分，实现嵌套布局
    /// 6. 递归调用每个子组件的 draw 方法，传入对应的区域
    pub fn draw(&self, frame: &mut ratatui::Frame<'_>, area: ratatui::layout::Rect) {
        let layout_style = &self.layout_style;
        // 1. 计算应用 margin/offset 后的实际区域
        let area = layout_style.inner_area(area);

        // 2. 绘制当前组件内容
        self.component.draw(frame, area);

        // 3. 构建主布局，按主轴方向分配子区域
        let layout = layout_style
            .get_layout()
            .constraints(self.children.get_constraints(layout_style.flex_direction));
        let areas = layout.split(area);

        let mut children_areas: Vec<ratatui::prelude::Rect> = vec![];

        // 4. 计算交叉轴方向（主轴为横则交叉轴为纵，反之亦然）
        let rev_direction = match layout_style.flex_direction {
            Direction::Horizontal => Direction::Vertical,
            Direction::Vertical => Direction::Horizontal,
        };

        // 5. 对每个主区域再按交叉轴方向细分，实现嵌套布局
        for (area, constraint) in areas
            .iter()
            .zip(self.children.get_constraints(rev_direction))
        {
            let area = Layout::new(rev_direction, [constraint]).split(*area)[0];
            children_areas.push(area);
        }

        // 6. 递归渲染所有子组件
        for (child, child_area) in self.children.iter().zip(children_areas) {
            child.draw(frame, child_area);
        }
    }
}

#[derive(Default)]
pub struct Components {
    pub components: Vec<InstantiatedComponent>,
}

impl Deref for Components {
    type Target = Vec<InstantiatedComponent>;

    fn deref(&self) -> &Self::Target {
        &self.components
    }
}

impl DerefMut for Components {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.components
    }
}

impl Components {
    /// 根据给定方向，收集所有子组件在该方向上的布局约束（Constraint）
    ///
    /// - 如果方向为 Horizontal，则收集每个子组件的宽度约束
    /// - 如果方向为 Vertical，则收集每个子组件的高度约束
    ///
    /// 这些约束用于 Ratatui 布局系统自动分配空间
    pub fn get_constraints(&self, direction: Direction) -> Vec<Constraint> {
        self.components
            .iter()
            .map(|c| match direction {
                Direction::Horizontal => c.layout_style.get_width(),
                Direction::Vertical => c.layout_style.get_height(),
            })
            .collect()
    }
}
