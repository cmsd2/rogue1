use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone, Copy, Debug)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub fn new() -> Pos {
        Pos {
            x: 0,
            y: 0,
        }
    }

    pub fn add<S>(&self, size: S) -> Pos where S: Into<Size> {
        let size = size.into();
        Pos {
            x: self.x + size.w as i32,
            y: self.y + size.h as i32,
        }
    }

    pub fn subtract<S>(&self, size: S) -> Pos where S: Into<Size>{
        let size = size.into();
        Pos {
            x: self.x - size.w as i32,
            y: self.y - size.h as i32,
        }
    }
}

impl Into<Size> for Pos {
    fn into(self) -> Size {
        assert!(self.x >= 0);
        assert!(self.y >= 0);

        Size {
            w: self.x as u32,
            h: self.y as u32,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Size {
    pub w: u32,
    pub h: u32,
}

impl Size {
    pub fn new() -> Size {
        Size {
            w: 0,
            h: 0,
        }
    }

    pub fn add<S>(&self, s: S) -> Size where S: Into<Size> {
        let s = s.into();
        Size {
            w: self.w + s.w,
            h: self.h + s.h,
        }
    }

    pub fn subtract<S>(&self, s: S) -> Size where S: Into<Size> {
        let s = s.into();
        Size {
            w: self.w - s.w,
            h: self.h - s.h,
        }
    }
}

impl Into<piston::Size> for Size {
    fn into(self) -> piston::Size {
        piston::Size {
            width: self.w as f64,
            height: self.h as f64,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub pos: Pos,
    pub size: Size,
}

pub struct View {
    pub grid: Rc<RefCell<Box<dyn Drawable>>>,
    pub pos: Pos,
    pub size: Size,
}

impl View {
    pub fn new(grid: Rc<RefCell<Box<dyn Drawable>>>, pos: Pos, size: Size) -> View {
        let grid_size = { grid.borrow().size() };
        
        assert!(pos.x >= 0);
        assert!(pos.y >= 0);
        assert!((pos.x + size.w as i32) <= grid_size.w as i32);
        assert!((pos.y + size.h as i32) <= grid_size.h as i32);

        View {
            grid,
            pos,
            size,
        }
    }
}

pub trait Drawable { 
    fn contains(&self, pos: &Pos) -> bool {
        let size = self.size();
        pos.x >= 0 && pos.x < size.w as i32 && pos.y >= 0 && pos.y < size.h as i32
    }

    fn blit<'a>(&mut self, pos: Pos, source: &'a dyn Drawable) {
        let source_size = source.size();
        for x in 0..source_size.w {
            for y in 0..source_size.h {
                let pos = pos.add(Size { w: x, h: y });
                if self.contains(&pos) {
                    let (character, color) = source.getc(x, y);

                    self.putc(pos.x as u32, pos.y as u32, character, color);
                }
            }
        }
    }

    fn size(&self) -> Size;
    fn getc(&self, col: u32, row: u32) -> (char, [f32; 4]);
    fn putc(&mut self, col: u32, row: u32, c: char, color: [f32; 4]);
    fn boxed(self) -> Box<dyn Drawable>;
}

impl Drawable for View {
    fn size(&self) -> Size {
        self.size
    }

    fn getc(&self, col: u32, row: u32) -> (char, [f32; 4]) {
        self.grid.borrow().getc(col + self.pos.x as u32, row + self.pos.y as u32)
    }

    fn putc(&mut self, col: u32, row: u32, c: char, color: [f32; 4]) {
        self.grid.borrow_mut().putc(col + self.pos.x as u32, row + self.pos.y as u32, c, color)
    }

    fn boxed(self) -> Box<dyn Drawable> {
        Box::new(self)
    }
}

#[derive(Copy, Clone)]
pub struct GridCell {
    pub c: char,
    pub color: [f32; 4],
}

pub struct Grid {
    rows: Vec<Vec<GridCell>>,
    pub color: [f32; 4],
    pub blank: char,
    pub width: u32,
    pub height: u32,
}

impl Default for Grid {
    fn default() -> Self {
        Grid::new(' ', 0, 0, [0.0, 0.0, 0.0, 1.0])
    }
}

impl Grid {
    pub fn new(blank: char, width: u32, height: u32, color: [f32; 4]) -> Self {
        let mut rows = vec![];
        for _i in 0..height {
            let mut row = vec![];
            row.resize(width as usize, GridCell { c: blank, color: color });
            rows.push(row.into_iter().collect());
        }

        Grid { 
            blank,
            rows,
            width,
            height,
            color,
        }
    }

    pub fn clear(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.putc(x, y, self.blank, self.color);
            }
        }
    }

    pub fn line_feed(&mut self) {
        self.rows.remove(0);
        
        let mut row = vec![];
        row.resize(self.width as usize, GridCell { c: self.blank, color: self.color });
        self.rows.push(row.into_iter().collect());
    }
}

impl Drawable for Grid {
    fn contains(&self, pos: &Pos) -> bool {
        pos.x >= 0 && pos.x < self.width as i32 && pos.y >= 0 && pos.y < self.height as i32
    }

    fn getc(&self, col: u32, row: u32) -> (char, [f32; 4]) {
        assert!(col < self.width);
        assert!(row < self.height);

        let cell = self.rows.get(row as usize).unwrap().get(col as usize).unwrap();

        (cell.c, cell.color)
    }

    fn putc(&mut self, col: u32, row: u32, c: char, color: [f32; 4]) {
        assert!(col < self.width);
        assert!(row < self.height);

        let cell = self.rows.get_mut(row as usize).unwrap().get_mut(col as usize).unwrap();
        cell.c = c;
        cell.color = color;
    }

    fn size(&self) -> Size {
        Size {
            w: self.width,
            h: self.height,
        }
    }

    fn boxed(self) -> Box<dyn Drawable> {
        Box::new(self)
    }
}