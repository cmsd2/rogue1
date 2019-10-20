use specs::{Entities, ReadStorage};
use quicksilver::geom::{Rectangle, Vector};
use quicksilver::lifecycle::Window;
use quicksilver::Result;
use crate::game::level::{Level, Tile};
use crate::game::ecs::{Character, Position, Rect};
use crate::game::fov::Fov;
use crate::color::{Palette};
use super::Tileset;
use super::Widget;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Visibility {
    Visible,
    Explored,
}

pub struct LevelView<'a> {
    pub area: Rectangle,
    pub level_offset: Position,
    pub level: &'a Level,
    pub fov: &'a Fov,
    pub entities: &'a Entities<'a>,
    pub characters: &'a ReadStorage<'a, Character>,
    pub positions: &'a ReadStorage<'a, Position>,
    pub tileset: &'a Tileset,
    pub palette: &'a Palette,
}

impl <'a> LevelView<'a> {
    pub fn new(level: &'a Level, tileset: &'a Tileset, palette: &'a Palette, fov: &'a Fov, entities: &'a Entities, characters: &'a ReadStorage<'a, Character>, positions: &'a ReadStorage<'a, Position>) -> Self {
        LevelView {
            area: Rectangle::new_sized(Vector::ZERO),
            level_offset: Position::default(),
            level: level,
            fov: fov,
            entities: entities,
            characters: characters,
            positions: positions,
            tileset: tileset,
            palette: palette,
        }
    }

    pub fn with_area(&mut self, area: Rectangle) -> &mut Self {
        self.area = area;
        self
    }

    pub fn with_level_offset(&mut self, level_offset: Position) -> &mut Self {
        self.level_offset = level_offset;
        self
    }

    fn draw_cell(&self, window: &mut Window, palette: &Palette, pos: Vector, level_cell: &Tile, visibility: Visibility) -> Result<()> {
        if let Some(entity) = level_cell.entities.get(0) {
            let color = if visibility == Visibility::Explored {
                entity.color.darker()
            } else {
                entity.color
            }.qs_color(palette);

            self.tileset.draw(window, entity.character, pos, color)?;
        } else {
            let color = if visibility == Visibility::Explored {
                level_cell.color.darker()
            } else {
                level_cell.color
            }.qs_color(palette);

            self.tileset.draw(window, level_cell.glyph, pos, color)?;
        }

        Ok(())
    }
}

impl <'a> Widget for LevelView<'a> {
    fn area(&self) -> Rectangle {
        self.area.clone()
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        let tile_size = self.tileset.tile_size();
        let level_size = self.area.size().times(tile_size.recip());
        let level_area = Rect::new(self.level_offset.x, self.level_offset.y, level_size.x as i32, level_size.y as i32);
        let level_area = self.level.area().intersection(&level_area);
        //let draw_area = Rectangle::new(self.area.x(), self.area.y(), level_area.width * tile_size.x, level_area.height * tile_size.y);

        for level_i in 0..level_area.width {
            for level_j in 0..level_area.height {
                let level_x = level_area.x + level_i;
                let level_y = level_area.y + level_j;
                let draw_pos = self.area.top_left() + tile_size.times((level_i, level_j));

                let c = self.level.get(level_x, level_y);
                if self.fov.is_in_fov(level_x, level_y) {
                    self.draw_cell(window, self.palette, draw_pos, &c, Visibility::Visible)?;
                } else if self.fov.is_explored(level_x, level_y) {
                    self.draw_cell(window, self.palette, draw_pos, &c, Visibility::Explored)?;
                }
            }
        }

        /*for (entity, pos) in (self.characters, self.positions).join() {
            let draw_x = pos.x - level_area.x as i32 + draw_area.x as i32;
            let draw_y = pos.y - level_area.y as i32 + draw_area.y as i32;

            if draw_x >= draw_area.x as i32 && draw_x < draw_area.right() as i32 && draw_y >= draw_area.y as i32 && draw_y < draw_area.bottom() as i32 {
                if self.fov.is_in_fov(pos) {
                    let c = buf.get_mut(draw_x as u16, draw_y as u16);
                    c.set_symbol(&entity.glyph.to_string());
                }
            }
		}*/
        
        Ok(())
    }
}
/*
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
*/