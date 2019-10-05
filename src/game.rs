use specs::{Entities, Entity, System, Write, ReadStorage, WriteStorage};
use crate::events::*;
use crate::app::App;
use crate::level::{CellType, Level};
use crate::ecs::*;

#[derive(Debug, Clone, PartialEq)]
pub enum PlayerAction {
    Move(i32, i32),
}

#[derive(Debug, Clone, PartialEq)]
pub enum NonPlayerAction {
    Move(i32, i32),
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameEvent {
    Turn(GameActor)
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameActionType {
    Pass,
    Stop,
    MoveAttack(i32, i32),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum GameActor {
    Player(Entity),
    NonPlayer(Entity),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GameAction {
    pub actor: GameActor,
    pub turn: u32,
    pub action: GameActionType,
}

pub type GameActionQueue = Vec<GameAction>;

pub type GameEventQueue = EventQueue<GameEvent>;

pub struct GameSystem {
}

impl GameSystem {
    pub fn new() -> Self {
        GameSystem {}
    }
}

impl <'a> System<'a> for GameSystem {
    type SystemData = (Write<'a, App>, Entities<'a>, Write<'a, Level>, WriteStorage<'a, Position>, ReadStorage<'a, Character>);

    fn run(&mut self, (mut app, entities, mut level, mut positions, characters): Self::SystemData) {
        //use specs::Join;

        /*
         * execute actions
         */
        while let Some(GameAction { actor, action, .. }) = app.next_action() {
            let turn_status = match action {
                GameActionType::Pass => {
                    TurnStatus::EndTurn(Time::default() + 1)
                },
                GameActionType::Stop => {
                    TurnStatus::Stop
                },
                GameActionType::MoveAttack(x, y) => {
                    self.move_or_attack(actor, x, y, &mut app, &entities, &mut level, &mut positions, &characters)
                },
            };

            match turn_status {
                TurnStatus::EndTurn(delay) => {
                    app.end_turn(actor);
                    app.schedule_turn(delay, actor);
                },
                TurnStatus::Stop => {
                    app.end_turn(actor);
                },
                TurnStatus::Continue => {
                }
            }
        }

        /*
         * advance game timeline
         */
        if app.actor_turn().is_none() {
            while let Some((time, game_event)) = app.next_event() {
                println!("[{:?}] {:?}", time, game_event);

                match game_event {
                    GameEvent::Turn(actor) => {
                        app.new_turn(actor);
                        break;
                    },
                }
            }
        }
    }
}

pub enum TurnStatus {
    EndTurn(Time),
    Continue,
    Stop,
}

impl GameSystem {
    pub fn move_or_attack<'a>(&mut self, actor: GameActor, x: i32, y: i32, app: &mut Write<'a, App>, _entities: &Entities<'a>, level: &mut Write<'a, Level>, positions: &mut WriteStorage<'a, Position>, _characters: &ReadStorage<'a, Character>) -> TurnStatus {
        //use specs::Join;

        let entity = match actor {
            GameActor::Player(entity) => entity,
            GameActor::NonPlayer(entity) => entity,
        };

        let mut pos = positions.get_mut(entity).unwrap();
        let new_pos = Position { x: pos.x + x, y: pos.y + y };

        match Collider::new(level).get(&new_pos) {
            Occupier::Empty => {
                EntityMover::new(level).move_entity(entity, &mut pos, x, y);
                TurnStatus::EndTurn(Time::default() + 1)
            },
            Occupier::Wall => {
                println!("[{:?}] path blocked by wall at {:?}", app.time, new_pos);
                TurnStatus::Continue
            },
            Occupier::Entity(target_entity) => {
                println!("[{:?}] entity {:?} interact with {:?}", app.time, entity, target_entity);
                TurnStatus::EndTurn(Time::default() + 1)
            }
        }
    }
}

pub struct EntityMover<'a,'b> {
    level_map: &'a mut Write<'b, Level>,
}

impl <'a,'b> EntityMover<'a,'b> {
    pub fn new(level_map: &'a mut Write<'b, Level>) -> Self {
        EntityMover {
            level_map: level_map,
        }
    }

    pub fn move_entity(&mut self, entity: Entity, pos: &mut Position, x: i32, y: i32) {
        self.level_map.move_entity(entity, pos.x as u16, pos.y as u16, (pos.x + x) as u16, (pos.y + y) as u16);
        pos.x += x;
        pos.y += y;
    }
}

pub enum Occupier {
    Empty,
    Wall,
    Entity(Entity),
}

pub struct Collider<'a> {
    level_map: &'a Level,
}

impl <'a> Collider<'a> {
    pub fn new(level_map: &'a Level) -> Self {
        Collider {
            level_map: level_map,
        }
    }

    pub fn get(&self, p: &Position) -> Occupier {
        let level_cell = self.level_map.get(p.x as u16, p.y as u16);
        
        if let Some(entity) = level_cell.entities.iter().find(|e| e.blocked) {
            Occupier::Entity(entity.id)
        } else {
            match level_cell.cell_type {
                CellType::Wall(_) => Occupier::Wall,
                _ => Occupier::Empty,
            }
        }
    }
}