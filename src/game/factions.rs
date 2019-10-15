use std::collections::HashMap as Map;

pub const PLAYER: &str = "player";
pub const MONSTER: &str = "monster";
pub const NEUTRAL: &str = "neutral";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OpinionKey {
    pub subject: String,
    pub object: String,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Opinion(f32);

impl Opinion {
    pub fn is_positive(self) -> bool {
        self.0 > 0.0
    }

    pub fn is_negative(self) -> bool {
        self.0 < 0.0
    }

    pub fn is_friendly(self) -> bool {
        self.0 > 0.5
    }

    pub fn is_hostile(self) -> bool {
        self.0 < -0.5
    }
}

pub struct Factions {
    pub player_faction: String,
    pub opinions: Map<OpinionKey, Opinion>,
}

impl Factions {
    pub fn new() -> Self {
        let mut f = Factions {
            player_faction: PLAYER.to_string(),
            opinions: Default::default()
        };
        f.set_symmetric(PLAYER, MONSTER, Opinion(-1.0));
        f
    }

    pub fn get(&self, subject: &str, object: &str) -> Opinion {
        *self.opinions.get(&OpinionKey { subject: subject.to_string(), object: object.to_string() }).unwrap_or(&Opinion::default())
    }

    pub fn set<S1,S2>(&mut self, subject: S1, object: S2, opinion: Opinion) where S1: Into<String>, S2: Into<String> {
        self.opinions.insert(OpinionKey { subject: subject.into(), object: object.into() }, opinion);
    }

    pub fn set_symmetric<S1,S2>(&mut self, subject: S1, object: S2, opinion: Opinion) where S1: Into<String>, S2: Into<String> {
        let subject = subject.into();
        let object = object.into();

        self.set(subject.clone(), object.clone(), opinion);
        self.set(object, subject, opinion);
    }
}