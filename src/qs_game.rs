use std::rc::Rc;
use quicksilver::prelude::*;
use quicksilver::graphics::View;
use specs::prelude::*;
use crate::ai::Ai;
use crate::scene::Scene;
use crate::data::Data;
use crate::game::ecs;
use crate::game::system::{GameActionType, GameActor, GameSystem};
use crate::qs_ui::Tileset;
use crate::data::GameText;

type SystemData<'a> = (WriteExpect<'a, Data>,);

pub struct Game<'a, 'b> {
    pub scene: Rc<Scene>,
    dispatcher: Dispatcher<'a, 'b>,
    pub world: World,
    pub tileset: Asset<Tileset>,
    pub text: Asset<GameText>,
}

impl State for Game<'static, 'static> {
    /// Load the assets and initialise the game
    fn new() -> Result<Self> {
        let mut world = World::new();

        let mut dispatcher = DispatcherBuilder::new()
            .with(GameSystem::new(), "game", &[])
            .build();

        dispatcher.setup(&mut world);

        ecs::setup(&mut world);

        let data = Data::new(&mut world);
        world.insert(data);

        world.insert(Ai::default());
        
        let scene = Scene::default();

        Ok(Self {
            dispatcher,
            world: world,
            scene: Rc::new(scene),
            text: GameText::load(),
            tileset: Tileset::load(),
        })
    }

    /// Process keyboard and mouse, update the game state
    fn update(&mut self, window: &mut Window) -> Result<()> {
        use quicksilver::input::ButtonState::*;

        {
            let (mut data,): SystemData = self.world.system_data();
            let player = data.player;

            if window.keyboard()[Key::Left] == Pressed {
                data.action(GameActor::Player(player), GameActionType::MoveAttack(-1, 0));
            }
            if window.keyboard()[Key::Right] == Pressed {
                data.action(GameActor::Player(player), GameActionType::MoveAttack(1, 0));
            }
            if window.keyboard()[Key::Up] == Pressed {
                data.action(GameActor::Player(player), GameActionType::MoveAttack(0, -1));
            }
            if window.keyboard()[Key::Down] == Pressed {
                data.action(GameActor::Player(player), GameActionType::MoveAttack(0, 1));
            }

            if window.keyboard()[Key::Escape].is_down() {
                data.stop = true;
            }

            if window.keyboard()[Key::X].is_down() {
                self.text.execute(|text| {
                    let inventory = text.font.render(
                        "Inventory:\n[A] Dagger\n[B] Buckler",
                        &FontStyle::new(20.0, Color::BLACK),
                    );
                    text.inventory = inventory?;
                    Ok(())
                })?;
            }

            if data.stop {
                window.close();
            }
        }

        self.dispatcher.dispatch(&self.world);

        self.world.maintain();

        Ok(())
    }

    /// Draw stuff on the screen
    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.set_view(View::new(Rectangle::new(Vector::ZERO, window.screen_size())));

        let scene = self.scene.clone();

        scene.draw(window, self)?;

        Ok(())
    }
}
