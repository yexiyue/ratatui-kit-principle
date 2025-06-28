// Element 扩展 trait 及相关工具，便于统一操作不同类型的 Element
use super::ElementKey;
use crate::{component::component_helper::ComponentHelperExt, props::AnyProps};
use std::io;

/// 私有模块，用于实现 trait 封装，防止外部实现 ElementExt
mod private {
    use crate::{
        component::Component,
        element::{AnyElement, Element},
    };

    /// Sealed trait，防止外部实现 ElementExt
    pub trait Sealed {}

    // 为 AnyElement 及其可变引用实现 Sealed
    impl<'a> Sealed for AnyElement<'a> {}
    impl<'a> Sealed for &mut AnyElement<'a> {}

    // 为泛型 Element 及其可变引用实现 Sealed
    impl<'a, T> Sealed for Element<'a, T> where T: Component {}
    impl<'a, T> Sealed for &mut Element<'a, T> where T: Component {}
}

/// ElementExt trait：为 Element/AnyElement 提供统一的扩展方法
pub trait ElementExt: private::Sealed + Sized {
    /// 获取节点唯一 key
    fn key(&self) -> &ElementKey;
    /// 获取可变 props（类型擦除）
    fn props_mut(&mut self) -> AnyProps;
    /// 获取组件 helper
    fn helper(&self) -> Box<dyn ComponentHelperExt>;

    /// 启动渲染主循环
    fn render_loop(&mut self) -> impl Future<Output = io::Result<()>>;
}
