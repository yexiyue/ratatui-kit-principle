use crate::{
    component::instantiated_component::{Components, InstantiatedComponent},
    context::{Context, ContextStack},
    element::{ElementExt, key::ElementKey},
    multimap::AppendOnlyMultimap,
    render::layout_style::LayoutStyle,
    terminal::Terminal,
};

pub struct ComponentUpdater<'a, 'b: 'a> {
    key: ElementKey,
    components: &'a mut Components,
    layout_style: &'a mut LayoutStyle,
    terminal: &'a mut Terminal,
    component_context_stack: &'a mut ContextStack<'b>,
}

impl<'a, 'b: 'a> ComponentUpdater<'a, 'b> {
    pub fn new(
        key: ElementKey,
        components: &'a mut Components,
        layout_style: &'a mut LayoutStyle,
        terminal: &'a mut Terminal,
        component_context_stack: &'a mut ContextStack<'b>,
    ) -> Self {
        Self {
            key,
            components,
            layout_style,
            terminal,
            component_context_stack,
        }
    }
    
    pub fn component_context_stack(&self) -> &ContextStack<'b> {
        self.component_context_stack
    }

    pub fn terminal(&mut self) -> &mut Terminal {
        self.terminal
    }

    /// 获取当前组件的唯一标识 key。
    pub fn key(&self) -> &ElementKey {
        &self.key
    }

    pub fn set_layout_style(&mut self, layout_style: LayoutStyle) {
        *self.layout_style = layout_style;
    }

    /// 根据传入的 children 列表，更新当前组件的所有子组件。
    ///
    /// 算法说明：
    /// 1. 遍历新的 children（每个 AnyElement），尝试用 key 从旧组件映射中取出对应的 InstantiatedComponent。
    /// 2. 如果 key 匹配且类型一致，则复用旧组件实例，否则新建一个组件实例。
    /// 3. 对每个组件实例调用 update，传入新的 props。
    /// 4. 将本轮用到的组件按顺序插入新的 multimap，最后整体替换原有的 components。
    ///
    /// 这样可以保证：
    /// - 组件 key 不变且类型一致时，组件实例被复用，保留内部状态。
    /// - key 变更或类型不一致时，自动销毁旧实例并新建，保证类型安全。
    /// - 未被复用的旧组件会被丢弃，实现“最小化重建”。
    pub fn update_children<T, E>(&mut self, children: T, context: Option<Context>)
    where
        T: IntoIterator<Item = E>,
        E: ElementExt,
    {
        // 通过 with_context 方法将 context 传递给组件上下文栈
        self.component_context_stack
            .with_context(context, |context_stack| {
                // 新建一个 multimap，用于存放本轮更新后实际用到的组件实例
                let mut used_compoent = AppendOnlyMultimap::default();

                // 遍历新的 children 列表
                for mut child in children {
                    // 尝试用 key 从旧组件集合中取出一个实例
                    let mut component = match self.components.pop_front(&child.key()) {
                        // 如果 key 匹配且类型一致，则复用旧组件实例
                        Some(component)
                            if component.component().type_id()
                                == child.helper().component_type_id() =>
                        {
                            component
                        }
                        // 否则新建一个组件实例
                        _ => {
                            let h = child.helper().copy();
                            InstantiatedComponent::new(child.key().clone(), child.props_mut(), h)
                        }
                    };

                    // 用新的 props和context 更新组件实例
                    component.update(child.props_mut(), self.terminal, context_stack);
                    // 将本轮用到的组件实例插入 multimap
                    used_compoent.push_back(child.key().clone(), component);
                }

                // 用新的 multimap 替换原有的 components，实现“最小化重建”
                self.components.components = used_compoent.into();
            });
    }
}
