use piston::input::*;
use opengl_graphics::{ GlGraphics, GlyphCache };
use graphics::Graphics;
use graphics::types::{FontSize};
use graphics::character::CharacterCache;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::commands::Commands;
use crate::window::{Id as WindowId, Window, WindowFactory};
use crate::grid::{Drawable, View, Grid, Size, Pos};
use crate::chords::{ChordResult};
use crate::input::{InputEventType, InputEventKey};

pub struct App<'a> {
    gl: GlGraphics, // OpenGL drawing backend.
    glyphs: Rc<RefCell<GlyphCache<'a>>>,
    grid: Rc<RefCell<Box<dyn Drawable>>>,
    color: [f32; 4],
    font_size: FontSize,
    cell_size: piston::Size,
    view_size: Size,
    pub commands: Commands,
    root_window: Rc<RefCell<Window>>,
    focus_window_id: Option<WindowId>,
    windows: Rc<RefCell<HashMap<WindowId, Rc<RefCell<Window>>>>>,
}

impl <'a> WindowFactory for App<'a> { 
    fn create_window(&mut self) -> Rc<RefCell<Window>> {
        let wnd = Window::new(Size {
            w: 0,
            h: 0,
        });
        let id = wnd.id();
        let wnd = Rc::new(RefCell::new(wnd));
        self.windows.borrow_mut().insert(id, wnd.clone());
        wnd
    }
}

impl <'a> App<'a> {
    pub fn new(gl: GlGraphics, glyphs: Rc<RefCell<GlyphCache<'a>>>, font_size: FontSize, draw_area: piston::Size, color: [f32; 4]) -> Self {
        let cell_size = Self::calc_cell_size(&mut glyphs.borrow_mut(), font_size);
        let (cols,rows) = Self::calc_grid_dimensions(draw_area, cell_size);
        let grid = Rc::new(RefCell::new(Grid::new(' ', cols, rows, color).boxed()));
        let commands = Commands::new();
        let root_window = Rc::new(RefCell::new(Window::new(Size { w: cols, h: rows })));
        let mut windows = HashMap::new();
        windows.insert(root_window.borrow().id(), root_window.clone());
        let windows = Rc::new(RefCell::new(windows));
        let view_size = Size { w: draw_area.width as u32, h: draw_area.height as u32 };

        App {
            gl,
            glyphs,
            grid,
            font_size,
            cell_size,
            view_size,
            color,
            commands,
            root_window,
            focus_window_id: None,
            windows,
        }
    }

    pub fn resize(&mut self, new_size: Size) {
        let (cols,rows) = Self::calc_grid_dimensions(new_size.into(), self.cell_size);
        let grid = Rc::new(RefCell::new(Grid::new(' ', cols, rows, self.color).boxed()));
        self.grid = grid;
        self.root_window.borrow_mut().resize(new_size);
        self.view_size = new_size;
    }

    pub fn focus(&mut self, window_id: WindowId) {
        self.blur();

        if let Some(window) = self.get_window(window_id) {
            self.focus_window_id = Some(window_id);
            window.borrow_mut().cursor.enabled = true;
        }
    }

    pub fn blur(&mut self) {
        self.get_focussed_window().borrow_mut().cursor.enabled = false;

        self.focus_window_id = None;
    }

    pub fn root_window(&self) -> Rc<RefCell<Window>> {
        self.root_window.clone()
    }
    
    fn calc_cell_size(glyphs: &mut GlyphCache, size: FontSize) -> piston::Size {
        let cell_width = Self::measure(glyphs, size, "M");

        piston::Size {
            width: cell_width,
            height: size as f64,
        }
    }

    fn calc_grid_dimensions(draw_area: piston::Size, cell_size: piston::Size) -> (u32, u32) {
        let rows = (draw_area.height / cell_size.height) as u32;
        let cols = (draw_area.width / cell_size.width) as u32;

        (cols, rows)
    }

    fn measure(glyphs: &mut GlyphCache, size: FontSize, text: &str) -> f64 {
        (glyphs as &mut dyn CharacterCache<Texture=<opengl_graphics::GlGraphics as Graphics>::Texture,Error=String>).width(size, text).unwrap()
    }

    fn draw_windows(root_window: Rc<RefCell<Window>>, windows: Rc<RefCell<HashMap<WindowId,Rc<RefCell<Window>>>>>, grid: Rc<RefCell<Box<dyn Drawable>>>) {
        let root_window = root_window.borrow_mut();

        {
            root_window.draw(&mut grid.borrow_mut());
        }

        for wnd_id in root_window.children() {
            if let Some(window) = { windows.borrow_mut().get(wnd_id) } {
                let (pos, size) = {
                    let window = window.borrow();
                    (window.pos, window.size())
                };
                let mut view = Rc::new(RefCell::new(View::new(grid.clone(), pos, size).boxed()));
                Self::draw_windows(window.clone(), windows.clone(), view);
            }
        }
    }

    pub fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const RED:   [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let cell_width = self.cell_size.width;
        let cell_height = self.cell_size.height;

        let glyphs = self.glyphs.clone();
        let grid = self.grid.clone();
        let font_size = self.font_size;
        let root_window = self.root_window.clone();
        let windows = self.windows.clone();
        let view_size = Size { w: args.window_size[0] as u32, h: args.window_size[1] as u32 };
        
        if view_size != self.view_size {
            self.resize(view_size);
        }

        self.gl.draw(args.viewport(), |ctx, gl| {
            // Clear the screen.
            clear(GREEN, gl);
            let mut s = String::new();

            Self::draw_windows(root_window, windows, grid.clone());

            let grid = grid.borrow_mut();
            let mut glyphs = glyphs.borrow_mut();
            let grid_size = grid.size();

            for x in 0..grid_size.w {
                for y in 0..grid_size.h {
                    let transform = ctx.transform.trans(x as f64 * cell_width, (y + 1) as f64 * cell_height);

                    let (character, color) = grid.getc(x, y);

                    s.clear();
                    s.push(character);

                    graphics::text::Text::new_color(color, font_size)
                            .draw(&s, &mut *glyphs, &ctx.draw_state, transform, gl)
                            .unwrap();
                }
            }
        });
    }

    pub fn update(&mut self, args: &UpdateArgs) {
        let mut root_window = self.root_window.borrow_mut();

        root_window.update(args);
    }

    pub fn get_window(&self, id: WindowId) -> Option<Rc<RefCell<Window>>> {
        self.windows.borrow().get(&id).map(|w| w.to_owned())
    }

    pub fn get_focussed_window(&self) -> Rc<RefCell<Window>> {
        self.focus_window_id.and_then(|id| self.get_window(id)).unwrap_or(self.root_window())
    }

    pub fn key_event(&mut self, state: InputEventType, key: InputEventKey) {
        let focussed_window = self.get_focussed_window();
        let mut focussed_window = focussed_window.borrow_mut();

        match state {
            InputEventType::KeyUp => {},
            _ => {
                if let Some(action) = self.commands.key_event(state, key) {
                    println!("action: {:?}", action);
                    match action {
                        ChordResult::Action(action) => {
                            focussed_window.action(action);
                        },
                        ChordResult::Building => {}
                    }
                } else {
                    focussed_window.key_event(state, key);
                }
            }
        }
    }

    fn grid_size(&self) -> (u32, u32) {
        let grid = self.grid.borrow().size();
        (grid.w, grid.h)
    }

}
