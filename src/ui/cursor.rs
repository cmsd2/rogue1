use tui::layout::Rect;
use tui::widgets::{Widget};
use tui::buffer::{Buffer};
use tui::style::{Color};
use crate::ecs::Position;

pub struct Cursor {
    position: Position
}

impl Cursor {
    pub fn new(position: Position) -> Self {
        Cursor {
            position: position
        }
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Cursor {
            position: Position { x: 0, y: 0 }
        }
    }
}

impl Widget for Cursor {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        if self.position.x >= area.x as i32 && self.position.x < area.right() as i32
        && self.position.y >= area.y as i32 && self.position.y < area.bottom() as i32 {
            let c = buf.get_mut(self.position.x as u16, self.position.y as u16);
            c.set_bg(Color::LightCyan);
        }
    }
}