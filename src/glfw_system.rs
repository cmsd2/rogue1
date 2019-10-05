use specs::{Entities, System, Write, ReadStorage};
use piston;
use piston::Window as PistonWindow;
use glfw_window::GlfwWindow;
use piston::event_loop::*;
use piston::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use graphics::character::CharacterCache;
use graphics::types::{FontSize};
use opengl_graphics::{ GlGraphics, GlyphCache, TextureSettings };
use std::rc::Rc;
use std::cell::{RefCell};
use std::path::Path;
use tui::Terminal;
use tui::backend::TestBackend;
use tui::layout::Rect;
use crate::app::App;
use crate::input::{InputHandler};
use crate::color::ColorMap;
use crate::level::Level;
use crate::ecs::{Character, Position, PlayerController};

pub struct RenderContext<'a,'b> {
    pub level: &'a mut Level,
    pub entities: &'a Entities<'b>,
    pub characters: &'a ReadStorage<'b, Character>,
    pub positions: &'a ReadStorage<'b, Position>,
}

pub struct GlfwSystem {
    window: GlfwWindow,
    terminal: Terminal<TestBackend>,
    gl: GlGraphics,
    glyphs: Rc<RefCell<GlyphCache<'static>>>,
    font_size: FontSize,
    events: Events,
    color_map: Rc<ColorMap>,
    cell_size: piston::Size,
    known_size: Option<piston::Size>,
}

impl GlfwSystem {
    pub fn new(window: GlfwWindow, gl: GlGraphics, font: &Path, font_size: FontSize, color: [f32; 4]) -> Self {

        let mut system = GlfwSystem {
            window: window,
            gl: gl,
            glyphs: Rc::new(RefCell::new(GlyphCache::new(font, (), TextureSettings::new()).unwrap())),
            font_size: font_size,
            events: Events::new(EventSettings::new()),
            terminal: Terminal::new(TestBackend::new(1,1)).expect("terminal"),
            color_map: Rc::new(ColorMap { default_fg: color }),
            cell_size: piston::Size { width: 0.0, height: 0.0 },
            known_size: None,
        };

        system.resize();

        system
    }

    fn resize(&mut self) {
        let size = self.window.draw_size();

        if Some(size) != self.known_size {
            self.cell_size = self.calc_cell_size();
            let (cols,rows) = self.calc_grid_dimensions(size, self.cell_size);
            self.terminal.resize(Rect::new(0, 0, cols as u16, rows as u16)).expect("resize");
            *self.terminal.backend_mut() = TestBackend::new(cols as u16, rows as u16);

            println!("cell size: {} x {}", self.cell_size.width, self.cell_size.height);
            println!("grid size: {} x {}", cols, rows);
            println!("term size: {} x {}", self.terminal.size().unwrap().width, self.terminal.size().unwrap().height);

            self.known_size = Some(size);
        }
    }

    fn calc_cell_size(&mut self) -> piston::Size {
        let cell_width = self.measure("█");

        piston::Size {
            width: cell_width,
            height: self.font_size as f64,
        }
    }

    fn calc_grid_dimensions(&self, draw_area: piston::Size, cell_size: piston::Size) -> (u32, u32) {
        let rows = (draw_area.height / cell_size.height) as u32;
        let cols = (draw_area.width / cell_size.width) as u32;

        (cols, rows)
    }

    fn measure(&mut self, text: &str) -> f64 {
        let mut glyphs = self.glyphs.borrow_mut();
        glyphs.width(self.font_size, text).unwrap()
    }

    fn paint_windows<'a,'b>(&mut self, app: &mut Write<'b, App>, render_context: RenderContext<'a,'b>) {
        self.terminal.draw(|mut frame| {
            let size = frame.size();
            app.scene.render(&mut frame, size, render_context);
        }).expect("render");
    }
    
    fn render(&mut self, args: RenderArgs) {
        use graphics::*;

        let buffer = self.terminal.backend().buffer();
        let size = self.terminal.size().expect("size");
        let mut glyphs = self.glyphs.borrow_mut();
        let mut s = String::new();
        let cell_size = self.cell_size;
        let color_map = self.color_map.clone();
        let font_size = self.font_size;

        self.gl.draw(args.viewport(), |ctx, gl| {
            clear([0.0, 0.0, 0.0, 1.0], gl);

            for i in 0..size.width {
                for j in 0..size.height {
                    let cell = buffer.get(i, j);
                    let color = color_map.lookup_tui(cell.style.fg);
                    let transform = ctx.transform.trans(i as f64 * cell_size.width, (j + 1) as f64 * cell_size.height);

                    s.clear();
                    s.push_str(&cell.symbol);

                    graphics::text::Text::new_color(color, font_size)
                            .draw(&s, &mut *glyphs, &ctx.draw_state, transform, gl)
                            .unwrap();
                }
            }
        });
    }

    fn update(&mut self, _args: UpdateArgs) {

    }
}

impl <'a> System<'a> for GlfwSystem {
    type SystemData = (Entities<'a>, Write<'a, Level>, Write<'a, App>, Write<'a, InputHandler>, ReadStorage<'a, Character>, ReadStorage<'a, Position>, ReadStorage<'a, PlayerController>);

    fn run(&mut self, (entities, mut level, mut app, mut input_handler, characters, positions, _player_controlled): Self::SystemData) {
        //use specs::Join;

        self.resize();

        if let Some(e) = self.events.next(&mut self.window) {
            if let Some(r) = e.render_args() {
                let render_context = RenderContext {
                    level: &mut level,
                    entities: &entities,
                    characters: &characters,
                    positions: &positions,
                };

                self.paint_windows(&mut app, render_context);
                self.render(r);
            }

            if let Some(u) = e.update_args() {
                self.update(u);
            }

            for kev in input_handler.event(&e) {
                println!("[{:?}] input: {:?}", app.time, kev);
                app.key_event(kev.state, kev.key);
            }
        } else {
            app.finish();
        }
    }
}
