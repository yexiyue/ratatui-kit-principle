use crate::render::{drawer::ComponentDrawer, updater::ComponentUpdater};
use std::{
    any::Any,
    pin::Pin,
    task::{Context, Poll},
};
pub mod use_events;
pub mod use_future;
pub mod use_state;

// Hook trait：所有 Hook 类型的基础接口，支持异步轮询
pub trait Hook: Unpin + Send {
    // 轮询 Hook 是否有变化，默认返回 Pending
    fn poll_change(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<()> {
        Poll::Pending
    }

    // 组件更新前的钩子，可用于副作用处理
    fn pre_component_update(&mut self, _updater: &mut ComponentUpdater) {}
    // 组件更新后的钩子
    fn post_component_update(&mut self, _updater: &mut ComponentUpdater) {}

    // 组件绘制前的钩子
    fn pre_component_draw(&mut self, _drawer: &mut ComponentDrawer) {}
    // 组件绘制后的钩子
    fn post_component_draw(&mut self, _drawer: &mut ComponentDrawer) {}
}

// AnyHook trait：用于类型擦除和运行时类型转换
pub trait AnyHook: Hook {
    // 获取可变引用，便于 downcast 到具体类型
    fn any_self_mut(&mut self) -> &mut dyn Any;
}

// 为所有实现 Hook 的类型自动实现 AnyHook
impl<T: Hook + 'static> AnyHook for T {
    fn any_self_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// 为 Hook 列表实现 Hook trait，便于批量管理和轮询所有 Hook
impl Hook for Vec<Box<dyn AnyHook>> {
    fn poll_change(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        let mut is_ready = false;

        // 遍历所有 Hook，只要有一个就绪则返回 Ready
        for hook in self.iter_mut() {
            if Pin::new(&mut **hook).poll_change(cx).is_ready() {
                is_ready = true;
            }
        }

        if is_ready {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }

    fn pre_component_update(&mut self, _updater: &mut ComponentUpdater) {
        for hook in self.iter_mut() {
            hook.pre_component_update(_updater);
        }
    }

    fn post_component_update(&mut self, _updater: &mut ComponentUpdater) {
        for hook in self.iter_mut() {
            hook.post_component_update(_updater);
        }
    }

    fn pre_component_draw(&mut self, _updater: &mut ComponentDrawer) {
        for hook in self.iter_mut() {
            hook.pre_component_draw(_updater);
        }
    }

    fn post_component_draw(&mut self, _updater: &mut ComponentDrawer) {
        for hook in self.iter_mut() {
            hook.post_component_draw(_updater);
        }
    }
}

// Hooks 结构体：管理组件中的所有 Hook 实例
pub struct Hooks<'a> {
    // 存储 Hook 的容器，生命周期与 Hooks 绑定
    hooks: &'a mut Vec<Box<dyn AnyHook>>,
    // 标记是否为首次更新（首次渲染）
    first_update: bool,
    // 当前 Hook 的索引，用于依次访问每个 Hook
    hook_index: usize,
}

impl<'a> Hooks<'a> {
    // 创建 Hooks 管理器
    pub fn new(hooks: &'a mut Vec<Box<dyn AnyHook>>, first_update: bool) -> Self {
        Self {
            hooks,
            first_update,
            hook_index: 0,
        }
    }

    // 注册或获取一个 Hook，类似 React 的 use_xxx 系列
    pub fn use_hook<F, H>(&mut self, f: F) -> &mut H
    where
        F: FnOnce() -> H,
        H: Hook + 'static,
    {
        // 首次渲染时创建 Hook 实例并存储
        if self.first_update {
            self.hooks.push(Box::new(f()));
        }
        let idx = self.hook_index;
        self.hook_index += 1;
        // 获取对应类型的 Hook，可变引用返回给调用者
        self.hooks
            .get_mut(idx)
            .and_then(|hook| hook.any_self_mut().downcast_mut::<H>())
            .expect("Hook type mismatch, ensure the hook is of the correct type")
    }
}
