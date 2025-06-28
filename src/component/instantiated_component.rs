use ratatui::layout::{Constraint, Direction};

use super::component_helper::ComponentHelperExt;
use crate::{
    component::AnyComponent,
    element::key::ElementKey,
    multimap::RemoveOnlyMultimap,
    props::AnyProps,
    render::{drawer::ComponentDrawer, layout_style::LayoutStyle, updater::ComponentUpdater},
};
use std::ops::{Deref, DerefMut};

pub struct InstantiatedComponent {
    key: ElementKey,
    component: Box<dyn AnyComponent>,
    children: Components,
    helper: Box<dyn ComponentHelperExt>,
    layout_style: LayoutStyle,
}

impl InstantiatedComponent {
    pub fn new(key: ElementKey, mut props: AnyProps, helper: Box<dyn ComponentHelperExt>) -> Self {
        let component = helper.new_component(props.borrow());

        Self {
            key,
            component,
            helper,
            children: Components::default(),
            layout_style: LayoutStyle::default(),
        }
    }

    pub fn draw(&self, drawer: &mut ComponentDrawer) {
        let layout_style = &self.layout_style;

        // 1. 计算应用 margin/offset 后的实际区域
        let area = layout_style.inner_area(drawer.area);
        drawer.area = area;

        // 2. 绘制当前组件内容
        self.component.draw(drawer);

        // 3. 计算所有子组件的区域划分
        let children_areas =
            self.component
                .calc_children_areas(&self.children, layout_style, drawer);

        // 4. 递归渲染所有子组件
        for (child, child_area) in self.children.iter().zip(children_areas) {
            drawer.area = child_area;
            child.draw(drawer);
        }
    }

    pub fn update(&mut self, props: AnyProps) {
        let mut updater =
            ComponentUpdater::new(self.key.clone(), &mut self.children, &mut self.layout_style);

        self.helper
            .update_component(&mut self.component, props, &mut updater);
    }

    pub fn component(&self) -> &dyn AnyComponent {
        &*self.component
    }
}

#[derive(Default)]
pub struct Components {
    pub components: RemoveOnlyMultimap<ElementKey, InstantiatedComponent>,
}

impl Deref for Components {
    type Target = RemoveOnlyMultimap<ElementKey, InstantiatedComponent>;

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
