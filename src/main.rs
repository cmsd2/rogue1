use piston::Size;
use piston::window::WindowSettings;
use glfw_window::GlfwWindow;
use opengl_graphics::{ GlGraphics, OpenGL };
use std::path::Path;
use specs::{World, DispatcherBuilder, WorldExt};

pub mod input;
pub mod charmap;
pub mod chords;
pub mod euclid;
pub mod commands;
pub mod app;
pub mod ai;
pub mod glfw_system;
pub mod color;
pub mod scene;
pub mod pane;
pub mod level;
pub mod level_gen;
pub mod ecs;
pub mod features;
pub mod level_view;
pub mod events;
pub mod game;

use commands::{Commands};
use app::App;
use glfw_system::GlfwSystem;
use ai::system::AiSystem;
use ai::Ai;
use input::InputHandler;
use game::{GameActor, GameSystem, GameEventQueue};
use events::Time;

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

fn create_window(opengl: OpenGL, scaler: Scaler) -> GlfwWindow {
    WindowSettings::new(
            "spinning-square",
            scaler.scale((200,200))
        )
        .graphics_api(opengl)
        .exit_on_esc(true)
        .fullscreen(false)
        .build()
        .unwrap()
}
/*
fn create_app<'a>(opengl: OpenGL, scaler: Scaler) -> App<'a> {
    let font = "UbuntuMono-R.ttf";
    let glyphs = GlyphCache::new(font, (), TextureSettings::new()).unwrap();

    let app = app::App::new(
        GlGraphics::new(opengl),
        Rc::new(RefCell::new(glyphs)),
        32,
        scaler.scale((200.0, 200.0)),
        [1.0, 0.0, 0.0, 1.0],
    );

    app
}

fn create_main_window(app: Rc<RefCell<App>>) -> WindowId {
    let mut window_app = app.borrow_mut();
    let root_window = window_app.root_window();
    let mut root_window_mut = root_window.borrow_mut();
    let main_window = root_window_mut.create_child_window(&mut *window_app);

    let main_window_mut = main_window.borrow_mut();
    main_window_mut.resize(root_window_mut.size().subtract(grid::Size { w: 0, h: 1 }));
    window_app.focus(main_window_mut.id());
    main_window_mut.id()
}

fn create_prompt_window(app: Rc<RefCell<App>>) -> WindowId {
    let mut window_app = app.borrow_mut();
    let root_window = window_app.root_window();
    let mut root_window_mut = root_window.borrow_mut();
    let prompt_window = root_window_mut.create_child_window(&mut *window_app);

    let mut prompt_window_mut = prompt_window.borrow_mut();
    prompt_window_mut.resize(grid::Size { w: root_window_mut.size().w, h: 1 });
    prompt_window_mut.set_pos(grid::Pos { x: 0, y: root_window_mut.size().h as i32 - 1 });
    prompt_window_mut.id()
}
*/

fn main() {
    let scaler = Scaler::new(2.25);
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let window = create_window(opengl, scaler);

    let mut world = World::new();
    let mut dispatcher = DispatcherBuilder::new()
        // .with(PlayerSystem, "player_system", &[])
        .with(AiSystem, "ai_system", &[])
        .with(GameSystem::new(), "game_system", &[])
        .with_thread_local(GlfwSystem::new(window, GlGraphics::new(opengl), Path::new("UbuntuMono-R.ttf"), 32, [1.0, 0.0, 0.0, 1.0]))
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

    let mut app = App::default();
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