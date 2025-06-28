use std::any::TypeId;

use crate::{
    component::{AnyComponent, Component},
    props::AnyProps,
    render::updater::ComponentUpdater,
};

// 定义一个扩展 trait，用于创建和复制组件辅助对象
pub trait ComponentHelperExt {
    // 使用类型擦除的 AnyProps 创建一个新的组件实例
    fn new_component(&self, props: AnyProps) -> Box<dyn AnyComponent>;

    // 创建当前组件辅助对象的一个副本
    fn copy(&self) -> Box<dyn ComponentHelperExt>;

    fn update_component(
        &self,
        component: &mut Box<dyn AnyComponent>,
        props: AnyProps,
        updater: &mut ComponentUpdater,
    );

    fn component_type_id(&self) -> TypeId;
}

// 通用组件辅助结构体，用于泛型组件的构造和管理
pub struct ComponentHelper<T> {
    // 使用 PhantomData 标记泛型参数 T
    _marker: std::marker::PhantomData<T>,
}

// 为 ComponentHelper 实现工厂方法
impl<T> ComponentHelper<T>
where
    T: Component,
{
    // 构造一个 Box<dyn ComponentHelperExt> 类型的组件辅助对象
    pub fn boxed() -> Box<dyn ComponentHelperExt> {
        Box::new(Self {
            _marker: std::marker::PhantomData,
        })
    }
}

// 为 ComponentHelper 实现 ComponentHelperExt trait
impl<T> ComponentHelperExt for ComponentHelper<T>
where
    T: Component,
{
    // 使用 AnyProps 创建具体类型的组件实例
    // 调用者需确保 props 指向的数据确实是 T::Props 所对应的类型
    fn new_component(&self, props: AnyProps) -> Box<dyn AnyComponent> {
        Box::new(T::new(unsafe { props.downcast_ref_unchecked() }))
    }

    // 创建当前组件辅助对象的新副本
    fn copy(&self) -> Box<dyn ComponentHelperExt> {
        Self::boxed()
    }

    fn update_component(
        &self,
        component: &mut Box<dyn AnyComponent>,
        props: AnyProps,
        updater: &mut ComponentUpdater,
    ) {
        component.update(props, updater);
    }

    fn component_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}
