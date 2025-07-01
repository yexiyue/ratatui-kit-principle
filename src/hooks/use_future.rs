use std::task::Poll;

use futures::future::BoxFuture;

use super::{Hook, Hooks};

/// 私有模块，防止外部实现 UseFuture trait
mod private {
    pub trait Sealed {}

    impl Sealed for crate::hooks::Hooks<'_, '_> {}
}

/// UseFuture trait：为 Hooks 扩展 use_future 方法
pub trait UseFuture: private::Sealed {
    /// 注册一个异步 Future，组件每次渲染时自动轮询
    fn use_future<F>(&mut self, f: F)
    where
        F: Future<Output = ()> + Send + 'static;
}

/// UseFutureImpl：包装 Future 的 Hook 实现
pub struct UseFutureImpl {
    f: Option<BoxFuture<'static, ()>>, // 存储待轮询的 Future
}

impl UseFutureImpl {
    /// 构造新的 UseFutureImpl，接收任意 Future
    pub fn new<F>(f: F) -> Self
    where
        F: Future<Output = ()> + Send + 'static,
    {
        UseFutureImpl {
            f: Some(Box::pin(f)),
        }
    }
}

/// 实现 Hook trait，支持异步轮询
impl Hook for UseFutureImpl {
    fn poll_change(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context,
    ) -> std::task::Poll<()> {
        // 轮询 Future，完成后清除
        if let Some(future) = self.f.as_mut() {
            if future.as_mut().poll(cx).is_ready() {
                self.f = None; // 清除已完成的 future
            }
        }
        Poll::Pending
    }
}

/// 为 Hooks 实现 UseFuture trait，便于在组件中直接调用 use_future
impl UseFuture for Hooks<'_, '_> {
    fn use_future<F>(&mut self, f: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.use_hook(move || UseFutureImpl::new(f));
    }
}
