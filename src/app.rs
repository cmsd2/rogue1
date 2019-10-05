use crate::commands::{Command, Commands};
use crate::scene::Scene;
use crate::events::Time;
use crate::game::{GameActor, GameEventQueue, GameActionQueue, GameAction, GameActionType, GameEvent};
use crate::input::{InputEventType, InputEventKey};
use crate::chords::ChordResult;

pub enum InputMode {
    Edit,
    View,
}

pub struct App {
    pub commands: Commands,
    pub stop: bool,
    pub player_turns: u32,
    pub turn: Option<GameActor>,
    pub scene: Scene,
    event_queue: GameEventQueue,
    pub action_queue: GameActionQueue,
    pub input_mode: InputMode,
    pub time: Time,
}

impl Default for App {
    fn default() -> Self {
        App::new()
    }
}

impl App {
    pub fn new() -> Self {
        App {
            commands: Commands::default(),
            stop: false,
            player_turns: 0,
            turn: None,
            scene: Scene::default(),
            event_queue: GameEventQueue::default(),
            action_queue: GameActionQueue::default(),
            input_mode: InputMode::View,
            time: Time::default(),
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
            InputMode::View => {
                match self.commands.key_event(state, key) {
                    Some(ChordResult::Action(command)) => {
                        self.command(command);
                    },
                    _ => {}
                }
            },
            InputMode::Edit => {
                // send key presses to text box
                
            }
        }
    }

    pub fn command(&mut self, command: Command) {
        println!("[{:?}] command: {:?} player_turn: {:?}", self.time, command, self.turn);
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
}
