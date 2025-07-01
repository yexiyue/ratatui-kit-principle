use std::{
    any::Any,
    cell::{Ref, RefMut},
};

use super::Hooks;

// 私有模块用于防止 trait 被外部实现，保证 API 封装性
mod private {
    pub trait Sealed {}

    impl Sealed for crate::hooks::Hooks<'_, '_> {}
}

/// UseContext trait 提供在 Hook 中访问上下文（ContextStack）的方法，
/// 支持只读和可变引用的获取，便于组件间共享全局或局部数据。
pub trait UseContext<'a>: private::Sealed {
    /// 获取类型为 T 的只读上下文引用，找不到会 panic。
    fn use_context<T: Any>(&self) -> Ref<'a, T>;
    /// 获取类型为 T 的可变上下文引用，找不到会 panic。
    fn use_context_mut<T: Any>(&self) -> RefMut<'a, T>;

    /// 尝试获取类型为 T 的只读上下文引用，找不到返回 None。
    fn try_use_context<T: Any>(&self) -> Option<Ref<'a, T>>;
    /// 尝试获取类型为 T 的可变上下文引用，找不到返回 None。
    fn try_use_context_mut<T: Any>(&self) -> Option<RefMut<'a, T>>;
}

impl<'a> UseContext<'a> for Hooks<'a, '_> {
    /// 获取类型为 T 的只读上下文引用，若不存在则 panic。
    fn use_context<T: Any>(&self) -> Ref<'a, T> {
        self.context
            .expect("context not available")
            .get_context()
            .expect("context not found")
    }

    /// 获取类型为 T 的可变上下文引用，若不存在则 panic。
    fn use_context_mut<T: Any>(&self) -> RefMut<'a, T> {
        self.context
            .expect("context not available")
            .get_context_mut()
            .expect("context not found")
    }

    /// 尝试获取类型为 T 的只读上下文引用，找不到返回 None。
    fn try_use_context<T: Any>(&self) -> Option<Ref<'a, T>> {
        self.context
            .and_then(|context_stack| context_stack.get_context())
    }

    /// 尝试获取类型为 T 的可变上下文引用，找不到返回 None。
    fn try_use_context_mut<T: Any>(&self) -> Option<RefMut<'a, T>> {
        self.context
            .and_then(|context_stack| context_stack.get_context_mut())
    }
}
