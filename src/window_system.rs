use specs::{Component, System, VecStorage, Write, World, WriteStorage, WorldExt, Builder};
use specs::world::Entity;
use crate::window::Window;
use crate::grid::{Grid, Size};

impl Component for Window {
    type Storage = VecStorage<Self>;
}

pub struct WindowSystem {
    root_window: Option<Entity>,
}

impl WindowSystem {
    pub fn new() -> Self {
        WindowSystem {
            root_window: None,
        }
    }
}

impl <'a> System<'a> for WindowSystem {
    type SystemData = (Write<'a, Grid>, WriteStorage<'a, Window>);

    fn run(&mut self, (mut grid, mut windows): Self::SystemData) {
        //use specs::Join;

        if let Some(root_window_id) = self.root_window {
            let root_window = windows.get_mut(root_window_id);
        }
    }

    fn setup(&mut self, res: &mut World) {
        use specs::prelude::SystemData;
        Self::SystemData::setup(res);

        let root_window = res.create_entity()
            .with(Window::new(Size { w: 0, h: 0 }))
            .build();
        
        self.root_window = Some(root_window);
    }
}