use quicksilver::prelude::*;
use quicksilver::lifecycle::Asset;
use specs::prelude::*;
use crate::color::{Palette};
use crate::game::level::{Level};
use crate::game::level_gen;
use crate::game::fov::Fov;
use crate::game::system::{GameActor, GameActionQueue, GameAction, GameActionType, GameEventQueue, GameEvent};
use crate::game::events::{Time};
use crate::game::ecs::{Position, Rect};
use crate::game::factions::Factions;

pub enum InputMode {
    Edit,
    Play,
    Look,
}

pub struct GameText {
    pub font: Font,
    pub title: String,
    pub mononoki_info: String,
    pub square_info: Image,
    pub inventory: Image,
}

impl GameText {
    pub fn load() -> Asset<GameText> {
        // The Mononoki font: https://madmalik.github.io/mononoki/
        // License: SIL Open Font License 1.1
        let font_mononoki = "mononoki-Regular.ttf";

        let font_mononoki = Font::load(font_mononoki);

        Asset::new(font_mononoki.and_then(|font| {
            let title = "Quicksilver Roguelike".to_string();
            let mononoki_info = "Mononoki font by Matthias Tellen, terms: SIL Open Font License 1.1".to_string();

            let square_info = font.render(
                "Square font by Wouter Van Oortmerssen, terms: CC BY 3.0",
                &FontStyle::new(20.0, Color::BLACK),
            )?;

            let inventory = font.render(
                "Inventory:\n[A] Sword\n[B] Shield\n[C] Darts",
                &FontStyle::new(20.0, Color::BLACK),
            )?;

            Ok(GameText {
                font,
                title,
                mononoki_info,
                square_info,
                inventory,
            })
        }))
    }
}

pub struct Data {
    pub level: Level,
    pub fov: Fov,
    pub player: Entity,
    pub turn: Option<GameActor>,
    pub player_turns: u32,
    pub time: Time,
    pub event_queue: GameEventQueue,
    pub action_queue: GameActionQueue,
    pub stop: bool,
    pub cursor: Option<Position>,
    pub input_mode: InputMode,
    pub palette: Palette,
    pub factions: Factions,
}

impl Data {
    pub fn new(world: &mut World) -> Self {
        let mut level = Level::empty(Rect::new_sized(40, 30));
        let palette = Palette::new();
        let entities = level_gen::make_map(&palette, &mut level, world);
        let mut fov = Fov::new(&level);
        let player = level_gen::create_player(&palette, &mut level, &mut fov, world);
        
        let mut data = Data {
            level,
            fov,
            player,
            turn: None,
            player_turns: 0,
            time: Time::default(),
            event_queue: GameEventQueue::default(),
            action_queue: GameActionQueue::default(),
            stop: false,
            cursor: None,
            input_mode: InputMode::Play,
            palette: palette,
            factions: Factions::new(),
        };

        data.new_turn(GameActor::Player(player));

        for entity in entities {
            data.schedule_turn(Time::new(1, 0), GameActor::NonPlayer(entity));
        }

        data
    }

    pub fn end_turn(&mut self, actor: GameActor) {
        self.turn = None;
        self.action_queue.clear();
        debug!(
            "[{:?}] end turn: {} for {:?}", self.time, self.player_turns, actor
        );
    }

    pub fn actor_turn(&self) -> Option<GameActor> {
        self.turn.clone()
    }

    pub fn turns(&self) -> u32 {
        self.player_turns
    }

    pub fn new_turn(&mut self, actor: GameActor) {
        match actor {
            GameActor::Player(_) => {
                self.player_turns += 1;
            }
            _ => {}
        }

        self.turn = Some(actor);

        debug!(
            "[{:?}] new turn: {} for: {:?}", self.time, self.player_turns, actor
        );
    }

    pub fn schedule_turn(&mut self, delay: Time, actor: GameActor) {
        debug!(
            "[{:?}] schedule actor turn in: {} for: {:?}", self.time, delay, actor
        );
        self.event_after(delay, GameEvent::Turn(actor));
    }

    pub fn finish(&mut self) {
        debug!("[{:?}] stop", self.time);
        self.stop = true;
    }

    pub fn is_finished(&self) -> bool {
        self.stop
    }

    pub fn action(&mut self, actor: GameActor, action_type: GameActionType) {
        self.action_queue.push(GameAction {
            actor: actor,
            turn: self.turns(),
            action: action_type,
        });
    }

    pub fn next_action(&mut self) -> Option<GameAction> {
        if self.action_queue.is_empty() {
            None
        } else {
            Some(self.action_queue.remove(0))
        }
    }

    pub fn event(&mut self, event: GameEvent) {
        self.event_after(Time::default(), event);
    }

    pub fn event_after(&mut self, delay: Time, event: GameEvent) {
        self.event_at(self.time + delay, event);
    }

    pub fn event_at(&mut self, at: Time, event: GameEvent) {
        debug!(
            "[{:?}] schedule at {}: {:?}", self.time, at, event
        );
        self.event_queue.add(at, event);
    }

    pub fn next_event(&mut self) -> Option<(Time, GameEvent)> {
        if let Some((time, event)) = self.event_queue.next() {
            self.time = time;
            Some((time, event))
        } else {
            None
        }
    }

    pub fn look_mode(&mut self, cursor: Position, _path: Option<Vec<Position>>) {
        debug!("[{:?}] look at: {:?}", self.time, cursor);
        self.input_mode = InputMode::Look;
        self.cursor = Some(cursor.clone());
    }

    pub fn play_mode(&mut self) {
        self.input_mode = InputMode::Play;
        self.cursor = None;
    }

}
