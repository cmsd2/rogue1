use quicksilver::graphics::Color;
use quicksilver::lifecycle::Window;
use quicksilver::geom::Rectangle;
use quicksilver::Result;
use crate::data::Data;
use crate::qs_ui::{LevelView, Widget};
use specs::{Entities, ReadStorage, ReadExpect};
use crate::game::ecs::{Position, Character};
use crate::qs_game::Game;

pub struct LevelScene;

type SystemData<'a> = (ReadExpect<'a, Data>, Entities<'a>, ReadStorage<'a, Position>, ReadStorage<'a, Character>);

impl LevelScene {
    pub fn draw(&self, window: &mut Window, game: &mut Game) -> Result<()> {
        //use specs::Join;

        window.clear(Color::BLACK)?;

        let (data, entities, positions, characters): SystemData = game.world.system_data();

        let level = &data.level;
        let fov = &data.fov;
        let tileset = &mut game.tileset;
        let palette = &data.palette;

        tileset.execute(|tileset| {
            LevelView::new(level, &tileset, &palette, &fov, &entities, &characters, &positions)
                .with_area(Rectangle::new_sized(window.screen_size()))
                .draw(window)?;
            Ok(())
        })?;
        

        Ok(())
    }
}