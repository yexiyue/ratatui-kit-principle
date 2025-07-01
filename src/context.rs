use std::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
};

/// Context 枚举用于封装不同类型的上下文数据，支持只读引用、可变引用和拥有权三种模式。
pub enum Context<'a> {
    /// 只读引用上下文
    Ref(&'a (dyn Any + Send + Sync)),
    /// 可变引用上下文
    Mut(&'a mut (dyn Any + Send + Sync)),
    /// 拥有所有权的上下文
    Owned(Box<dyn Any + Send + Sync>),
}

impl<'a> Context<'a> {
    /// 以拥有权的方式创建 Context
    pub fn owned<T: Any + Send + Sync>(context: T) -> Self {
        Context::Owned(Box::new(context))
    }

    /// 以只读引用的方式创建 Context
    pub fn form_ref<T: Any + Send + Sync>(context: &'a T) -> Self {
        Context::Ref(context)
    }

    /// 以可变引用的方式创建 Context
    pub fn form_mut<T: Any + Send + Sync>(context: &'a mut T) -> Self {
        Context::Mut(context)
    }

    /// 尝试将 Context 向下转型为指定类型的只读引用
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        match self {
            Context::Ref(context) => context.downcast_ref(),
            Context::Mut(context) => context.downcast_ref(),
            Context::Owned(context) => context.downcast_ref(),
        }
    }

    /// 尝试将 Context 向下转型为指定类型的可变引用
    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        match self {
            Context::Ref(_) => None,
            Context::Mut(context) => context.downcast_mut(),
            Context::Owned(context) => context.downcast_mut(),
        }
    }

    /// 获取 Context 的可变引用副本（Owned 会转为 Mut）
    pub fn borrow(&mut self) -> Context {
        match self {
            Context::Ref(context) => Context::Ref(*context),
            Context::Mut(context) => Context::Mut(*context),
            Context::Owned(context) => Context::Mut(&mut **context),
        }
    }
}

/// ContextStack 用于维护一个上下文栈，支持多层嵌套的上下文数据管理。
pub struct ContextStack<'a> {
    /// 栈结构，存储各层级的 Context
    stack: Vec<RefCell<Context<'a>>>,
}

impl<'a> ContextStack<'a> {
    /// 创建一个以根上下文为起点的 ContextStack
    pub(crate) fn root(root_context: &'a mut (dyn Any + Send + Sync)) -> Self {
        ContextStack {
            stack: vec![RefCell::new(Context::Mut(root_context))],
        }
    }

    /// 在上下文栈中临时插入一个新的上下文，并在闭包 f 执行期间可用。
    /// 适用于组件树递归遍历时临时注入局部上下文。
    ///
    /// # Safety
    /// 通过 transmute 缩短生命周期，仅在闭包作用域内安全。
    pub(crate) fn with_context<'b, F>(&'b mut self, context: Option<Context<'b>>, f: F)
    where
        F: FnOnce(&mut ContextStack),
    {
        if let Some(context) = context {
            // SAFETY: 可变引用在生命周期上是不变的，为了插入更短生命周期的上下文，需要对 'a 进行转变。
            // 只有在不允许对栈进行其他更改，并且在调用后立即恢复栈的情况下才是安全的。
            let shorter_lived_self =
                unsafe { std::mem::transmute::<&mut Self, &mut ContextStack<'b>>(self) };
            shorter_lived_self.stack.push(RefCell::new(context));
            f(shorter_lived_self);
            shorter_lived_self.stack.pop();
        } else {
            f(self);
        };
    }

    /// 获取栈顶到栈底第一个类型为 T 的只读上下文引用
    pub fn get_context<T: Any>(&self) -> Option<Ref<T>> {
        for context in self.stack.iter().rev() {
            if let Ok(context) = context.try_borrow() {
                if let Ok(res) = Ref::filter_map(context, |context| context.downcast_ref::<T>()) {
                    return Some(res);
                }
            }
        }
        None
    }

    /// 获取栈顶到栈底第一个类型为 T 的可变上下文引用
    pub fn get_context_mut<T: Any>(&self) -> Option<RefMut<T>> {
        for context in self.stack.iter().rev() {
            if let Ok(context) = context.try_borrow_mut() {
                if let Ok(res) = RefMut::filter_map(context, |context| context.downcast_mut::<T>())
                {
                    return Some(res);
                }
            }
        }
        None
    }
}

pub struct SystemContext {
    should_exit: bool,
}

unsafe impl Send for SystemContext {}
unsafe impl Sync for SystemContext {}

impl SystemContext {
    pub(crate) fn new() -> Self {
        Self { should_exit: false }
    }

    pub(crate) fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub fn exit(&mut self) {
        self.should_exit = true;
    }
}
