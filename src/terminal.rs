// 引入终端事件相关依赖
use crossterm::event::{Event, EventStream, KeyCode, KeyModifiers};
use futures::{Stream, StreamExt};
use std::{
    collections::VecDeque, // 用于存储事件队列
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex, Weak},
    task::{Poll, Waker},
};

// 事件队列和唤醒器的内部结构
pub struct TerminalEventsInner {
    pending: VecDeque<Event>, // 待处理的事件队列
    waker: Option<Waker>,     // 用于异步唤醒的 Waker
}

// 终端事件流，支持异步 Stream 读取事件
pub struct TerminalEvents {
    inner: Arc<Mutex<TerminalEventsInner>>, // 共享内部状态
}

// 实现 Stream trait，使 TerminalEvents 可被异步轮询
impl Stream for TerminalEvents {
    type Item = Event;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut inner = self.inner.lock().unwrap();

        // 如果有事件，直接返回
        if let Some(event) = inner.pending.pop_front() {
            Poll::Ready(Some(event))
        } else {
            // 没有事件则注册 waker，等待唤醒
            inner.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

// 封装终端，负责事件分发和订阅
pub struct Terminal {
    inner: ratatui::DefaultTerminal,                    // 终端渲染对象
    event_stream: EventStream,                          // crossterm 事件流
    subscribers: Vec<Weak<Mutex<TerminalEventsInner>>>, // 事件订阅者列表
    received_ctrl_c: bool,                              // 是否收到 Ctrl+C
}

// 允许像操作 ratatui::DefaultTerminal 一样操作 Terminal
impl Deref for Terminal {
    type Target = ratatui::DefaultTerminal;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Terminal {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Terminal {
    // 创建新的终端对象
    pub fn new() -> Self {
        Terminal {
            inner: ratatui::init(),
            event_stream: EventStream::new(),
            subscribers: Vec::new(),
            received_ctrl_c: false,
        }
    }

    // 查询是否收到 Ctrl+C
    pub fn received_ctrl_c(&self) -> bool {
        self.received_ctrl_c
    }

    // 创建一个事件订阅流，供组件异步消费事件
    pub fn events(&mut self) -> TerminalEvents {
        let inner = Arc::new(Mutex::new(TerminalEventsInner {
            pending: VecDeque::new(),
            waker: None,
        }));

        // 订阅者弱引用加入列表
        self.subscribers.push(Arc::downgrade(&inner));

        TerminalEvents { inner }
    }

    // 异步事件主循环，将事件分发给所有订阅者
    pub async fn wait(&mut self) {
        while let Some(Ok(event)) = self.event_stream.next().await {
            // 检测 Ctrl+C 事件
            if let Event::Key(key) = event {
                if matches!(key.code, KeyCode::Char('c'))
                    && key.modifiers.contains(KeyModifiers::CONTROL)
                {
                    self.received_ctrl_c = true;
                    return;
                }
            }

            // 分发事件到所有订阅者
            self.subscribers.retain(|subscriber| {
                if let Some(inner) = subscriber.upgrade() {
                    let mut subscriber = inner.lock().unwrap();
                    subscriber.pending.push_back(event.clone());

                    // 唤醒等待事件的 waker
                    if let Some(waker) = subscriber.waker.take() {
                        waker.wake();
                    }

                    true
                } else {
                    // 订阅者已被释放则移除
                    false
                }
            });
        }
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        // 在终端被销毁时恢复原始终端状态
        ratatui::restore();
    }
}
