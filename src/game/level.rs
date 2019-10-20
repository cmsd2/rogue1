use std::fmt;
use specs::Entity as SpecsEntity;
use crate::color::{Color, Hue};
use crate::game::grid::Grid;

pub const BLANK: char = ' ';
pub const LINE_BLOCK: char = '#';
pub const LINE_LIGHT_BOX: char = '☐';
pub const LINE_LIGHT_DOWN_AND_RIGHT: char = '┌';
pub const LINE_LIGHT_VERTICAL: char = '│';
pub const LINE_LIGHT_UP_AND_RIGHT: char = '└';
pub const LINE_LIGHT_HORIZONTAL: char = '─';
pub const LINE_LIGHT_UP_AND_LEFT: char = '┘';
pub const LINE_LIGHT_DOWN_AND_LEFT: char = '┐';
pub const LINE_LIGHT_VERTICAL_AND_RIGHT: char = '├';
pub const LINE_LIGHT_VERTICAL_AND_LEFT: char = '┤';
pub const LINE_LIGHT_HORIZONTAL_AND_UP: char = '┴';
pub const LINE_LIGHT_HORIZONTAL_AND_DOWN: char = '┬';
pub const LINE_LIGHT_VERTICAL_AND_HORIZONTAL: char = '┼';
pub const LINE_LIGHT_LEFT: char = '╴';
pub const LINE_LIGHT_RIGHT: char = '╶';
pub const LINE_LIGHT_UP: char = '╵';
pub const LINE_LIGHT_DOWN: char = '╷';
pub const MIDDLE_DOT: char = '.';
pub const GLYPHS: &'static [char] = &[
            BLANK,
            LINE_BLOCK,
            LINE_LIGHT_DOWN_AND_RIGHT,
            LINE_LIGHT_BOX,
            LINE_LIGHT_VERTICAL,
            LINE_LIGHT_UP_AND_RIGHT,
            LINE_LIGHT_HORIZONTAL,
            LINE_LIGHT_UP_AND_LEFT,
            LINE_LIGHT_DOWN_AND_LEFT,
            LINE_LIGHT_VERTICAL_AND_RIGHT,
            LINE_LIGHT_VERTICAL_AND_LEFT,
            LINE_LIGHT_HORIZONTAL_AND_UP,
            LINE_LIGHT_HORIZONTAL_AND_DOWN,
            LINE_LIGHT_VERTICAL_AND_HORIZONTAL,
            LINE_LIGHT_LEFT,
            LINE_LIGHT_RIGHT,
            LINE_LIGHT_UP,
            LINE_LIGHT_DOWN,
            MIDDLE_DOT,
        ];

#[derive(Debug, Clone, PartialEq)]
pub struct Entity {
    pub character: char,
    pub blocked: bool,
    pub id: SpecsEntity,
    pub color: Color,
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.character)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TileType {
    Floor,
    Wall,
    Void,   
}

impl Default for TileType {
    fn default() -> Self {
        TileType::Void
    }
}

/// A buffer cell
#[derive(Debug, Clone, PartialEq)]
pub struct Tile {
    pub entities: Vec<Entity>,
    pub blocked: bool,
    pub block_sight: bool,
    pub cell_type: TileType,
    pub glyph: char,
    pub color: Color,
}

impl Tile {
    pub fn reset(&mut self) {
        self.entities.clear();
    }

    pub fn floor() -> Self {
        Tile {
            cell_type: TileType::Floor,
            glyph: MIDDLE_DOT,
            ..Default::default()
        }
    }

    pub fn wall() -> Self {
        Tile {
            blocked: true,
            block_sight: true,
            glyph: LINE_BLOCK,
            cell_type: TileType::Wall,
            ..Default::default()
        }
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    pub fn remove_entity(&mut self, id: SpecsEntity) -> Option<Entity> {
        if let Some(index) = self.entities.iter().position(|e| e.id == id) {
            Some(self.entities.remove(index))
        } else {
            None
        }
    }
}

impl Default for Tile {
    fn default() -> Tile {
        Tile {
            entities: vec![],
            blocked: false,
            block_sight: false,
            glyph: BLANK,
            cell_type: TileType::default(),
            color: Color::new(Hue::White, 128),
        }
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(entity) = self.entities.get(0) {
            write!(f, "{}", entity.character)
        } else {
            write!(f, "{}", self.glyph)
        }
    }
}

pub type Level = Grid<Tile>;

pub trait EntityGrid {
    fn move_entity(&mut self, id: SpecsEntity, x1: i32, y1: i32, x2: i32, y2: i32);
}

impl EntityGrid for Level {
    fn move_entity(&mut self, id: SpecsEntity, x1: i32, y1: i32, x2: i32, y2: i32) {
        if let Some(entity) = self.get_mut(x1, y1).remove_entity(id) {
            self.get_mut(x2, y2).add_entity(entity);
        }
    }
}
/*
#[derive(Clone, PartialEq)]
pub struct Level {
    /// The area represented by this buffer
    pub area: Rect,
    /// The content of the buffer. The length of this Vec should always be equal to area.width *
    /// area.height
    pub content: Vec<Tile>,
    pub start: (u16, u16),
}

impl Default for Level {
    fn default() -> Level {
        Level {
            area: Default::default(),
            content: Vec::new(),
            start: (0, 0),
        }
    }
}

impl fmt::Debug for Level {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Level: {:?}", self.area)?;
        f.write_str("Content (quoted lines):\n")?;
        for cells in self.content.chunks(self.area.width as usize) {
            let mut line = String::new();
            for (_x, c) in cells.iter().enumerate() {
                line.push_str(&format!("{}", c));
            }
            f.write_fmt(format_args!("{:?},", line))?;
            f.write_str("\n")?;
        }
        Ok(())
    }
}

impl Level {
    pub fn glyphs() -> &'static [char] {
        GLYPHS
    }

    /// Returns a Level with all cells set to the default one
    pub fn empty(area: Rect) -> Level {
        let cell: Tile = Default::default();
        Level::filled(area, &cell)
    }

    /// Returns a Level with all cells initialized with the attributes of the given Tile
    pub fn filled(area: Rect, cell: &Tile) -> Level {
        let start = (area.left(), area.top());
        let size = area.area() as usize;
        let mut content = Vec::with_capacity(size);
        for _ in 0..size {
            content.push(cell.clone());
        }
        Level { area, content, start }
    }

    /// Returns the content of the buffer as a slice
    pub fn content(&self) -> &[Tile] {
        &self.content
    }

    /// Returns the area covered by this buffer
    pub fn area(&self) -> &Rect {
        &self.area
    }

    pub fn start(&self) -> (u16, u16) {
        self.start
    }

    pub fn with_start(&mut self, x: u16, y: u16) -> &mut Self {
        self.start = (x, y);
        self
    }

    pub fn move_entity(&mut self, id: SpecsEntity, x1: u16, y1: u16, x2: u16, y2: u16) {
        if let Some(entity) = self.get_mut(x1, y1).remove_entity(id) {
            self.get_mut(x2, y2).add_entity(entity);
        }
    }

    /// Returns a reference to Tile at the given coordinates
    pub fn get(&self, x: u16, y: u16) -> &Tile {
        let i = self.index_of(x, y);
        &self.content[i]
    }

    /// Returns a mutable reference to Tile at the given coordinates
    pub fn get_mut(&mut self, x: u16, y: u16) -> &mut Tile {
        let i = self.index_of(x, y);
        &mut self.content[i]
    }

    /// Returns the index in the Vec<Tile> for the given global (x, y) coordinates.
    ///
    /// Global coordinates are offset by the Level's area offset (`x`/`y`).
    pub fn index_of(&self, x: u16, y: u16) -> usize {
        debug_assert!(
            x >= self.area.left()
                && x < self.area.right()
                && y >= self.area.top()
                && y < self.area.bottom(),
            "Trying to access position outside the buffer: x={}, y={}, area={:?}",
            x,
            y,
            self.area
        );
        ((y - self.area.y) * self.area.width + (x - self.area.x)) as usize
    }

    /// Returns the (global) coordinates of a cell given its index
    ///
    /// Global coordinates are offset by the Level's area offset (`x`/`y`).
    pub fn pos_of(&self, i: usize) -> (u16, u16) {
        debug_assert!(
            i < self.content.len(),
            "Trying to get the coords of a cell outside the buffer: i={} len={}",
            i,
            self.content.len()
        );
        (
            self.area.x + i as u16 % self.area.width,
            self.area.y + i as u16 / self.area.width,
        )
    }

    /// Resize the buffer so that the mapped area matches the given area and that the buffer
    /// length is equal to area.width * area.height
    pub fn resize(&mut self, area: Rect) {
        let length = area.area() as usize;
        if self.content.len() > length {
            self.content.truncate(length);
        } else {
            self.content.resize(length, Default::default());
        }
        self.area = area;
        self.start = (self.area.left(), self.area.top());
    }

    /// Reset all cells in the buffer
    pub fn reset(&mut self) {
        for c in &mut self.content {
            c.reset();
        }
    }

    /// Merge an other buffer into this one
    pub fn merge(&mut self, other: &Level) {
        let area = self.area.union(other.area);
        let cell: Tile = Default::default();
        self.content.resize(area.area() as usize, cell.clone());

        // Move original content to the appropriate space
        let size = self.area.area() as usize;
        for i in (0..size).rev() {
            let (x, y) = self.pos_of(i);
            // New index in content
            let k = ((y - area.y) * area.width + x - area.x) as usize;
            if i != k {
                self.content[k] = self.content[i].clone();
                self.content[i] = cell.clone();
            }
        }

        // Push content of the other buffer into this one (may erase previous
        // data)
        let size = other.area.area() as usize;
        for i in 0..size {
            let (x, y) = other.pos_of(i);
            // New index in content
            let k = ((y - area.y) * area.width + x - area.x) as usize;
            self.content[k] = other.content[i].clone();
        }
        self.area = area;
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::ecs::Rect;

    #[test]
    fn it_translates_to_and_from_coordinates() {
        let rect = Rect::new(200, 100, 50, 80);
        let buf = Level::empty(rect);

        // First cell is at the upper left corner.
        assert_eq!(buf.pos_of(0), (200, 100));
        assert_eq!(buf.index_of(200, 100), 0);

        // Last cell is in the lower right.
        assert_eq!(buf.pos_of(buf.content.len() - 1), (249, 179));
        assert_eq!(buf.index_of(249, 179), buf.content.len() - 1);
    }

    #[test]
    #[should_panic(expected = "outside the buffer")]
    fn pos_of_panics_on_out_of_bounds() {
        let rect = Rect::new(0, 0, 10, 10);
        let buf = Level::empty(rect);

        // There are a total of 100 cells; zero-indexed means that 100 would be the 101st cell.
        buf.pos_of(100);
    }

    #[test]
    #[should_panic(expected = "outside the buffer")]
    fn index_of_panics_on_out_of_bounds() {
        let rect = Rect::new(0, 0, 10, 10);
        let buf = Level::empty(rect);

        // width is 10; zero-indexed means that 10 would be the 11th cell.
        buf.index_of(10, 0);
    }
}
