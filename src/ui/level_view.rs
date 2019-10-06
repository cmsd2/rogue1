use std::cmp;
use tui::widgets::Widget;
use tui::buffer::{Buffer, Cell};
use tui::layout::Rect;
use tui::style::{Style, Color, Modifier};
use specs::{Entities, ReadExpect, ReadStorage};
use crate::game::level::{Level, Cell as LevelCell};
use crate::game::ecs::{Character, Position};
use crate::game::fov::Fov;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Visibility {
    Visible,
    Explored,
}

pub struct LevelView<'a> {
    pub area: Rect,
    pub level: &'a Level,
    pub fov: &'a ReadExpect<'a, Fov>,
    pub entities: &'a Entities<'a>,
    pub characters: &'a ReadStorage<'a, Character>,
    pub positions: &'a ReadStorage<'a, Position>,
}

impl <'a> LevelView<'a> {
    pub fn new(level: &'a Level, fov: &'a ReadExpect<'a, Fov>, entities: &'a Entities, characters: &'a ReadStorage<'a, Character>, positions: &'a ReadStorage<'a, Position>) -> Self {
        LevelView {
            area: Rect::default(),
            level: level,
            fov: fov,
            entities: entities,
            characters: characters,
            positions: positions,
        }
    }

    pub fn level_area(mut self, area: Rect) -> Self {
        self.area = area;
        self
    }

    fn draw_cell(&self, cell: &mut Cell, level_cell: &LevelCell, visibility: Visibility) {
        if let Some(entity) = level_cell.entities.get(0) {
            cell.set_symbol(&entity.character.to_string());
            cell.set_style(Style::default().fg(entity.color).bg(Color::Black));
        } else {
            cell.set_symbol(&format!("{}", level_cell));
            cell.set_style(Style::default().fg(level_cell.color).bg(Color::Black));
        }

        if visibility == Visibility::Explored {
            cell.set_modifier(Modifier::DIM);
        }
    }
}

impl <'a> Widget for LevelView<'a> {
    fn draw(&mut self, draw_area: Rect, buf: &mut Buffer) {
        use specs::Join;

        let mut level_area = self.level.area.intersection(self.area);
        level_area.width = cmp::min(level_area.width, draw_area.width);
        level_area.height = cmp::min(level_area.height, draw_area.height);

        for i in draw_area.left()..draw_area.right() {
            for j in draw_area.top()..draw_area.bottom() {
                buf.get_mut(i, j).reset();
            }
        }

        for level_i in 0..level_area.width {
            for level_j in 0..level_area.height {
                let level_x = level_area.x + level_i;
                let level_y = level_area.y + level_j;
                let draw_x = draw_area.x + level_i;
                let draw_y = draw_area.y + level_j;

                let c = self.level.get(level_x, level_y);
                if self.fov.is_in_fov(&Position::new(level_x as i32, level_y as i32)) {
                    self.draw_cell(buf.get_mut(draw_x, draw_y), &c, Visibility::Visible);
                } else if c.explored {
                    self.draw_cell(buf.get_mut(draw_x, draw_y), &c, Visibility::Explored);
                }
            }
        }

        for (entity, pos) in (self.characters, self.positions).join() {
            let draw_x = pos.x - level_area.x as i32 + draw_area.x as i32;
            let draw_y = pos.y - level_area.y as i32 + draw_area.y as i32;

            if draw_x >= draw_area.x as i32 && draw_x < draw_area.right() as i32 && draw_y >= draw_area.y as i32 && draw_y < draw_area.bottom() as i32 {
                if self.fov.is_in_fov(pos) {
                    let c = buf.get_mut(draw_x as u16, draw_y as u16);
                    c.set_symbol(&entity.glyph.to_string());
                }
            }
		}
    }
}