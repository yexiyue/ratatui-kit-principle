pub mod key;
use std::io;

use crate::{
    component::{
        Component,
        component_helper::{ComponentHelper, ComponentHelperExt},
    },
    props::AnyProps,
    render::tree::render_loop,
};
pub use key::ElementKey;
mod any_element;
pub use any_element::AnyElement;
mod element_ext;
pub use element_ext::ElementExt;
mod extend_with_elements;
pub use extend_with_elements::{ExtendWithElements, extend_with_elements};

/// ElementType trait：为每种组件类型定义 Props 类型，便于泛型处理
pub trait ElementType {
    type Props<'a>
    where
        Self: 'a;
}

/// 为所有实现 Component 的类型自动实现 ElementType
impl<C> ElementType for C
where
    C: Component,
{
    type Props<'a> = C::Props<'a>;
}

/// Element 结构体：描述一个待实例化的组件，包括 key 和 props
pub struct Element<'a, T: ElementType + 'a> {
    pub key: ElementKey,     // 唯一标识
    pub props: T::Props<'a>, // 组件属性
}

impl<'a, T> ElementExt for Element<'a, T>
where
    T: Component,
{
    fn key(&self) -> &ElementKey {
        &self.key
    }

    fn helper(&self) -> Box<dyn ComponentHelperExt> {
        ComponentHelper::<T>::boxed()
    }

    fn props_mut(&mut self) -> AnyProps {
        AnyProps::borrowed(&mut self.props)
    }

    async fn render_loop(&mut self) -> io::Result<()> {
        render_loop(self).await?;
        Ok(())
    }
}

impl<'a, T> ElementExt for &mut Element<'a, T>
where
    T: Component,
{
    fn key(&self) -> &ElementKey {
        &self.key
    }

    fn helper(&self) -> Box<dyn ComponentHelperExt> {
        ComponentHelper::<T>::boxed()
    }

    fn props_mut(&mut self) -> AnyProps {
        AnyProps::borrowed(&mut self.props)
    }

    async fn render_loop(&mut self) -> io::Result<()> {
        render_loop(&mut **self).await?;
        Ok(())
    }
}
