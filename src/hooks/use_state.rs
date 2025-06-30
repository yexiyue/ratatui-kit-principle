use super::{Hook, Hooks};
use generational_box::{
    AnyStorage, BorrowError, BorrowMutError, GenerationalBox, Owner, SyncStorage,
};
use std::{
    cmp,
    fmt::{self, Debug, Display, Formatter},
    hash::{Hash, Hasher},
    ops::{self, Deref, DerefMut},
    task::{Poll, Waker},
};

mod private {
    pub trait Sealed {}
    impl Sealed for crate::hooks::Hooks<'_> {}
}

// use_state 的 Hook trait扩展，允许在组件中声明本地状态
pub trait UseState: private::Sealed {
    /// use_state：在组件中声明一个本地状态
    /// init: 状态初始化闭包
    /// 返回 State<T> 句柄
    fn use_state<T, F>(&mut self, init: F) -> State<T>
    where
        F: FnOnce() -> T,
        T: Unpin + Send + Sync + 'static;
}

// use_state 的 Hook 实现，负责状态的生命周期和变更检测
struct UseStateImpl<T>
where
    T: Unpin + Send + Sync + 'static,
{
    state: State<T>,              // 状态句柄
    _storage: Owner<SyncStorage>, // 状态存储的所有权，保证生命周期
}

impl<T> UseStateImpl<T>
where
    T: Unpin + Send + Sync + 'static,
{
    /// 创建新的 UseStateImpl，初始化状态
    pub fn new(initial_value: T) -> Self {
        let storage = Owner::default();
        UseStateImpl {
            state: State {
                inner: storage.insert(StateValue {
                    value: initial_value,
                    waker: None,
                    is_changed: false,
                }),
            },
            _storage: storage,
        }
    }
}

// Hook trait实现，集成到组件生命周期，自动检测状态变更
impl<T> Hook for UseStateImpl<T>
where
    T: Unpin + Send + Sync + 'static,
{
    fn poll_change(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context,
    ) -> std::task::Poll<()> {
        if let Ok(mut value) = self.state.inner.try_write() {
            if value.is_changed {
                value.is_changed = false;
                Poll::Ready(()) // 状态变更，触发 UI 更新
            } else {
                value.waker = Some(cx.waker().clone()); // 注册唤醒器
                Poll::Pending
            }
        } else {
            Poll::Pending
        }
    }
}

// Hooks<'_> 扩展，实现 use_state 方法
impl UseState for Hooks<'_> {
    fn use_state<T, F>(&mut self, init: F) -> State<T>
    where
        F: FnOnce() -> T,
        T: Unpin + Send + Sync + 'static,
    {
        self.use_hook(move || UseStateImpl::new(init())).state
    }
}

// 状态实际存储结构，包含值、唤醒器和变更标记
struct StateValue<T> {
    value: T,             // 当前状态值
    waker: Option<Waker>, // 用于唤醒 UI 的 waker
    is_changed: bool,     // 变更标记
}

// 状态只读引用包装器
pub struct StateRef<'a, T: 'static> {
    inner: <SyncStorage as AnyStorage>::Ref<'a, StateValue<T>>,
}

impl<T: 'static> Deref for StateRef<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner.value
    }
}

// 状态可变引用包装器，支持变更追踪
pub struct StateMutRef<'a, T: 'static> {
    inner: <SyncStorage as AnyStorage>::Mut<'a, StateValue<T>>,
    is_deref_mut: bool, // 标记是否发生过可变借用
}

impl<T: 'static> Deref for StateMutRef<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner.value
    }
}

impl<T: 'static> DerefMut for StateMutRef<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.is_deref_mut = true;
        &mut self.inner.value
    }
}

// Drop 时自动标记变更并唤醒 UI
impl<T: 'static> Drop for StateMutRef<'_, T> {
    fn drop(&mut self) {
        if self.is_deref_mut {
            self.inner.is_changed = true;
            if let Some(waker) = self.inner.waker.take() {
                waker.wake();
            }
        }
    }
}

// State 句柄，负责状态的安全访问和操作
pub struct State<T: Send + Sync + 'static> {
    inner: GenerationalBox<StateValue<T>, SyncStorage>,
}

// State 支持 Copy/Clone，方便在组件中多处传递
impl<T: Send + Sync + 'static> Clone for State<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: Send + Sync + 'static> Copy for State<T> {}

// State 支持直接获取值（T: Copy）
impl<T: Send + Sync + Copy + 'static> State<T> {
    pub fn get(&self) -> T {
        *self.read()
    }
}

impl<T: Send + Sync + 'static> State<T> {
    /// 尝试获取状态的不可变引用
    ///
    /// 返回 `Option<StateRef<T>>`，如果状态已被释放返回 `None`
    pub fn try_read(&self) -> Option<StateRef<T>> {
        loop {
            match self.inner.try_read() {
                Ok(inner) => return Some(StateRef { inner }),
                Err(BorrowError::Dropped(_)) => {
                    return None;
                }
                Err(BorrowError::AlreadyBorrowedMut(_)) => match self.inner.try_write() {
                    Err(BorrowMutError::Dropped(_)) => {
                        return None;
                    }
                    _ => continue,
                },
            }
        }
    }

    /// 获取状态的不可变引用（阻塞等待）
    ///
    /// 如果状态已被释放，会 panic
    pub fn read(&self) -> StateRef<T> {
        self.try_read()
            .expect("attempt to read state after owner was dropped")
    }

    /// 尝试获取状态的可变引用
    ///
    /// 返回 `Option<StateMutRef<T>>`，如果状态已被释放返回 `None`
    pub fn try_write(&self) -> Option<StateMutRef<T>> {
        self.inner
            .try_write()
            .map(|inner| StateMutRef {
                inner,
                is_deref_mut: false,
            })
            .ok()
    }

    /// 获取状态的可变引用（阻塞等待）
    ///
    /// 如果状态已被释放，会 panic
    pub fn write(&self) -> StateMutRef<T> {
        self.try_write()
            .expect("attempt to write state after owner was dropped")
    }

    /// 设置新的状态值
    ///
    /// 如果能获取到可变引用，则更新状态值
    pub fn set(&mut self, value: T) {
        if let Some(mut v) = self.try_write() {
            *v = value;
        }
    }
}

// State 支持 Debug/Display/算术/比较等常用 trait，方便直接参与运算和输出
impl<T: Debug + Sync + Send + 'static> Debug for State<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.read().fmt(f)
    }
}

impl<T: Display + Sync + Send + 'static> Display for State<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.read().fmt(f)
    }
}

impl<T: ops::Add<Output = T> + Copy + Sync + Send + 'static> ops::Add<T> for State<T> {
    type Output = T;

    fn add(self, rhs: T) -> Self::Output {
        self.get() + rhs
    }
}

impl<T: ops::AddAssign<T> + Copy + Sync + Send + 'static> ops::AddAssign<T> for State<T> {
    fn add_assign(&mut self, rhs: T) {
        if let Some(mut v) = self.try_write() {
            *v += rhs;
        }
    }
}

impl<T: ops::Sub<Output = T> + Copy + Sync + Send + 'static> ops::Sub<T> for State<T> {
    type Output = T;

    fn sub(self, rhs: T) -> Self::Output {
        self.get() - rhs
    }
}

impl<T: ops::SubAssign<T> + Copy + Sync + Send + 'static> ops::SubAssign<T> for State<T> {
    fn sub_assign(&mut self, rhs: T) {
        if let Some(mut v) = self.try_write() {
            *v -= rhs;
        }
    }
}

impl<T: ops::Mul<Output = T> + Copy + Sync + Send + 'static> ops::Mul<T> for State<T> {
    type Output = T;

    fn mul(self, rhs: T) -> Self::Output {
        self.get() * rhs
    }
}

impl<T: ops::MulAssign<T> + Copy + Sync + Send + 'static> ops::MulAssign<T> for State<T> {
    fn mul_assign(&mut self, rhs: T) {
        if let Some(mut v) = self.try_write() {
            *v *= rhs;
        }
    }
}

impl<T: ops::Div<Output = T> + Copy + Sync + Send + 'static> ops::Div<T> for State<T> {
    type Output = T;

    fn div(self, rhs: T) -> Self::Output {
        self.get() / rhs
    }
}

impl<T: ops::DivAssign<T> + Copy + Sync + Send + 'static> ops::DivAssign<T> for State<T> {
    fn div_assign(&mut self, rhs: T) {
        if let Some(mut v) = self.try_write() {
            *v /= rhs;
        }
    }
}

impl<T: Hash + Sync + Send> Hash for State<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.read().hash(state)
    }
}

impl<T: cmp::PartialEq<T> + Sync + Send + 'static> cmp::PartialEq<T> for State<T> {
    fn eq(&self, other: &T) -> bool {
        *self.read() == *other
    }
}

impl<T: cmp::PartialOrd<T> + Sync + Send + 'static> cmp::PartialOrd<T> for State<T> {
    fn partial_cmp(&self, other: &T) -> Option<cmp::Ordering> {
        self.read().partial_cmp(other)
    }
}

impl<T: cmp::PartialEq<T> + Sync + Send + 'static> cmp::PartialEq<State<T>> for State<T> {
    fn eq(&self, other: &State<T>) -> bool {
        *self.read() == *other.read()
    }
}

impl<T: cmp::PartialOrd<T> + Sync + Send + 'static> cmp::PartialOrd<State<T>> for State<T> {
    fn partial_cmp(&self, other: &State<T>) -> Option<cmp::Ordering> {
        self.read().partial_cmp(&other.read())
    }
}

impl<T: cmp::Eq + Sync + Send + 'static> cmp::Eq for State<T> {}
