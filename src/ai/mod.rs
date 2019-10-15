use crate::data::Data;
use crate::game::ecs::{Attributes, Liquid, Position};
use crate::game::system::{GameActionType, GameActor};
use crate::game::path::PathFinder;
use specs::{Entities, Entity, ReadStorage, WriteStorage};

pub mod actions;
pub mod state;

use actions::{Agent, AiActionType, AiActions};

pub struct Ai {}

impl Default for Ai {
    fn default() -> Self {
        Ai {}
    }
}

impl Ai {
    pub fn schedule_ai_actions<'a>(
        &mut self,
        app: &mut Data,
        entity: Entity,
        entities: &Entities<'a>,
        positions: &mut WriteStorage<'a, Position>,
        attributes: &ReadStorage<'a, Attributes>,
        liquids: &ReadStorage<'a, Liquid>,
    ) {
        // unimplemented
        let entity_attrs = attributes.get(entity).unwrap();
        let entity_position = positions.get(entity).map(|e| e.to_owned()).unwrap();
        let agent = Agent::new(entity_position.clone(), entity_attrs.clone());
        let mut ai_actions = AiActions::new(agent);
        ai_actions.setup_actions(app, entities, positions, attributes, liquids);

        if let Some(actions) = ai_actions.plan() {
            if let Some(action) = actions.get(0) {
                match action {
                    AiActionType::Meditate => {
                        app.action(GameActor::NonPlayer(entity), GameActionType::Pass);
                    },
                    AiActionType::Attack(target) => {
                        if let Some(target_position) = positions.get(*target) {
                            let pathfinder = PathFinder::new(&app.level);
                            if let Some((path, cost)) = pathfinder.path(&entity_position, target_position) {
                                debug!("[{:?}] ai entity {:?} targetting {:?} on path ({:?},{})", app.time, entity, target, path, cost);
                                assert!(path.get(0).unwrap() == &entity_position);
                                let next_tile = path.get(1).unwrap();
                                let x = next_tile.x - entity_position.x;
                                let y = next_tile.y - entity_position.y;
                                app.action(GameActor::NonPlayer(entity), GameActionType::MoveAttack(x, y));
                            }
                        }
                    },
                    _ => {
                        // not implemented
                    }
                }
            }
        }
        // only necessary if we didn't issue an action that ends the turn for some reason,
        // (either nothing to do or a bug)
        app.action(GameActor::NonPlayer(entity), GameActionType::Pass);
    }
}
