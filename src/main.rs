use piston::Size;
use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use piston::input::keyboard::ModifierKey;
use glfw_window::GlfwWindow;
use opengl_graphics::{ GlGraphics, OpenGL, GlyphCache, TextureSettings };
use graphics::Graphics;
use graphics::context::Context;
use graphics::types::{Color, FontSize};
use graphics::character::CharacterCache;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::cmp;

pub mod grid;
pub mod input;
pub mod charmap;
pub mod chords;
pub mod euclid;
pub mod commands;
pub mod window;
pub mod app;

use commands::{Commands};
use grid::{Drawable, View};

pub struct Scaler {
    pub scale_x: f64,
    pub scale_y: f64,
}

impl Scaler {
    pub fn new(scale: f64) -> Scaler {
        Scaler {
            scale_x: scale,
            scale_y: scale,
        }
    }

    pub fn scale_x(&self, x: f64) -> f64 {
        x * self.scale_x as f64
    }

    pub fn scale_y(&self, y: f64) -> f64 {
        y * self.scale_y as f64
    }

    pub fn scale<S>(&self, s: S) -> Size where S: Into<Size> {
        let size = s.into();
        Size { width: self.scale_x(size.width), height: self.scale_y(size.height) }
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    let scaler = Scaler::new(2.25);

    // Create an Glutin window.
    let mut window: GlfwWindow = WindowSettings::new(
            "spinning-square",
            scaler.scale((200,200))
        )
        .graphics_api(opengl)
        .exit_on_esc(true)
        .fullscreen(false)
        .build()
        .unwrap();

    let font = "UbuntuMono-R.ttf";
    let glyphs = GlyphCache::new(font, (), TextureSettings::new()).unwrap();

    let app = Rc::new(RefCell::new(app::App::new(
        GlGraphics::new(opengl),
        Rc::new(RefCell::new(glyphs)),
        32,
        scaler.scale((200.0, 200.0)),
        [1.0, 0.0, 0.0, 1.0],
    )));

    let prompt_window_id = {
        let mut window_app = app.borrow_mut();
        let root_window = window_app.root_window();

        let main_window = {
            let mut root_window_mut = root_window.borrow_mut();
            root_window_mut.create_child_window(&mut *window_app)
        };

        let main_window_id = {
            let mut main_window = main_window.borrow_mut();
            let root_window_mut = root_window.borrow_mut();
            main_window.resize(root_window_mut.size().subtract(grid::Size { w: 0, h: 1 }));
            main_window.id()
        };
        
        let prompt_window_id = {
            let mut root_window_mut = root_window.borrow_mut();
            let prompt_window = root_window_mut.create_child_window(&mut *window_app);

            let mut prompt_window = prompt_window.borrow_mut();
            prompt_window.resize(grid::Size { w: root_window_mut.size().w, h: 1 });
            prompt_window.set_pos(grid::Pos { x: 0, y: root_window_mut.size().h as i32 - 1 });
            prompt_window.id()
        };

        window_app.focus(main_window_id);

        prompt_window_id
    };

    let input_app = app.clone();
    let mut input_handler = input::InputHandler::new((0.2,0.1), move |kev: input::InputEvent| {
        let mut app = input_app.borrow_mut();
        app.key_event(kev.state, kev.key);
    });

    let event_app = app.clone();
    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        input_handler.event(&e);

        let mut app = event_app.borrow_mut();

        {
            let keys = app.commands.chords.get_keys();
            let prompt_window = app.get_window(prompt_window_id).unwrap();
            let mut prompt_window = prompt_window.borrow_mut();
            prompt_window.cursor.pos = grid::Pos::new();
            let size = prompt_window.size();
            let msg: String = format!("{:?}", keys);
            let len = cmp::min(msg.len(), (size.w - 1) as usize);
            prompt_window.set_text(&msg[0..len]);
        }

        if let Some(r) = e.render_args() {
            app.render(&r);
        }

        if let Some(u) = e.update_args() {
            app.update(&u);
        }
    }
}