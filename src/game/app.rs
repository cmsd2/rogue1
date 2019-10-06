use piston::input::Key;
use piston::input::keyboard::ModifierKey;
use tui::{Frame};
use tui::layout::Rect;
use tui::backend::Backend;
use crate::commands::{Command, Commands};
use crate::ui::scene::Scene;
use crate::input::{InputEventType, InputEventKey};
use crate::chords::ChordResult;
use crate::glfw_system::RenderContext;
use super::events::Time;
use super::system::{GameActor, GameEventQueue, GameActionQueue, GameAction, GameActionType, GameEvent};
use super::ecs::Position;

pub enum InputMode {
    Edit,
    Play,
    Look,
}

pub struct App {
    pub title: String,
    pub commands: Commands,
    pub stop: bool,
    pub player_turns: u32,
    pub turn: Option<GameActor>,
    pub scene: Scene,
    event_queue: GameEventQueue,
    pub action_queue: GameActionQueue,
    pub input_mode: InputMode,
    pub time: Time,
    pub cursor: Option<Position>,
}

impl Default for App {
    fn default() -> Self {
        App::new()
    }
}

impl App {
    pub fn new() -> Self {
        let title = format!("rogue1");
        App {
            title: title.clone(),
            commands: Commands::default(),
            stop: false,
            player_turns: 0,
            turn: None,
            scene: Scene::Text { title, cursor: None, path: None },
            event_queue: GameEventQueue::default(),
            action_queue: GameActionQueue::default(),
            input_mode: InputMode::Play,
            time: Time::default(),
            cursor: None,
        }
    }

    pub fn end_turn(&mut self, actor: GameActor) {
        self.turn = None;
        self.action_queue.clear();
        println!("[{:?}] end turn: {} for {:?}", self.time, self.player_turns, actor);
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
            },
            _ => {}
        }

        self.turn = Some(actor);

        println!("[{:?}] new turn: {} for: {:?}", self.time, self.player_turns, actor);
    }

    pub fn schedule_turn(&mut self, delay: Time, actor: GameActor) {
        println!("[{:?}] schedule actor turn in: {} for: {:?}", self.time, delay, actor);
        self.event_after(delay, GameEvent::Turn(actor));
    }

    pub fn finish(&mut self) {
        println!("[{:?}] stop", self.time);
        self.stop = true;
    }

    pub fn is_finished(&self) -> bool {
        self.stop
    }

    pub fn key_event(&mut self, state: InputEventType, key: InputEventKey) {
        match self.input_mode {
            InputMode::Play => {
                match self.commands.key_event(state, key) {
                    Some(ChordResult::Action(command)) => {
                        self.play_command(command);
                    },
                    None => {
                        self.play_key_event(state, key);
                    },
                    _ => {}
                }
            },
            InputMode::Look => {
                match self.commands.key_event(state, key) {
                    Some(ChordResult::Action(command)) => {
                        self.look_command(command);
                    },
                    None => {
                        self.look_key_event(state, key);
                    },
                    _ => {}
                }
            },
            InputMode::Edit => {
                // send key presses to text box
                
            }
        }
    }

    pub fn look_mode(&mut self, cursor: Position, path: Option<Vec<Position>>) {
        println!("[{:?}] look at: {:?}", self.time, cursor);
        self.input_mode = InputMode::Look;
        self.cursor = Some(cursor.clone());
        self.scene = Scene::Text {
            title: self.title.clone(),
            cursor: Some(cursor),
            path: path
        };
    }

    pub fn look_key_event(&mut self, state: InputEventType, key: InputEventKey) {
        match (state, key) {
            (InputEventType::KeyDown, InputEventKey::KeyboardKey { modifiers: ModifierKey::NO_MODIFIER, key: Key::X, .. }) => {
                self.play_mode();
            },
            _ => {}
        }
    }

    pub fn look_command(&mut self, command: Command) {
        println!("[{:?}] look command: {:?} player_turn: {:?}", self.time, command, self.turn);
        match (command, self.turn) {
            (Command::Quit, _) => {
                self.finish();
            },
            (Command::Up, Some(actor @ GameActor::Player(_))) => {
                self.action(actor, GameActionType::Look(0, -1));
            },
            (Command::Down, Some(actor @ GameActor::Player(_))) => {
                self.action(actor, GameActionType::Look(0, 1));
            },
            (Command::Left, Some(actor @ GameActor::Player(_))) => {
                self.action(actor, GameActionType::Look(-1, 0));
            },
            (Command::Right, Some(actor @ GameActor::Player(_))) => {
                self.action(actor, GameActionType::Look(1, 0));
            },
            _ => {}
        }
    }

    pub fn play_mode(&mut self) {
        self.input_mode = InputMode::Play;
        self.cursor = None;
        self.scene = Scene::Text {
            title: self.title.clone(),
            cursor: None,
            path: None
        };
    }

    pub fn play_key_event(&mut self, state: InputEventType, key: InputEventKey) {
        match (state, key, self.turn) {
            (InputEventType::KeyDown,
             InputEventKey::KeyboardKey { modifiers: ModifierKey::NO_MODIFIER, key: Key::X, .. },
             Some(actor @ GameActor::Player(_))) => {
                self.action(actor, GameActionType::Look(0, 0));
            },
            _ => {}
        }
    }

    pub fn play_command(&mut self, command: Command) {
        println!("[{:?}] play command: {:?} player_turn: {:?}", self.time, command, self.turn);
        match (command, self.turn) {
            (Command::Quit, _) => {
                self.finish();
            },
            (Command::Up, Some(actor @ GameActor::Player(_))) => {
                self.action(actor, GameActionType::MoveAttack(0, -1));
            },
            (Command::Down, Some(actor @ GameActor::Player(_))) => {
                self.action(actor, GameActionType::MoveAttack(0, 1));
            },
            (Command::Left, Some(actor @ GameActor::Player(_))) => {
                self.action(actor, GameActionType::MoveAttack(-1, 0));
            },
            (Command::Right, Some(actor @ GameActor::Player(_))) => {
                self.action(actor, GameActionType::MoveAttack(1, 0));
            },
            _ => {}
        }
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
        println!("[{:?}] schedule at {}: {:?}", self.time, at, event);
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

    pub fn render<'a, 'b, B>(&mut self, f: &mut Frame<B>, size: Rect, render_context: RenderContext<'a,'b>) where B: Backend {
        self.scene.render(f, size, render_context);
    }
}
