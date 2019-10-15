#[macro_use]
extern crate log;

pub mod ai;
pub mod qs_game;
pub mod qs_ui;
pub mod scene;
pub mod data;
pub mod game;
pub mod color;
pub mod tween;

use quicksilver::lifecycle::{run, Settings};
use quicksilver::graphics::{ImageScaleStrategy, ResizeStrategy};
use quicksilver::geom::Vector;
use qs_game::Game;

fn main() {
    std::env::set_var("WINIT_HIDPI_FACTOR", "1.0");
    std::env::set_var("WINIT_UNIX_BACKEND", "x11");

    env_logger::init();

    let settings = Settings {
        // If the graphics do need to be scaled (e.g. using
        // `with_center`), blur them. This looks better with fonts.
        scale: ImageScaleStrategy::Blur,
        resize: ResizeStrategy::Stretch,
        ..Default::default()
    };

    info!("starting");
    run::<Game>("Quicksilver Roguelike", Vector::new(800, 600), settings);
    info!("stopped");
}