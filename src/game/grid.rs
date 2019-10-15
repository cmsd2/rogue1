use super::ecs::{Position, Rect};
use std::fmt;

#[derive(Clone, PartialEq)]
pub struct Grid<T> {
    /// The area represented by this buffer
    pub area: Rect,
    /// The content of the buffer. The length of this Vec should always be equal to area.width *
    /// area.height
    pub content: Vec<T>,
    pub start: Position,
}

impl <T> Default for Grid<T> {
    fn default() -> Self {
        Grid {
            area: Default::default(),
            content: Vec::new(),
            start: Position::default(),
        }
    }
}

impl <T> fmt::Debug for Grid<T> where T: fmt::Display {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Grid: {:?}", self.area)?;
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

impl <T> Grid<T> where T: Clone + Default {
    /// Returns a Level with all cells set to the default one
    pub fn empty(area: Rect) -> Self {
        let cell: T = Default::default();
        Self::filled(area, &cell)
    }

    /// Returns a Level with all cells initialized with the attributes of the given Tile
    pub fn filled(area: Rect, cell: &T) -> Self {
        let start = area.top_left();
        let size = area.area() as usize;
        let mut content = Vec::with_capacity(size);
        for _ in 0..size {
            content.push(cell.clone());
        }
        Self { area, content, start }
    }

    /// Returns the content of the buffer as a slice
    pub fn content(&self) -> &[T] {
        &self.content
    }

    /// Returns the area covered by this buffer
    pub fn area(&self) -> &Rect {
        &self.area
    }

    pub fn start(&self) -> Position {
        self.start.clone()
    }

    pub fn with_start(&mut self, start: Position) -> &mut Self {
        self.start = start;
        self
    }

    /// Returns a reference to Tile at the given coordinates
    pub fn get(&self, x: i32, y: i32) -> &T {
        let i = self.index_of(x, y);
        &self.content[i]
    }

    /// Returns a mutable reference to Tile at the given coordinates
    pub fn get_mut(&mut self, x: i32, y: i32) -> &mut T {
        let i = self.index_of(x, y);
        &mut self.content[i]
    }

    /// Returns the index in the Vec<Tile> for the given global (x, y) coordinates.
    ///
    /// Global coordinates are offset by the Level's area offset (`x`/`y`).
    pub fn index_of(&self, x: i32, y: i32) -> usize {
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
    pub fn pos_of(&self, i: usize) -> (i32, i32) {
        debug_assert!(
            i < self.content.len(),
            "Trying to get the coords of a cell outside the buffer: i={} len={}",
            i,
            self.content.len()
        );
        (
            self.area.x + i as i32 % self.area.width,
            self.area.y + i as i32 / self.area.width,
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
        self.start = self.area.top_left();
    }

    /// Reset all cells in the buffer
    pub fn reset(&mut self) {
        for c in &mut self.content {
            *c = Default::default();
        }
    }

    /// Merge an other buffer into this one
    pub fn merge(&mut self, other: &Grid<T>) {
        let area = self.area.union(&other.area);
        let cell: T = Default::default();
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
