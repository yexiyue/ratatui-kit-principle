use ratatui::{layout::Rect, widgets::Widget};

/// 用于封装组件绘制上下文，便于在组件内部安全地操作 frame 和区域
pub struct ComponentDrawer<'a, 'b: 'a> {
    /// 当前组件的绘制区域
    pub area: ratatui::layout::Rect,
    /// 指向全局 frame 的可变引用
    pub frame: &'a mut ratatui::Frame<'b>,
}

impl<'a, 'b> ComponentDrawer<'a, 'b> {
    /// 创建新的 ComponentDrawer
    pub fn new(frame: &'a mut ratatui::Frame<'b>, area: ratatui::layout::Rect) -> Self {
        Self { area, frame }
    }

    /// 获取底层 buffer 的可变引用
    pub fn buffer_mut(&mut self) -> &mut ratatui::buffer::Buffer {
        self.frame.buffer_mut()
    }

    pub fn render_widget<W: Widget>(&mut self, widget: W, area: Rect) {
        widget.render(area, self.buffer_mut());
    }
}
