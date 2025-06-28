use std::{
    collections::{HashMap, VecDeque},
    hash::Hash,
};

/// AppendOnlyMultimap 是一个多值映射，允许向末尾追加值。
/// 它的主要特点是只支持追加操作，不支持删除操作。
pub(crate) struct AppendOnlyMultimap<K, V> {
    items: Vec<Option<V>>,          // 存储所有值的容器
    m: HashMap<K, VecDeque<usize>>, // 键到索引的映射
}

impl<K, V> Default for AppendOnlyMultimap<K, V> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            m: HashMap::new(),
        }
    }
}

impl<K, V> AppendOnlyMultimap<K, V>
where
    K: Eq + Hash,
{
    /// 向 multimap 末尾追加一个值，关联到指定的键。
    pub fn push_back(&mut self, key: K, value: V) {
        let index = self.items.len();
        self.items.push(Some(value));
        self.m.entry(key).or_default().push_back(index);
    }
}

/// RemoveOnlyMultimap 是一个多值映射，允许从前面移除值。
/// 它的主要特点是只支持删除操作，不支持追加操作。
pub struct RemoveOnlyMultimap<K, V> {
    items: Vec<Option<V>>,          // 存储所有值的容器
    m: HashMap<K, VecDeque<usize>>, // 键到索引的映射
}

impl<K, V> Default for RemoveOnlyMultimap<K, V> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            m: HashMap::new(),
        }
    }
}

impl<K, V> From<AppendOnlyMultimap<K, V>> for RemoveOnlyMultimap<K, V>
where
    K: Eq + Hash,
{
    fn from(value: AppendOnlyMultimap<K, V>) -> Self {
        Self {
            items: value.items,
            m: value.m,
        }
    }
}

impl<K, V> RemoveOnlyMultimap<K, V>
where
    K: Eq + Hash,
{
    /// 从 multimap 中移除与指定键关联的第一个值。
    pub fn pop_front(&mut self, key: &K) -> Option<V> {
        let index = self.m.get_mut(key)?.pop_front()?;
        self.items[index].take()
    }

    /// 遍历 multimap 中的所有值。
    pub fn iter(&self) -> impl Iterator<Item = &V> {
        self.items.iter().filter_map(|item| item.as_ref())
    }

    /// 遍历 multimap 中的所有值（可变引用）。
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.items.iter_mut().filter_map(|item| item.as_mut())
    }
}
