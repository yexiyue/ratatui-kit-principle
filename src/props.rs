// 定义一个用于释放原始指针内存的 trait
trait DropRaw {
    fn drop_raw(&self, raw: *mut ());
}

// 通过 PhantomData 标记类型 T 来保留结构体与泛型参数的关联关系
struct DropRawImpl<T> {
    _marker: std::marker::PhantomData<T>,
}

// 为 DropRawImpl 实现 DropRaw trait
impl<T> DropRaw for DropRawImpl<T> {
    // 将 *mut () 转换为 *mut T 并通过 Box::from_raw 构造智能指针来安全释放对应的堆内存
    fn drop_raw(&self, raw: *mut ()) {
        unsafe {
            let _ = Box::from_raw(raw as *mut T);
        }
    }
}

// 一个可以持有任意类型属性的结构体
pub struct AnyProps<'a> {
    raw: *mut (),                                  // 指向实际数据的原始指针
    drop: Option<Box<dyn DropRaw + 'a>>,           // 用于释放 raw 所指向的数据
    _marker: std::marker::PhantomData<&'a mut ()>, // 标记生命周期信息
}

impl<'a> AnyProps<'a> {
    pub(crate) fn owned<T: 'a>(props: T) -> Self {
        // 将堆分配的值转换为原始指针，用于手动内存管理
        let raw = Box::into_raw(Box::new(props));

        Self {
            // 将 *mut T 转换为 *mut () 实现类型擦除
            // 保留指向具体类型的指针信息，但隐藏具体类型
            raw: raw as *mut (),
            drop: Some(Box::new(DropRawImpl::<T> {
                _marker: std::marker::PhantomData,
            })),
            _marker: std::marker::PhantomData,
        }
    }

    pub(crate) fn borrowed<T>(props: &'a mut T) -> Self {
        // 创建一个不负责内存释放的 AnyProps 实例
        // 用于持有对 T 类型数据的引用
        Self {
            // 将 &mut T 转换为 *mut ()，实现类型擦除
            raw: props as *const _ as *mut (),

            // 不负责内存释放，因此 drop 设置为 None
            drop: None, // 不负责内存释放

            // 使用 PhantomData 标记生命周期信息
            _marker: std::marker::PhantomData,
        }
    }

    // 创建一个新的 AnyProps 实例，共享当前实例的 raw 指针
    // 不获取所有权，也不负责释放内存
    pub(crate) fn borrow(&mut self) -> Self {
        Self {
            raw: self.raw,
            drop: None, // 不负责内存释放
            _marker: std::marker::PhantomData,
        }
    }

    // 不安全地将内部指针向下转换为具体类型的不可变引用
    // 必须保证当前 raw 指针确实指向 T 类型的数据
    pub(crate) unsafe fn downcast_ref_unchecked<T>(&self) -> &T {
        unsafe { &*(self.raw as *const T) }
    }

    // 不安全地将内部指针向下转换为具体类型的可变引用
    // 必须保证当前 raw 指针确实指向 T 类型的数据
    pub(crate) unsafe fn downcast_mut_unchecked<T>(&mut self) -> &mut T {
        unsafe { &mut *(self.raw as *mut T) }
    }
}

impl Drop for AnyProps<'_> {
    fn drop(&mut self) {
        // 如果 drop 字段存在，则调用其 drop_raw 方法释放内存
        if let Some(drop) = self.drop.take() {
            drop.drop_raw(self.raw);
        }
    }
}
