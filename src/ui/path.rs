use tui::layout::Rect;
use tui::widgets::{Widget};
use tui::buffer::{Buffer};
use tui::style::{Color};
use crate::ecs::Position;

pub struct Path<'a> {
    path: &'a [Position]
}

impl <'a> Path<'a> {
    pub fn new(path: &'a [Position]) -> Self {
        Path {
            path: path
        }
    }
}

impl <'a> Path<'a> {
    fn draw_cell(&mut self, area: Rect, position: &'a Position, buf: &mut Buffer) {
        if position.x >= area.x as i32 && position.x < area.right() as i32
        && position.y >= area.y as i32 && position.y < area.bottom() as i32 {
            let c = buf.get_mut(position.x as u16, position.y as u16);
            c.set_bg(Color::LightCyan);
        }
    }
}

impl <'a> Widget for Path<'a> {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        for pos in self.path {
            self.draw_cell(area, pos, buf);
        }
    }
}