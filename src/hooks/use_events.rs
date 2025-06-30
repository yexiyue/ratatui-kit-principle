use crate::{
    hooks::{Hook, Hooks},
    render::{drawer::ComponentDrawer, updater::ComponentUpdater},
    terminal::TerminalEvents,
};
use crossterm::event::Event;
use futures::Stream;
use ratatui::layout::Rect;
use std::{pin::pin, task::Poll};

// 私有 trait，用于防止外部实现 UseEvents
mod private {
    pub trait Sealed {}
    impl Sealed for crate::hooks::Hooks<'_> {}
}

// UseEvents trait：为 Hooks 提供 use_events 和 use_local_events 两种事件 Hook
pub trait UseEvents: private::Sealed {
    /// 订阅全局终端事件，所有事件都会回调 f
    fn use_events<F>(&mut self, f: F)
    where
        F: FnMut(Event) + Send + 'static;

    /// 只处理当前组件区域内的事件（如鼠标事件），回调 f
    fn use_local_events<F>(&mut self, f: F)
    where
        F: FnMut(Event) + Send + 'static;
}

// Hooks 实现 UseEvents trait
impl UseEvents for Hooks<'_> {
    fn use_events<F>(&mut self, f: F)
    where
        F: FnMut(Event) + Send + 'static,
    {
        // 注册全局事件 Hook
        let h = self.use_hook(move || UseEventsImpl {
            events: None,
            component_area: Default::default(),
            in_component: false,
            f: None,
        });
        h.f = Some(Box::new(f));
    }

    fn use_local_events<F>(&mut self, f: F)
    where
        F: FnMut(Event) + Send + 'static,
    {
        // 注册局部事件 Hook（只处理组件区域内事件）
        let h = self.use_hook(move || UseEventsImpl {
            events: None,
            component_area: Default::default(),
            in_component: true,
            f: None,
        });
        h.f = Some(Box::new(f));
    }
}

// 事件 Hook 的具体实现体
struct UseEventsImpl {
    f: Option<Box<dyn FnMut(Event) + Send>>, // 事件回调闭包
    events: Option<TerminalEvents>,          // 事件流
    in_component: bool,                      // 是否只处理组件区域内事件
    component_area: Rect,                    // 组件区域
}

// 实现 Hook trait，使事件 Hook 能参与组件生命周期
impl Hook for UseEventsImpl {
    fn poll_change(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context,
    ) -> std::task::Poll<()> {
        // 轮询事件流，处理所有就绪事件
        while let Some(Poll::Ready(Some(event))) = self
            .events
            .as_mut()
            .map(|events| pin!(events).poll_next(cx))
        {
            let area = self.component_area;
            let in_component = self.in_component;
            if let Some(f) = &mut self.f {
                if in_component {
                    // 只处理组件区域内的鼠标事件
                    match event {
                        Event::Mouse(mouse_event) => {
                            if mouse_event.row >= area.y && mouse_event.column >= area.x {
                                let row = mouse_event.row - area.y;
                                let column = mouse_event.column - area.x;
                                if row < area.height && column < area.width {
                                    f(event);
                                }
                            }
                        }
                        _ => {
                            f(event);
                        }
                    }
                } else {
                    // 全局事件直接回调
                    f(event);
                }
            }
        }
        Poll::Pending
    }

    fn post_component_update(&mut self, updater: &mut ComponentUpdater) {
        // 组件 update 后，首次初始化事件流
        if self.events.is_none() {
            self.events = Some(updater.terminal().events());
        }
    }

    fn pre_component_draw(&mut self, drawer: &mut ComponentDrawer) {
        // 绘制前记录当前组件区域，用于局部事件判断
        self.component_area = drawer.area;
    }
}
