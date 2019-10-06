use crate::ecs::{Attributes, Liquid, Position};
use rgoap::{self, Action, State};
use specs::{Entities, Entity, ReadStorage, WriteStorage};
use std::f32;

pub trait StateBuilder<P> {
    fn with<S>(self, name: S, value: bool) -> Self
    where
        S: Into<P>;
}

impl<P> StateBuilder<P> for State<P>
where
    P: Ord,
{
    fn with<S>(mut self, name: S, value: bool) -> Self
    where
        S: Into<P>,
    {
        self.insert(name.into(), value);
        self
    }
}

pub trait ActionBuilder<K, P> {
    fn build<S>(name: S, cost: usize) -> Self
    where
        S: Into<K>;
    fn pre<S>(self, name: S, value: bool) -> Self
    where
        S: Into<P>;
    fn post<S>(self, name: S, value: bool) -> Self
    where
        S: Into<P>;
}

impl<K, P> ActionBuilder<K, P> for Action<K, P>
where
    P: Ord,
{
    fn build<S>(name: S, cost: usize) -> Self
    where
        S: Into<K>,
    {
        Action::new(name, cost)
    }

    fn pre<S>(mut self, name: S, value: bool) -> Self
    where
        S: Into<P>,
    {
        self.pre_conditions.insert(name.into(), value);
        self
    }

    fn post<S>(mut self, name: S, value: bool) -> Self
    where
        S: Into<P>,
    {
        self.post_conditions.insert(name.into(), value);
        self
    }
}

#[derive(Clone, Debug)]
pub struct Agent {
    pub turn_ended: bool,
    pub attributes: Attributes,
}

impl Agent {
    pub fn new(attributes: Attributes) -> Self {
        Agent {
            turn_ended: false,
            attributes: attributes,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AiAction {
    pub cost: u32,
    pub utility: f32,
    pub action_type: AiActionType,
    pub name: String,
    pub pre_conditions: State<AiPredicate>,
    pub post_conditions: State<AiPredicate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AiActionType {
    Meditate,
    DrinkPotable(Entity),
    Get(Entity),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AiPredicate {
    Have(Entity),
}

impl AiAction {
    pub fn action(self) -> Action<AiActionType, AiPredicate> {
        let mut action = Action::new(self.action_type, self.cost as usize);
        action.pre_conditions = self.pre_conditions;
        action.post_conditions = self.post_conditions;
        action
    }

    pub fn meditate(agent: &Agent) -> AiAction {
        AiAction {
            name: format!("meditate"),
            cost: 1,
            utility: 1.0 - agent.attributes.calmness,
            action_type: AiActionType::Meditate,
            pre_conditions: State::new(),
            post_conditions: State::new(),
        }
    }

    pub fn get(_agent: &Agent, e: Entity) -> AiAction {
        AiAction {
            name: format!("get({:?})", e),
            cost: 1,
            utility: 0.0,
            action_type: AiActionType::Get(e),
            pre_conditions: State::new().with(AiPredicate::Have(e), false),
            post_conditions: State::new().with(AiPredicate::Have(e), true),
        }
    }

    pub fn drink(agent: &Agent, e: Entity) -> AiAction {
        AiAction {
            name: format!("drink({:?})", e),
            cost: 1,
            utility: 1.0 - agent.attributes.thirst,
            action_type: AiActionType::DrinkPotable(e),
            pre_conditions: State::new().with(AiPredicate::Have(e), true),
            post_conditions: State::new().with(AiPredicate::Have(e), false),
        }
    }
}

pub struct AiActions {
    pub agent: Agent,
    pub actions: Vec<AiAction>,
}

impl AiActions {
    pub fn new(agent: Agent) -> Self {
        AiActions {
            agent: agent,
            actions: vec![],
        }
    }

    pub fn len(&self) -> usize {
        self.actions.len()
    }

    pub fn add_action(&mut self, action: AiAction) {
        self.actions.push(action);
    }

    pub fn setup_actions<'a>(
        &mut self,
        entities: &Entities<'a>,
        positions: &mut WriteStorage<'a, Position>,
        attributes: &ReadStorage<'a, Attributes>,
        liquids: &ReadStorage<'a, Liquid>,
    ) {
        use specs::Join;

        self.add_action(AiAction::meditate(&self.agent));

        for (e, _pos, _attr, liquid) in (entities, positions, attributes, liquids).join() {
            if liquid.potable {
                self.add_action(AiAction::drink(&self.agent, e));
                self.add_action(AiAction::get(&self.agent, e));
            }
        }
    }

    pub fn find_max_utility<'a>(&'a self) -> Option<&'a AiAction> {
        let mut max_i = None;
        let mut max_u = f32::NEG_INFINITY;

        for (i, a) in self.actions.iter().enumerate() {
            let u = a.utility;

            if u > max_u {
                max_i = Some(i);
                max_u = u;
            }
        }

        max_i.and_then(|i| self.actions.get(i))
    }

    pub fn plan(&self) -> Option<Vec<AiActionType>> {
        let possible_actions: Vec<Action<AiActionType, AiPredicate>> =
            self.actions.iter().map(|a| a.clone().action()).collect();

        if let Some(ai_action) = self.find_max_utility() {
            let initial_state = State::new();
            let action = ai_action.clone().action();

            let planned = rgoap::plan(&initial_state, &action.post_conditions, &possible_actions);
            planned.map(|actions| actions.iter().map(|action| action.name.clone()).collect())
        } else {
            None
        }
    }
}
