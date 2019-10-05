use tui::widgets::Widget;
use tui::buffer::{Buffer, Cell};
use tui::layout::Rect;
use tui::style::Style;
use specs::{Entities, ReadStorage};
use crate::level::{Level, Cell as LevelCell};
use crate::ecs::{Character, Position};

pub struct LevelView<'a> {
    pub area: Rect,
    pub level: &'a Level,
    pub entities: &'a Entities<'a>,
    pub characters: &'a ReadStorage<'a, Character>,
    pub positions: &'a ReadStorage<'a, Position>,
}

impl <'a> LevelView<'a> {
    pub fn new(level: &'a Level, entities: &'a Entities, characters: &'a ReadStorage<'a, Character>, positions: &'a ReadStorage<'a, Position>) -> Self {
        LevelView {
            area: Rect::default(),
            level: level,
            entities: entities,
            characters: characters,
            positions: positions,
        }
    }

    pub fn size(mut self, area: Rect) -> Self {
        self.area = area;
        self
    }

    fn draw_cell(&self, cell: &mut Cell, level_cell: &LevelCell) {
        if let Some(entity) = level_cell.entities.get(0) {
            cell.set_symbol(&entity.character.to_string());
            cell.set_style(Style::default().fg(entity.color));
        } else {
            cell.set_symbol(&format!("{}", level_cell));
            cell.set_style(Style::default().fg(level_cell.color));
        }
    }
}

impl <'a> Widget for LevelView<'a> {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        use specs::Join;

        let r = self.level.area.intersection(area);
        for i in r.left()..r.right() {
            for j in r.top()..r.bottom() {
                let c = self.level.get(i, j);
                self.draw_cell(buf.get_mut(i, j), &c);
            }
        }

        for (entity, pos) in (self.characters, self.positions).join() {
            if pos.x >= r.x as i32 && pos.x < r.right() as i32 && pos.y >= r.y as i32 && pos.y < r.bottom() as i32 {
                //if fov_map.is_in_fov(pos.x, pos.y) {
                    let c = buf.get_mut(pos.x as u16, pos.y as u16);
                    c.set_symbol(&entity.glyph.to_string());
                //}
            }
		}
    }
}