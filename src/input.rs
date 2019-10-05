use std::collections::HashMap;
use std::collections::hash_map::Entry;
use piston::input::*;
use piston::input::keyboard::ModifierKey;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyboardKey {
    pub button: Button,
    pub scancode: Option<i32>,
}

impl From<ButtonArgs> for KeyboardKey {
    fn from(ba: ButtonArgs) -> KeyboardKey {
        KeyboardKey {
            button: ba.button,
            scancode: ba.scancode,
        }
    }
}

impl From<&ButtonArgs> for KeyboardKey {
    fn from(ba: &ButtonArgs) -> KeyboardKey {
        KeyboardKey {
            button: ba.button,
            scancode: ba.scancode,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum InputEventType {
    KeyDown,
    KeyRepeat,
    KeyUp,
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum InputEventKey {
    KeyboardKey {
        key: Key,
        scancode: Option<i32>,
        character:  Option<char>,
        modifiers: ModifierKey,
    },
    Controller(ControllerButton),
    Hat(ControllerHat),
    Mouse(MouseButton),
}

impl InputEventKey {
    pub fn new(k: KeyboardKey, modifiers: ModifierKey) -> InputEventKey {
        match k.button {
            Button::Keyboard(key) => {
                let c = crate::charmap::modify(key.code(), k.scancode, modifiers);

                InputEventKey::KeyboardKey {
                    key: key,
                    scancode: k.scancode,
                    character: c,
                    modifiers: modifiers
                }
            },
            Button::Controller(button) => {
                InputEventKey::Controller(button)
            },
            Button::Hat(button) => {
                InputEventKey::Hat(button)
            },
            Button::Mouse(button) => {
                InputEventKey::Mouse(button)
            }
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct InputEvent {
    pub state: InputEventType,
    pub modifiers: keyboard::ModifierKey,
    pub key: InputEventKey,
}

struct KeyRepeatTimer {
    pub button_repeat_delay: f64,
    pub elapsed: f64,
}

impl KeyRepeatTimer {
    pub fn new(delay: f64) -> Self {
        KeyRepeatTimer {
            button_repeat_delay: delay,
            elapsed: 0.0,
        }
    }
}

pub struct InputHandler {
    pressed: HashMap<KeyboardKey,KeyRepeatTimer>,
    pub button_repeat_delay: f64,
    pub button_repeat_initial_delay: f64,
    pub modifier_keys: keyboard::ModifierKey,
}

impl Default for InputHandler {
    fn default() -> Self {
        InputHandler::new((0.2, 0.1))
    }
}

impl InputHandler {
    pub fn new(button_repeat_delay: (f64, f64)) -> Self {
        let (initial, thereafter) = button_repeat_delay;

        InputHandler {
            pressed: HashMap::new(),
            button_repeat_initial_delay: initial,
            button_repeat_delay: thereafter,
            modifier_keys: keyboard::ModifierKey::default(),
        }
    }

    pub fn is_pressed(&self, button: &ButtonArgs) -> bool {
        self.pressed.contains_key(&KeyboardKey::from(button))
    }

    pub fn press(&mut self, button: &ButtonArgs) -> Vec<InputEvent> {
        let mut events = vec![];
        match self.pressed.entry(KeyboardKey::from(button)) {
            Entry::Vacant(v) => {
                events.push(InputEvent {
                    state: InputEventType::KeyDown,
                    key: InputEventKey::new(*v.key(), self.modifier_keys),
                    modifiers: self.modifier_keys
                });
                v.insert(KeyRepeatTimer::new(self.button_repeat_initial_delay));                
            },
            _ => {}
        }
        events
    }

    pub fn unpress(&mut self, button: &ButtonArgs) -> Vec<InputEvent> {
        let mut events = vec![];
        if self.pressed.remove(&KeyboardKey::from(button)).is_some() {
            events.push(InputEvent {
                state: InputEventType::KeyUp,
                key: InputEventKey::new(KeyboardKey::from(button), self.modifier_keys),
                modifiers: self.modifier_keys
            });
        }
        events
    }

    pub fn event(&mut self, e: &Event) -> Vec<InputEvent> {
        let mut events = vec![];
        self.modifier_keys.event(e);

        if let Some(b) = e.button_args() {
            events.append(&mut self.button(&b));
        }

        if let Some(u) = e.update_args() {
            events.append(&mut self.update(&u));
        }

        events
    }

    pub fn button(&mut self, args: &ButtonArgs) -> Vec<InputEvent> {
        match args.state {
            ButtonState::Press => {
                self.press(args)
            },
            ButtonState::Release => {
                self.unpress(args)
            },
        }
    }

    pub fn update(&mut self, args: &UpdateArgs) -> Vec<InputEvent> {
        let mut events = vec![];
        for (k,v) in self.pressed.iter_mut() {
            v.elapsed += args.dt;

            if v.elapsed >= v.button_repeat_delay {
                v.elapsed -= v.button_repeat_delay;
                v.button_repeat_delay = self.button_repeat_delay;

                events.push(InputEvent { state: InputEventType::KeyRepeat, key: InputEventKey::new(*k, self.modifier_keys), modifiers: self.modifier_keys });
            }
        }
        events
    }
}
