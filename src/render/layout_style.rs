use ratatui::layout::{Constraint, Direction, Flex, Layout, Margin, Offset};

/// 用于描述组件布局样式的结构体，类似于 Web 的 Flex 布局属性
#[derive(Default)]
pub struct LayoutStyle {
    /// 主轴方向（横向/纵向）
    pub flex_direction: Direction,
    /// 主轴对齐方式（如 Start, End, Center, SpaceBetween 等）
    pub justify_content: Flex,
    /// 子项间距
    pub gap: i32,
    /// 外边距
    pub margin: Margin,
    /// 偏移量
    pub offset: Offset,
    /// 宽度约束
    pub width: Constraint,
    /// 高度约束
    pub height: Constraint,
}

impl LayoutStyle {
    /// 根据当前样式生成 Ratatui 的 Layout 对象
    pub fn get_layout(&self) -> Layout {
        Layout::default()
            .direction(self.flex_direction)
            .flex(self.justify_content)
            .spacing(self.gap)
    }

    /// 获取宽度约束
    pub fn get_width(&self) -> Constraint {
        self.width
    }

    /// 获取高度约束
    pub fn get_height(&self) -> Constraint {
        self.height
    }

    /// 计算应用 margin 和 offset 后的内部区域
    pub fn inner_area(&self, area: ratatui::layout::Rect) -> ratatui::layout::Rect {
        area.offset(self.offset).inner(self.margin)
    }
}
