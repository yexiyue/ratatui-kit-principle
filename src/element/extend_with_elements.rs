use super::{AnyElement, Element, ElementType};

/// 一个 trait，用于将元素批量扩展（append）到目标集合中。
/// 主要用于支持多种元素类型（单个元素、AnyElement、迭代器等）统一扩展到目标集合，
/// 便于声明式 UI 宏灵活拼接元素。
pub trait ExtendWithElements<T> {
    /// 将自身的元素批量扩展到 dest 中。
    fn extend_with_elements<E: Extend<T>>(self, dest: &mut E);
}

/// 支持将单个 Element 扩展到目标集合。
impl<'a, T, U> ExtendWithElements<T> for Element<'a, U>
where
    U: ElementType + 'a,
    T: From<Element<'a, U>>,
{
    fn extend_with_elements<E: Extend<T>>(self, dest: &mut E) {
        // 将单个 Element 转换为 T 后扩展到目标集合
        dest.extend([self.into()]);
    }
}

/// 支持将单个 AnyElement 扩展到目标集合。
impl<'a> ExtendWithElements<AnyElement<'a>> for AnyElement<'a> {
    fn extend_with_elements<E: Extend<AnyElement<'a>>>(self, dest: &mut E) {
        // 直接扩展单个 AnyElement
        dest.extend([self]);
    }
}

/// 支持将迭代器中的元素批量扩展到目标集合。
impl<T, U, I> ExtendWithElements<T> for I
where
    T: From<U>,
    I: IntoIterator<Item = U>,
{
    fn extend_with_elements<E: Extend<T>>(self, dest: &mut E) {
        // 将迭代器中的每个元素转换为 T 后批量扩展到目标集合
        dest.extend(self.into_iter().map(|x| x.into()));
    }
}

/// 通用扩展函数，支持将任意实现 ExtendWithElements 的元素批量扩展到目标集合。
/// 主要用于声明式 UI 宏内部，统一处理单个元素、AnyElement、迭代器等多种情况。
pub fn extend_with_elements<T, U, E>(dest: &mut T, elements: U)
where
    U: ExtendWithElements<E>,
    T: Extend<E>,
{
    elements.extend_with_elements(dest);
}
