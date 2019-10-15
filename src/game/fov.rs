use doryen_fov::{FovAlgorithm, FovRecursiveShadowCasting, MapData};
use super::level::Level;
use super::ecs::{Position, Rect};
use super::grid::Grid;

pub struct Fov {
    area: Rect,
    map: MapData,
    explored: Grid<bool>,
}

impl Fov {
    pub fn new(level: &Level) -> Self {
        let r = level.area();

        let mut fov = Fov {
            area: r.clone(),
            map: MapData::new(r.width as usize, r.height as usize),
            explored: Grid::default(),
        };

        fov.load_level(level);
        fov.reset_explored();

        fov
    }

    pub fn compute(&mut self, position: &Position, radius: u16) {
        self.map.clear_fov();
        let mut fov = FovRecursiveShadowCasting::new();
        fov.compute_fov(&mut self.map, position.x as usize, position.y as usize, radius as usize, true);
        self.save_mapped();
    }

    fn save_mapped(&mut self) {
        let r = &self.area;

        for i in r.left()..r.right() {
            for j in r.top()..r.bottom() {
                if self.is_in_fov(i, j) {
                    *self.explored.get_mut(i, j) = true;
                }
            }
        }
    }

    pub fn reset_explored(&mut self) {
        self.explored = Grid::filled(self.area.clone(), &false);
    }

    pub fn is_in_fov(&self, x: i32, y: i32) -> bool {
        self.map.is_in_fov(x as usize, y as usize)
    }

    pub fn is_explored(&self, x: i32, y: i32) -> bool {
        *self.explored.get(x, y)
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