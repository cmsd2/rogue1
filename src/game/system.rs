use crate::data::Data;
use super::ecs::*;
use super::events::*;
use super::fov::Fov;
use super::level::{TileType, Level, EntityGrid};
use super::path::PathFinder;
use crate::ai::Ai;
use specs::{Entities, Entity, ReadStorage, System, Write, WriteExpect, WriteStorage};

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
    Turn(GameActor),
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameActionType {
    Pass,
    Stop,
    MoveAttack(i32, i32),
    Look(i32, i32),
    Play,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum GameActor {
    Player(Entity),
    NonPlayer(Entity),
}

impl GameActor {
    pub fn entity(&self) -> Entity {
        match *self {
            GameActor::Player(entity) => entity,
            GameActor::NonPlayer(entity) => entity,
        }
    }

    pub fn is_player(&self) -> bool {
        !self.is_ai()
    }

    pub fn is_ai(&self) -> bool {
        match self {
            GameActor::Player(_) => false,
            GameActor::NonPlayer(_) => true,
        }
    }
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

impl<'a> System<'a> for GameSystem {
    type SystemData = (
        WriteExpect<'a, Data>,
        Write<'a, Ai>,
        Entities<'a>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Character>,
        ReadStorage<'a, Attributes>,
        ReadStorage<'a, Liquid>,
    );

    fn run(
        &mut self,
        (mut app, mut ai, entities, mut positions, characters, attributes, liquids): Self::SystemData,
    ) {
        //use specs::Join;

        /*
         * loop:
         *   if ai turn:
         *     run ai
         *   for each action:
         *     handle action
         *     end turn if needed
         *     schedule wakeup if needed
         *   if not waiting for a human to take their turn:
         *     advance to next game time step
         *     evaluate game event
         *   else:
         *     break from loop
         */

        loop {
            /*
             * run ai
             */
            match app.actor_turn() {
                Some(GameActor::NonPlayer(entity)) => {
                    debug!("[{:?}] ai turn: {:?}", app.time, entity);
                    ai.schedule_ai_actions(
                        &mut app,
                        entity,
                        &entities,
                        &mut positions,
                        &attributes,
                        &liquids,
                    );
                }
                _ => {}
            }

            /*
             * execute actions
             */
            while let Some(GameAction { actor, action, .. }) = app.next_action() {
                info!("[{:?}] action {:?} by {:?}", app.time, action, actor);
                let turn_status = match action {
                    GameActionType::Pass => TurnStatus::EndTurn(Time::default() + 1),
                    GameActionType::Stop => TurnStatus::Stop,
                    GameActionType::MoveAttack(x, y) => self.move_or_attack(
                        actor,
                        x,
                        y,
                        &mut app,
                        &entities,
                        &mut positions,
                        &characters,
                        &attributes,
                    ),
                    GameActionType::Look(x, y) => {
                        let path_finder = PathFinder::new(&app.level);
                        let actor_pos = positions.get(actor.entity()).unwrap().clone();
                        let cursor_pos = if let Some(cursor) = app.cursor.clone() {
                            cursor.delta(x, y)
                        } else {
                            actor_pos.clone()
                        };
                        let cursor_path = path_finder
                            .path(&actor_pos, &cursor_pos)
                            .map(|(path, _)| path);
                        app.look_mode(cursor_pos, cursor_path);
                        TurnStatus::Continue
                    },
                    GameActionType::Play => {
                        app.play_mode();
                        TurnStatus::Continue
                    }
                };

                match turn_status {
                    TurnStatus::EndTurn(delay) => {
                        app.end_turn(actor);
                        app.schedule_turn(delay, actor);
                    }
                    TurnStatus::Stop => {
                        app.end_turn(actor);
                    }
                    TurnStatus::Continue => {}
                }
            }

            if app.actor_turn().is_none() {
                /*
                 * advance game timeline
                 */
                if let Some((time, game_event)) = app.next_event() {
                    info!("[{:?}] {:?}", time, game_event);

                    match game_event {
                        GameEvent::Turn(actor) => {
                            app.new_turn(actor);
                        }
                    }
                } else {
                    warn!("[{:?}] game event queue empty and out of turns. stopping", app.time);
                    app.stop = true;
                    break;
                }
            } else {
                break;
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
    pub fn move_or_attack<'a>(
        &mut self,
        actor: GameActor,
        x: i32,
        y: i32,
        app: &mut WriteExpect<'a, Data>,
        _entities: &Entities<'a>,
        positions: &mut WriteStorage<'a, Position>,
        _characters: &ReadStorage<'a, Character>,
        attributes: &ReadStorage<'a, Attributes>,
    ) -> TurnStatus {
        //use specs::Join;

        let entity = match actor {
            GameActor::Player(entity) => entity,
            GameActor::NonPlayer(entity) => entity,
        };

        let mut pos = positions.get_mut(entity).unwrap();
        let new_pos = Position {
            x: pos.x + x,
            y: pos.y + y,
        };
        let attrs = attributes.get(entity).unwrap();

        match Collider::new(&app.level).get(&new_pos) {
            Occupier::Empty => {
                EntityMover::new(&mut app.level).move_entity(entity, &mut pos, x, y);
                if actor.is_player() {
                    app.fov.compute(&new_pos, attrs.vision_radius);
                }
                TurnStatus::EndTurn(Time::default() + 1)
            }
            Occupier::Wall => {
                debug!(
                    "[{:?}] path blocked by wall at {:?}", app.time, new_pos
                );
                TurnStatus::Continue
            }
            Occupier::Entity(target_entity) => {
                debug!(
                    "[{:?}] entity {:?} interact with {:?}", app.time, entity, target_entity
                );
                TurnStatus::EndTurn(Time::default() + 1)
            }
        }
    }
}

pub struct EntityMover<'a> {
    level_map: &'a mut Level,
}

impl<'a> EntityMover<'a> {
    pub fn new(level_map: &'a mut Level) -> Self {
        EntityMover {
            level_map: level_map,
        }
    }

    pub fn move_entity(&mut self, entity: Entity, pos: &mut Position, x: i32, y: i32) {
        self.level_map.move_entity(
            entity,
            pos.x,
            pos.y,
            pos.x + x,
            pos.y + y,
        );
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

impl<'a> Collider<'a> {
    pub fn new(level_map: &'a Level) -> Self {
        Collider {
            level_map: level_map,
        }
    }

    pub fn get(&self, p: &Position) -> Occupier {
        let level_cell = self.level_map.get(p.x, p.y);
        if let Some(entity) = level_cell.entities.iter().find(|e| e.blocked) {
            Occupier::Entity(entity.id)
        } else {
            match level_cell.cell_type {
                TileType::Wall => Occupier::Wall,
                _ => Occupier::Empty,
            }
        }
    }
}
