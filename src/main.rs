#[macro_use]
extern crate slog;
use glfw_window::GlfwWindow;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::window::WindowSettings;
use piston::Size;
use slog::Drain;
use specs::{DispatcherBuilder, World, WorldExt};
use std::path::Path;
use std::sync::Arc;

pub mod ai;
pub mod charmap;
pub mod chords;
pub mod color;
pub mod commands;
pub mod euclid;
pub mod game;
pub mod glfw_system;
pub mod input;
pub mod ui;

use ai::Ai;
use commands::Commands;
use game::app::App;
use game::ecs;
use game::events::Time;
use game::level;
use game::level_gen;
use game::system::{GameActor, GameEventQueue, GameSystem};
use glfw_system::GlfwSystem;
use input::InputHandler;

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

    pub fn scale<S>(&self, s: S) -> Size
    where
        S: Into<Size>,
    {
        let size = s.into();
        Size {
            width: self.scale_x(size.width),
            height: self.scale_y(size.height),
        }
    }
}

fn create_window(opengl: OpenGL, scaler: Scaler) -> GlfwWindow {
    WindowSettings::new("spinning-square", scaler.scale((200, 200)))
        .graphics_api(opengl)
        .exit_on_esc(true)
        .fullscreen(false)
        .build()
        .unwrap()
}

fn main() {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_envlogger::new(drain);
    let drain = slog_async::Async::new(drain).build().fuse();
    let log = Arc::new(slog::Logger::root(drain, o!()));

    let scaler = Scaler::new(2.25);
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let window = create_window(opengl, scaler);

    let mut world = World::new();
    let mut dispatcher = DispatcherBuilder::new()
        // .with(PlayerSystem, "player_system", &[])
        //.with(AiSystem::new(log.clone()), "ai_system", &[])
        .with(GameSystem::new(log.clone()), "game_system", &[])
        .with_thread_local(GlfwSystem::new(
            log.clone(),
            window,
            GlGraphics::new(opengl),
            Path::new("UbuntuMono-R.ttf"),
            32,
            [1.0, 0.0, 0.0, 1.0],
        ))
        .build();
    world.register::<ecs::Position>();
    world.register::<ecs::Character>();
    world.register::<ecs::Attributes>();
    world.register::<ecs::AiController>();
    world.register::<ecs::Fighter>();

    dispatcher.setup(&mut world);

    world.insert(InputHandler::default());
    world.insert(Commands::default());
    world.insert(GameEventQueue::default());
    world.insert(log.clone());

    let mut app = App::new(log.clone());
    let mut level = level::Level::empty(tui::layout::Rect::new(0, 0, 80, 25));
    let entities = level_gen::make_map(&color::ColorMap::default(), &mut level, &mut world);
    let player_entity = level_gen::create_player(&mut level, &mut world);
    app.schedule_turn(Time::default(), GameActor::Player(player_entity));
    for entity in entities {
        app.schedule_turn(Time::default(), GameActor::NonPlayer(entity));
    }

    world.insert(level);
    world.insert(app);
    world.insert(Ai::default());

    loop {
        dispatcher.dispatch(&mut world);
        world.maintain();

        {
            let app = world.read_resource::<App>();
            if app.is_finished() {
                break;
            }
        }
    }
    /*
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
    */
}
