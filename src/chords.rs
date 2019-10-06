use piston::input::keyboard::ModifierKey;
use piston::input::Key;
use std::collections::BTreeMap;
use std::iter::IntoIterator;
use std::cmp;
use std::fmt::Debug;

#[derive(Copy, Clone, Debug)]
pub enum ChordResult<A> {
    Building,
    Action(A),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChordKey {
    Key(Key, ModifierKey),
    Character(char, ModifierKey),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ChordTree<A> where A: Debug {
    keys: BTreeMap<ChordKey, ChordTree<A>>,
    action: Option<ChordAction<A>>,
}

impl <A> ChordTree<A> where A: Debug {
    pub fn new() -> Self {
        ChordTree {
            keys: BTreeMap::new(),
            action: None,
        }
    }

    pub fn insert<'a, K>(&mut self, keys: K, action: ChordAction<A>) where K: IntoIterator<Item=&'a ChordKey> {
        let mut keys = keys.into_iter();
        if let Some(key) = keys.next() {
            let sub_tree = self.keys.entry(key.to_owned()).or_insert_with(|| ChordTree::new());
            sub_tree.insert(keys, action);
        } else {
            self.action = Some(action);
        }
    }

    #[allow(dead_code)]
    pub fn remove<'a, K>(&mut self, keys: K) where K: IntoIterator<Item=&'a ChordKey> {
        let mut keys = keys.into_iter();
        if let Some(key) = keys.next() {
            if let Some(sub_tree) = self.keys.get_mut(key) {
                sub_tree.remove(keys);
            }
        } else {
            self.action = None;
        }
    }

    pub fn get<'a, 'b, K>(&'b self, keys: K) -> Option<&'b ChordTree<A>> where K: IntoIterator<Item=&'a ChordKey> {
        let mut keys = keys.into_iter();
        if let Some(key) = keys.next() {
            if let Some(sub_tree) = self.keys.get(key) {
                sub_tree.get(keys)
            } else {
                None
            }
        } else {
            Some(self)
        }
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.keys.clear();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ChordAction<A> where A: Debug {
    action: A,
}

impl <A> ChordAction<A> where A: Debug {
    pub fn new(action: A) -> Self {
        ChordAction {
            action
        }
    }
}

#[derive(Debug, Clone)]
pub struct Chords<A> where A: Copy + cmp::Ord + Debug {
    chords: ChordTree<A>,
    keys: Vec<ChordKey>,
    actions: BTreeMap<A,String>,
}

impl <A> Chords<A> where A: Copy + cmp::Ord + Debug {
    pub fn new() -> Chords<A> {
        Chords {
            chords: ChordTree::new(),
            keys: vec![],
            actions: BTreeMap::new(),
        }
    }

    pub fn register<'a, K, N>(&mut self, keys: K, action: A, name: N) where K: IntoIterator<Item=&'a ChordKey>, N: Into<String> {
        self.chords.insert(keys, ChordAction::new(action));
        self.actions.insert(action, name.into());
    }

    pub fn perform(&mut self, key: ChordKey) -> Option<ChordResult<A>> {
        self.keys.push(key);

        if let Some(chord) = self.chords.get(&self.keys) {
            if let Some(ref action) = chord.action {
                self.keys.clear();

                Some(ChordResult::Action(action.action))
            } else {
                Some(ChordResult::Building)
            }
        } else {
            self.keys.clear();
            
            None
        }
    }

    pub fn current_status(&self) -> Option<ChordResult<A>> {
        if let Some(chord) = self.chords.get(&self.keys) {
            if let Some(ref action) = chord.action {
                Some(ChordResult::Action(action.action))
            } else {
                Some(ChordResult::Building)
            }
        } else {
            None
        }
    }

    pub fn get_keys<'a>(&'a self) -> &'a [ChordKey] {
        &self.keys
    }

    pub fn get_action_name<'a>(&'a self, action: A) -> Option<&'a String> {
        self.actions.get(&action)
    }

    pub fn clear_chord(&mut self) {
        self.keys.clear();
    }
}
