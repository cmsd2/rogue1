use std::rc::Rc;
use std::cell::RefCell;
use crate::euclid;
use crate::input;
use crate::commands;
use crate::charmap;
use crate::grid::{Pos, Size, Grid, Drawable};
use piston::UpdateArgs;
use specs::world::Entity;

pub type WindowId = Entity;

pub trait WindowFactory {
    fn create_window(&mut self) -> Rc<RefCell<Window>>;
}

pub struct Cursor {
    pub pos: Pos,
    pub character: char,
    pub visible: bool,
    pub enabled: bool,
    pub color: [f32; 4],
    pub blink_on: f64,
    pub blink_off: f64,
    pub elapsed: f64,
}

impl Cursor {
    pub fn new() -> Cursor {
        Cursor {
            pos: Pos::new(),
            character: '_',
            visible: true,
            enabled: true,
            color: [0.0, 0.0, 0.0, 1.0],
            blink_on: 0.6,
            blink_off: 0.6,
            elapsed: 0.0,
        }
    }

    pub fn update(&mut self, args: &UpdateArgs) {
        self.elapsed += args.dt;
        if self.visible {
            if self.elapsed >= self.blink_on {
                self.elapsed -= self.blink_on;
                self.visible = false;
            }
        } else {
            if self.elapsed >= self.blink_off {
                self.elapsed -= self.blink_off;
                self.visible = true;
            }
        }
    }
}

pub struct Window {
    pub cursor: Cursor,
    pub pos: Pos,
    pub blank_char: char,
    pub grid: Grid,
    pub color: [f32; 4],
    pub children: Vec<WindowId>,
    pub visible: bool,
}

impl Window {
    pub fn new(size: Size) -> Window {
        let blank_char = ' ';
        let color = [0.0, 0.0, 0.0, 1.0];

        Window {
            cursor: Cursor::new(),
            pos: Pos::new(),
            blank_char,
            color,
            grid: Grid::new(blank_char, size.w, size.h, color),
            children: vec![],
            visible: true,
        }
    }

    pub fn children<'a>(&'a self) -> &'a Vec<WindowId> {
        &self.children
    }

    pub fn children_mut<'a>(&'a mut self) -> &'a mut Vec<WindowId> {
        &mut self.children
    }

    pub fn update(&mut self, args: &UpdateArgs) {
        self.cursor.update(args);
    }

    pub fn size(&self) -> Size {
        Size {
            w: self.grid.width as u32,
            h: self.grid.height as u32,
        }
    }

    pub fn resize(&mut self, size: Size) {
        self.grid = Grid::new(self.blank_char, size.w, size.h, self.color);
    }

    pub fn set_pos(&mut self, pos: Pos) {
        self.pos = pos;
    }

    pub fn draw(&self, grid: &mut Box<dyn Drawable>) {
        grid.blit(Pos::new(), &self.grid);

        if self.cursor.visible && self.cursor.enabled {
            let pos = self.cursor.pos;
            if grid.contains(&pos) {
                grid.putc(pos.x as u32, pos.y as u32, self.cursor.character, self.cursor.color);
            }
        }
    }

    fn carriage_return(&mut self) {
        self.cursor.pos.x = 0;
    }

    fn line_feed(&mut self) {
        self.cursor.pos.y += 1;

        if self.cursor.pos.y >= self.grid.height as i32 {
            self.grid.line_feed();
            self.cursor.pos.y -= 1;
        }
    }

    fn backspace(&mut self) {
        let blank = self.grid.blank;

        if self.cursor.pos.x != 0 {
            self.cursor.pos.x = self.cursor.pos.x - 1;
            self.grid.putc(self.cursor.pos.x as u32, self.cursor.pos.y as u32, blank, self.color);
        }
    }
    
    fn move_cursor(&mut self, offset: Pos) {
        let cols = euclid::modulo(offset.x + self.cursor.pos.x as i32, self.grid.width as i32) - self.cursor.pos.x as i32;
        let rows = (offset.x - cols) / self.grid.width as i32 + offset.y;

        // println!("in ({},{})", grid.width, grid.height);
        // println!("at ({},{})", self.cursor.col, self.cursor.row);
        // println!("by ({},{})", cols, rows);

        if -rows > self.cursor.pos.y as i32 {
            //self.cursor.pos.x = 0;
            self.cursor.pos.y = 0;
        } else if (self.cursor.pos.y as i32 + rows) >= self.grid.height as i32 {
            //self.cursor.pos.x = self.grid.width as i32 - 1;
            self.cursor.pos.y = self.grid.height as i32 - 1;
        } else {
            self.cursor.pos.x = self.cursor.pos.x as i32 + cols;
            self.cursor.pos.y = self.cursor.pos.y as i32 + rows;
        }

        // println!("at ({},{})", self.cursor.col, self.cursor.row);
    }

    pub fn clear(&mut self) {
        self.cursor.pos = Pos::new();
        self.grid.clear();
    }

    pub fn set_text(&mut self, s: &str) {
        self.clear();
        self.text(s);
    }

    fn text(&mut self, args: &str) {
        let Size { w, .. } = self.size();

        for c in args.chars() {
            self.grid.putc(self.cursor.pos.x as u32, self.cursor.pos.y as u32, c, self.color);

            self.cursor.pos.x = self.cursor.pos.x + 1;

            if self.cursor.pos.x == w as i32 {
                self.carriage_return();
                self.line_feed();
            }
        }
    }

    pub fn action(&mut self, action: commands::Action) {
        use commands::Action;

        match action {
            Action::NewLine => {
                self.carriage_return();
                self.line_feed();
            },
            Action::Backspace => {
                self.backspace();
            },
            Action::Left => {
                self.move_cursor(Pos { x: -1, y: 0 });
            },
            Action::Right => {
                self.move_cursor(Pos { x: 1, y: 0 });
            },
            Action::Up => {
                self.move_cursor(Pos { x: 0, y: -1 });
            },
            Action::Down => {
                self.move_cursor(Pos { x: 0, y: 1 });
            },
            _  => {}
        }
    }

    pub fn key_event(&mut self, state: input::InputEventType, key: input::InputEventKey) {
        match state {
            input::InputEventType::KeyUp => {},
            _ => {
                match key {
                    input::InputEventKey::KeyboardKey { character: Some(c), .. } => { 
                        if charmap::is_printable(c) {
                            self.text(&[c].iter().collect::<String>());
                        }
                    },
                    _ => {}
                }
            }
        }
    }
}