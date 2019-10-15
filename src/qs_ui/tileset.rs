use std::collections::HashMap;
use quicksilver::lifecycle::Window;
use quicksilver::geom::{Rectangle, Vector};
use quicksilver::graphics::{Background, Color, Image, Font, FontStyle};
use quicksilver::Result;
use quicksilver::lifecycle::Asset;
use quicksilver::Future;

pub struct Tileset {
    tile_size: Vector,
    tiles_map: HashMap<char, Image>,
}

impl Tileset {
    pub fn new<S>(size: Vector, font: &Font, glyphs: S) -> Result<Self> where S: AsRef<str> {
        let tiles = font
                .render(glyphs.as_ref(), &FontStyle::new(size.y, Color::WHITE))?;
        
        let mut tiles_map = HashMap::new();
        for (index, glyph) in glyphs.as_ref().chars().enumerate() {
            let pos = (index as i32 * size.x as i32, 0);
            let tile = tiles.subimage(Rectangle::new(pos, size));
            tiles_map.insert(glyph, tile);
        }

        Ok(Tileset {
            tile_size: size,
            tiles_map: tiles_map,
        })
    }

    pub fn tile_size(&self) -> Vector {
        self.tile_size
    }

    pub fn draw(&self, window: &mut Window, c: char, pos: Vector, color: Color) -> Result<()> {
        if let Some(image) = self.tiles_map.get(&c) {
            window.draw(
                &Rectangle::new(pos, image.area().size()),
                Background::Blended(&image, color),
            );
        }
        Ok(())
    }

    pub fn load() -> Asset<Tileset> {
        // The Square font: http://strlen.com/square/?s[]=font
        // License: CC BY 3.0 https://creativecommons.org/licenses/by/3.0/deed.en_US
        let font_square = "square.ttf";
        let game_glyphs = "#@oTg.%";
        let tile_size_px = Vector::new(24, 24);
        Asset::new(Font::load(font_square).and_then(move |text| {
            Tileset::new(tile_size_px, &text, &game_glyphs)
        }))
    }
}
