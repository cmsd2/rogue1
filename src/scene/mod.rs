use quicksilver::lifecycle::{Window};
use quicksilver::Result;
use crate::qs_game::Game;

pub mod level;

pub enum Scene {
    Main,
}

impl Default for Scene {
    fn default() -> Self {
        Scene::Main
    }
}

impl Scene {
    pub fn draw(&self, window: &mut Window, game: &mut Game) -> Result<()> {
        match self {
            Scene::Main => {
                level::LevelScene.draw(window, game)?;
            },
        }

        Ok(())
    }
}