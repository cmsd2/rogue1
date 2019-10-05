extern crate utility_programming as up;

use specs::{Entities, System, ReadStorage, Write};
use up::{/*Generator, Modifier, ModifyOptimizer,*/ Utility};
use crate::ecs::{Attributes, Position, Liquid};
use crate::app::App;
use crate::ai::Ai;
use crate::game::GameActor;

pub struct AiSystem;

impl<'a> System<'a> for AiSystem {
	type SystemData = (
        Write<'a, Ai>,
        Write<'a, App>,
        Entities<'a>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Attributes>,
        ReadStorage<'a, Liquid>,
    );

    fn run(&mut self, (mut ai, mut app, entities, positions, attributes, liquids): Self::SystemData) {
        //use specs::Join;

        /*
         * decide on actions
         */
        match app.actor_turn() {
            Some(GameActor::NonPlayer(entity)) => {
                ai.schedule_ai_actions(&mut app, entity, &entities, &positions, &attributes, &liquids);
            },
            _ => {}
        }
    }
}

pub enum PersonalityUtility {
    BeCalm { threshold: f32, penalty: f32, reward: f32 },
}

pub trait Named {
    fn name(&self) -> &'static str;
}

impl Named for PersonalityUtility {
    fn name(&self) -> &'static str {
        match self {
            PersonalityUtility::BeCalm { .. } => "be_calm",
        }
    }
}

impl Utility<Attributes> for PersonalityUtility {
    fn utility(&self, obj: &Attributes) -> f64 {
        match *self {
            PersonalityUtility::BeCalm { threshold, penalty, reward } => {
                (if obj.calmness > threshold {
                    (obj.calmness - threshold) * reward
                } else {
                    (threshold - obj.calmness) * penalty
                }).into()
            }
        }
    }
}

