use quicksilver::Result;
use quicksilver::lifecycle::Window;
use quicksilver::geom::{Rectangle, Shape, Vector};
use quicksilver::graphics::{Color, Background, FontStyle};
use specs::world::WorldExt;
use crate::qs_game::{Pos, Render};
use crate::qs_ui::{Widget, Label, LayoutRect};
use crate::data::{Data};
use tui::layout::{Constraint, Direction, Layout};

pub struct DemoScene;

impl DemoScene {
    pub fn draw(&self, window: &mut Window, game: &mut Data) -> Result<()> {
        use specs::Join;

        window.clear(Color::WHITE)?;
        
        //let screen_area = Rectangle::new_sized(window.screen_size());
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(50)
            .constraints([
                Constraint::Length(50),
                Constraint::Percentage(100),
                Constraint::Min(60)
            ].as_ref())
            .split(LayoutRect::from(Rectangle::new_sized(window.screen_size())).into());

        // Draw the game title
        game.text.execute(|text| {
            /*window.draw(
                &text
                    .title
                    .area()
                    .with_center((window.screen_size().x as i32 / 2, 40)),
                Background::Img(&text.title),
            );*/
            let mut label = Label::new(&text.title, &text.font, FontStyle::new(72.0, Color::BLACK))?;

            let chunks_row = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(label.area().width() as u16),
                    Constraint::Balance(0, 1, 1),
                ].as_ref())
                .split(chunks[0]);

            label
                .with_area(LayoutRect::from(chunks_row[1]).into())
                //.with_center((window.screen_size().x as i32 / 2, 40).into())
                .draw(window)?;
            Ok(())
        })?;

        // Draw the mononoki font credits
        game.text.execute(|text| {
            Label::new(&text.mononoki_info, &text.font, FontStyle::new(20.0, Color::BLACK))?
                .with_pos((2, window.screen_size().y as i32 - 60).into())
                .draw(window)?;
            Ok(())
        })?;

        // Draw the Square font credits
        game.text.execute(|text| {
            window.draw(
                &text
                    .square_info
                    .area()
                    .translate((2, window.screen_size().y as i32 - 30)),
                Background::Img(&text.square_info),
            );
            Ok(())
        })?;

        let offset_px = Vector::new(50, 120);

        // Draw the map
        let (tileset, map) = (&mut game.tileset, &game.map);
        tileset.execute(|tileset| {
            for tile in map.iter() {
                let pos_px = offset_px + tile.pos.times(tileset.tile_size());
                tileset.draw(window, tile.glyph, pos_px, tile.color)?;

                /*if let Some(image) = tileset.get(&tile.glyph) {
                    
                    window.draw(
                        &Rectangle::new(offset_px + pos_px, image.area().size()),
                        Background::Blended(&image, tile.color),
                    );
                }*/
            }
            Ok(())
        })?;

        let pos_storage = game.world.read_storage::<Pos>();
        let render_storage = game.world.read_storage::<Render>();
        game.tileset.execute(|tileset| {
            for (pos, render) in (&pos_storage, &render_storage).join() {
                let pos_px = offset_px + pos.0.times(tileset.tile_size());
                tileset.draw(window, render.glyph, pos_px, render.color)?;

                /*if let Some(image) = tileset.get(&render.glyph) {
                    
                    window.draw(
                        &Rectangle::new(pos_px, image.area().size()),
                        Background::Blended(&image, render.color),
                    );
                }*/
            }

            Ok(())
        })?;

        Ok(())
    }
}