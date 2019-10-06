use doryen_fov::{FovAlgorithm, FovRecursiveShadowCasting, MapData};
use super::level::Level;
use super::ecs::Position;

pub struct Fov {
    map: MapData,
}

impl Fov {
    pub fn new(level: &Level) -> Self {
        let r = level.area();

        let mut fov = Fov {
            map: MapData::new(r.width as usize, r.height as usize),
        };

        fov.load_level(level);

        fov
    }

    pub fn compute(&mut self, level: &mut Level, position: &Position, radius: u16) {
        self.map.clear_fov();
        let mut fov = FovRecursiveShadowCasting::new();
        fov.compute_fov(&mut self.map, position.x as usize, position.y as usize, radius as usize, true);
        self.save_mapped(level);
    }

    fn save_mapped(&self, level: &mut Level) {
        let r = level.area().clone();

        for i in r.left()..r.right() {
            for j in r.top()..r.bottom() {
                if self.is_in_fov(&Position::new(i as i32, j as i32)) {
                    level.get_mut(i, j).explored = true;
                }
            }
        }
    }

    pub fn is_in_fov(&self, position: &Position) -> bool {
        self.map.is_in_fov(position.x as usize, position.y as usize)
    }

    fn load_level(&mut self, level: &Level) {
        let r = level.area();

        let mut x = 0;
        for i in r.left()..r.right() {

            let mut y = 0;
            for j in r.top()..r.bottom() {
                let cell = level.get(i, j);
                let is_transparent = !cell.block_sight;

                self.map.set_transparent(x, y, is_transparent);

                y += 1;
            }

            x += 1;
        }
    }
}