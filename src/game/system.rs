use super::app::App;
use super::ecs::*;
use super::events::*;
use super::level::{CellType, Level};
use super::path::PathFinder;
use crate::ai::Ai;
use slog::Logger;
use specs::{Entities, Entity, ReadStorage, System, Write, WriteExpect, WriteStorage};
use std::sync::Arc;

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
    log: Arc<Logger>,
}

impl GameSystem {
    pub fn new(log: Arc<Logger>) -> Self {
        GameSystem { log: log }
    }
}

impl<'a> System<'a> for GameSystem {
    type SystemData = (
        WriteExpect<'a, App>,
        Write<'a, Ai>,
        Entities<'a>,
        Write<'a, Level>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Character>,
        ReadStorage<'a, Attributes>,
        ReadStorage<'a, Liquid>,
    );

    fn run(
        &mut self,
        (mut app, mut ai, entities, mut level, mut positions, characters, attributes, liquids): Self::SystemData,
    ) {
        //use specs::Join;

        /*
         * loop:
         *   if not waiting for a human to take their turn:
         *     advance to next game time step
         *     evaluate game event
         *     while ai turn:
         *       run ai
         *       process actions
         *   else:
         *     break
         */

        loop {
            /*
             * run ai
             */
            match app.actor_turn() {
                Some(GameActor::NonPlayer(entity)) => {
                    debug!(self.log, "[{:?}] ai turn: {:?}", app.time, entity);
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
                let turn_status = match action {
                    GameActionType::Pass => TurnStatus::EndTurn(Time::default() + 1),
                    GameActionType::Stop => TurnStatus::Stop,
                    GameActionType::MoveAttack(x, y) => self.move_or_attack(
                        actor,
                        x,
                        y,
                        &mut app,
                        &entities,
                        &mut level,
                        &mut positions,
                        &characters,
                    ),
                    GameActionType::Look(x, y) => {
                        let path_finder = PathFinder::new(&level);
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
                    info!(self.log, "[{:?}] {:?}", time, game_event);

                    match game_event {
                        GameEvent::Turn(actor) => {
                            app.new_turn(actor);
                        }
                    }
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
        app: &mut WriteExpect<'a, App>,
        _entities: &Entities<'a>,
        level: &mut Write<'a, Level>,
        positions: &mut WriteStorage<'a, Position>,
        _characters: &ReadStorage<'a, Character>,
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

        match Collider::new(level).get(&new_pos) {
            Occupier::Empty => {
                EntityMover::new(level).move_entity(entity, &mut pos, x, y);
                TurnStatus::EndTurn(Time::default() + 1)
            }
            Occupier::Wall => {
                debug!(
                    self.log,
                    "[{:?}] path blocked by wall at {:?}", app.time, new_pos
                );
                TurnStatus::Continue
            }
            Occupier::Entity(target_entity) => {
                debug!(
                    self.log,
                    "[{:?}] entity {:?} interact with {:?}", app.time, entity, target_entity
                );
                TurnStatus::EndTurn(Time::default() + 1)
            }
        }
    }
}

pub struct EntityMover<'a, 'b> {
    level_map: &'a mut Write<'b, Level>,
}

impl<'a, 'b> EntityMover<'a, 'b> {
    pub fn new(level_map: &'a mut Write<'b, Level>) -> Self {
        EntityMover {
            level_map: level_map,
        }
    }

    pub fn move_entity(&mut self, entity: Entity, pos: &mut Position, x: i32, y: i32) {
        self.level_map.move_entity(
            entity,
            pos.x as u16,
            pos.y as u16,
            (pos.x + x) as u16,
            (pos.y + y) as u16,
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
