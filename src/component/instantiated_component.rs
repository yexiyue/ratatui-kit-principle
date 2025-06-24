use super::component_helper::ComponentHelperExt;
use crate::{component::AnyComponent, props::AnyProps};
use std::ops::{Deref, DerefMut};

pub struct InstantiatedComponent {
    component: Box<dyn AnyComponent>,
    children: Components,
    helper: Box<dyn ComponentHelperExt>,
}

impl InstantiatedComponent {
    pub fn new(mut props: AnyProps, helper: Box<dyn ComponentHelperExt>) -> Self {
        let component = helper.new_component(props.borrow());

        Self {
            component,
            children: Components::default(),
            helper,
        }
    }
}

#[derive(Default)]
pub struct Components {
    pub components: Vec<InstantiatedComponent>,
}

impl Deref for Components {
    type Target = Vec<InstantiatedComponent>;

    fn deref(&self) -> &Self::Target {
        &self.components
    }
}

impl DerefMut for Components {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.components
    }
}
