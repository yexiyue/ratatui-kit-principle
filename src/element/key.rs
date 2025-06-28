use any_key::AnyHash;
use std::{fmt::Debug, hash::Hash, sync::Arc};

/// ElementKey：用于唯一标识组件树中的节点，支持任意可哈希类型
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ElementKey(Arc<Box<dyn AnyHash + Send + Sync>>);

impl ElementKey {
    /// 创建新的 ElementKey，支持任意实现了 AnyHash 的类型
    pub fn new<T>(value: T) -> Self
    where
        T: Debug + Send + Sync + AnyHash,
    {
        Self(Arc::new(Box::new(value)))
    }
}
