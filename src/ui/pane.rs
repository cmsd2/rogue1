use tui::widgets::Widget;
use tui::buffer::{Buffer};
use tui::layout::Rect;

pub struct Pane {
    pub buffer: Buffer,
}

impl Pane {
    pub fn new() -> Self {
        Pane {
            buffer: Buffer::default(),
        }
    }

    pub fn size(mut self, area: Rect) -> Pane {
        self.buffer.resize(area);
        self
    }
}

impl Default for Pane {
    fn default() -> Self {
        Pane::new()
    }
}

impl Widget for Pane {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let r = self.buffer.area.intersection(area);
        for i in r.left()..r.right() {
            for j in r.top()..r.bottom() {
                let c = self.buffer.get(i, j);
                *buf.get_mut(i, j) = c.clone();
            }
        }
    }
}