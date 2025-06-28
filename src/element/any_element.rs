use std::io;

use crate::{
    component::{
        Component,
        component_helper::{ComponentHelper, ComponentHelperExt},
    },
    element::{Element, ElementExt, key::ElementKey},
    props::AnyProps,
    render::tree::render_loop,
};

/// AnyElement 是一个类型擦除的容器，用于存储任意类型的 Element 组件
///
/// 它与 Element 的关系类似于 trait 对象（dyn Trait）与具体实现的关系：
/// - Element<'a, T> 表示一个具体的、带有特定组件类型的元素
/// - AnyElement 表示一个类型被擦除的元素，可以存储任何类型的组件
///
/// 这种设计模式在框架中非常常见，主要出于以下考虑：
/// 1. 类型擦除：允许存储和操作不同类型的组件，而不需要知道具体类型
/// 2. 接口统一：提供统一的接口来处理不同类型的组件
/// 3. 动态多态：实现运行时动态分发不同组件的行为
pub struct AnyElement<'a> {
    pub key: ElementKey,                     // 组件的唯一标识
    pub props: AnyProps<'a>,                 // 类型擦除的属性容器
    pub helper: Box<dyn ComponentHelperExt>, // 用于创建和管理组件的帮助器
}

/// 将拥有所有权的 Element<T> 转换为 AnyElement
///
/// 这个转换实现了从具体类型到类型擦除的转变，主要做了三件事：
/// 1. 提取 Element 的 key 作为组件标识
/// 2. 使用 AnyProps::owned 将具体类型的 props 转换为类型擦除的形式
/// 3. 创建对应的 ComponentHelper 作为组件工厂
impl<'a, T> From<Element<'a, T>> for AnyElement<'a>
where
    T: Component,
{
    fn from(value: Element<'a, T>) -> Self {
        Self {
            key: value.key,
            props: AnyProps::owned(value.props),
            helper: ComponentHelper::<T>::boxed(),
        }
    }
}

/// 将可变引用的 Element<T> 转换为 AnyElement
///
/// 与上面的实现相比，这个转换保留了对原始 props 的引用，
/// 使用 AnyProps::borrowed 而不是获取所有权。
impl<'a, 'b: 'a, T> From<&'a mut Element<'b, T>> for AnyElement<'a>
where
    T: Component,
{
    fn from(value: &'a mut Element<'b, T>) -> Self {
        Self {
            key: value.key.clone(),
            props: AnyProps::borrowed(&mut value.props),
            helper: ComponentHelper::<T>::boxed(),
        }
    }
}

/// 将可变引用的 AnyElement 转换为 AnyElement
///
/// 这个转换用于复制现有的 AnyElement，主要用于组件更新场景。
/// 它会复制 key、借用 props 并克隆 helper。
impl<'a, 'b: 'a> From<&'a mut AnyElement<'b>> for AnyElement<'b> {
    fn from(value: &'a mut AnyElement<'b>) -> Self {
        Self {
            key: value.key.clone(),
            props: value.props.borrow(),
            helper: value.helper.copy(),
        }
    }
}

impl<'a> ElementExt for AnyElement<'a> {
    fn key(&self) -> &ElementKey {
        &self.key
    }

    fn helper(&self) -> Box<dyn ComponentHelperExt> {
        self.helper.copy()
    }

    fn props_mut(&mut self) -> AnyProps {
        self.props.borrow()
    }

    async fn render_loop(&mut self) -> io::Result<()> {
        render_loop(self).await?;
        Ok(())
    }
}

impl<'a> ElementExt for &mut AnyElement<'a> {
    fn key(&self) -> &ElementKey {
        &self.key
    }

    fn helper(&self) -> Box<dyn ComponentHelperExt> {
        self.helper.copy()
    }

    fn props_mut(&mut self) -> AnyProps {
        self.props.borrow()
    }

    async fn render_loop(&mut self) -> io::Result<()> {
        render_loop(&mut **self).await?;
        Ok(())
    }
}
