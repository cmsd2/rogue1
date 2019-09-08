use crate::chords::{Chords, ChordKey, ChordResult};
use crate::input::{InputEventKey, InputEventType};
use crate::charmap;
use piston::input::Key;
use piston::input::keyboard::ModifierKey;

pub const NEW_LINE: &'static str = "nl";
pub const BACKSPACE: &'static str = "bs";
pub const LEFT: &'static str = "left";
pub const RIGHT: &'static str = "right";
pub const UP: &'static str = "up";
pub const DOWN: &'static str = "down";
pub const QUIT: &'static str = "quit";

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    NewLine,
    Backspace,
    Left,
    Right,
    Up,
    Down,
    Quit,
}

pub struct Commands {
    pub chords: Chords<Action>,
}

impl Commands {
    pub fn new() -> Self {
        let mut chords = Chords::new();

        chords.register(&[ChordKey::Key(Key::Return, ModifierKey::NO_MODIFIER)], Action::NewLine, NEW_LINE);
        chords.register(&[ChordKey::Key(Key::Backspace, ModifierKey::NO_MODIFIER)], Action::Backspace, BACKSPACE);
        chords.register(&[ChordKey::Key(Key::Left, ModifierKey::NO_MODIFIER)], Action::Left, LEFT);
        chords.register(&[ChordKey::Key(Key::Right, ModifierKey::NO_MODIFIER)], Action::Right, RIGHT);
        chords.register(&[ChordKey::Key(Key::Up, ModifierKey::NO_MODIFIER)], Action::Up, UP);
        chords.register(&[ChordKey::Key(Key::Down, ModifierKey::NO_MODIFIER)], Action::Down, DOWN);
        chords.register(&[
            ChordKey::Character('x', ModifierKey::CTRL),
            ChordKey::Character('c', ModifierKey::CTRL),
        ], Action::Quit, QUIT);

        Commands {
            chords: chords,
        }
    }

    fn get_action_for_key<'a>(&'a mut self, key: Key, modifiers: ModifierKey) -> Option<ChordResult<Action>> {
        match key {
            Key::LCtrl => None,
            Key::RCtrl => None,
            Key::LShift => None,
            Key::RShift => None,
            Key::LAlt => None,
            Key::RAlt => None,
            _ => {
                self.chords.perform(ChordKey::Key(key, modifiers))
            }
        }
    }

    fn get_action_for_char<'a>(&'a mut self, c: char, modifiers: ModifierKey) -> Option<ChordResult<Action>> {
        self.chords.perform(ChordKey::Character(c, modifiers))
    }

    pub fn key_event<'a>(&'a mut self, state: InputEventType, key: InputEventKey) -> Option<ChordResult<Action>> {
        if state == InputEventType::KeyUp {
            return None
        }

        match key {
            InputEventKey::KeyboardKey { character: Some(c), modifiers, .. } if charmap::is_printable(c) => { 
                self.get_action_for_char(c, modifiers)
            },
            InputEventKey::KeyboardKey { key, modifiers, .. } => { 
                self.get_action_for_key(key, modifiers)
            }
            _ => {
                None
            }
        }
    }
}