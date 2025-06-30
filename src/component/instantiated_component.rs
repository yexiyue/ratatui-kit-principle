use futures::future::poll_fn;
use ratatui::layout::{Constraint, Direction};

use super::component_helper::ComponentHelperExt;
use crate::{
    component::AnyComponent,
    element::key::ElementKey,
    hooks::{AnyHook, Hook, Hooks},
    multimap::RemoveOnlyMultimap,
    props::AnyProps,
    render::{drawer::ComponentDrawer, layout_style::LayoutStyle, updater::ComponentUpdater},
    terminal::Terminal,
};
use std::{
    ops::{Deref, DerefMut},
    pin::Pin,
    task::{Context, Poll},
};

pub struct InstantiatedComponent {
    key: ElementKey,
    component: Box<dyn AnyComponent>,
    children: Components,
    helper: Box<dyn ComponentHelperExt>,
    layout_style: LayoutStyle,
    hooks: Vec<Box<dyn AnyHook>>,
    first_update: bool,
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
            hooks: Default::default(),
            first_update: true,
        }
    }

    /// 递归渲染当前组件及其所有子组件，自动处理布局和 Hook 生命周期
    pub fn draw(&mut self, drawer: &mut ComponentDrawer) {
        let layout_style = &self.layout_style;

        // 1. 计算应用 margin/offset 后的实际区域
        let area = layout_style.inner_area(drawer.area);
        drawer.area = area;
        // 渲染前调用所有 Hook 的 pre_component_draw 钩子
        self.hooks.pre_component_draw(drawer);
        // 2. 绘制当前组件内容
        self.component.draw(drawer);

        // 3. 计算所有子组件的区域划分（支持嵌套布局）
        let children_areas =
            self.component
                .calc_children_areas(&self.children, layout_style, drawer);

        // 4. 递归渲染所有子组件，每个子组件分配独立的区域
        for (child, child_area) in self.children.iter_mut().zip(children_areas) {
            drawer.area = child_area;
            child.draw(drawer);
        }
        // 渲染后调用所有 Hook 的 post_component_draw 钩子
        self.hooks.post_component_draw(drawer);
    }

    /// 更新当前组件及其子组件的状态，驱动 Hook 生命周期和属性变更
    pub fn update(&mut self, props: AnyProps, terminal: &mut Terminal) {
        // 构造组件更新辅助器，便于管理子组件和布局
        let mut updater = ComponentUpdater::new(
            self.key.clone(),
            &mut self.children,
            &mut self.layout_style,
            terminal,
        );

        // 更新前调用所有 Hook 的 pre_component_update 钩子
        self.hooks.pre_component_update(&mut updater);

        // 构造 Hooks 管理器，驱动本次 update 的所有 Hook 生命周期
        let hooks = Hooks::new(&mut self.hooks, self.first_update);

        // 调用组件的 update_component 方法，传递 props、hooks、updater
        self.helper
            .update_component(&mut self.component, props, hooks, &mut updater);

        // 更新后调用所有 Hook 的 post_component_update 钩子
        self.hooks.post_component_update(&mut updater);

        // 首次 update 标记为 false，后续渲染复用 Hook
        self.first_update = false;
    }

    pub fn component(&self) -> &dyn AnyComponent {
        &*self.component
    }

    /// 递归检查当前组件及其所有 Hook、子组件是否有状态变更需要刷新
    pub fn poll_change(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        // 先检查自身 hooks 是否有变化
        let hooks_status = Pin::new(&mut self.hooks).poll_change(cx);
        // 再检查所有子组件是否有变化
        let children_status = Pin::new(&mut self.children).poll_change(cx);

        // 只要有一个就绪，则整个组件需要刷新
        if hooks_status.is_ready() || children_status.is_ready() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }

    /// 异步等待，直到当前组件或其子组件有状态变更（常用于事件驱动的刷新）
    pub async fn wait(&mut self) {
        let mut self_mut = Pin::new(self);
        poll_fn(move |cx| self_mut.as_mut().poll_change(cx)).await;
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

    pub fn poll_change(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        let mut is_ready = false;

        for component in self.components.iter_mut() {
            if Pin::new(component).poll_change(cx).is_ready() {
                is_ready = true;
            }
        }

        if is_ready {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}
