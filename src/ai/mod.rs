use specs::{Entities, Entity, ReadStorage};
use crate::game::app::App;
use crate::game::system::{GameActor, GameActionType};
use crate::game::ecs::{Attributes, Position, Liquid};

pub mod system;
pub mod state;
pub mod actions;

use actions::{AiActions, AiActionType, Agent};

pub struct Ai {
}

impl Default for Ai {
    fn default() -> Self {
        Ai {}
    }
}

impl Ai {
    pub fn schedule_ai_actions<'a>(&mut self, app: &mut App, entity: Entity, entities: &Entities<'a>, positions: &ReadStorage<'a, Position>, attributes: &ReadStorage<'a, Attributes>, liquids: &ReadStorage<'a, Liquid>) {
        // unimplemented
        let entity_attrs = attributes.get(entity).unwrap();
        let agent = Agent::new(entity_attrs.clone());
        let mut ai_actions = AiActions::new(agent);
        ai_actions.setup_actions(entities, positions, attributes, liquids);

        if let Some(actions) = ai_actions.plan() {
            if let Some(action) = actions.get(0) {
                match action {
                    AiActionType::Meditate => {
                        app.action(GameActor::NonPlayer(entity), GameActionType::Pass);
                    }
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