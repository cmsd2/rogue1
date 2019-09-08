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

pub trait InputListener {
    fn button_event(&mut self, args: InputEvent);
}

impl <F> InputListener for F where F: Fn(InputEvent) -> () + 'static {
    fn button_event(&mut self, args: InputEvent) {
        self(args)
    }
}

pub struct InputHandler {
    pressed: HashMap<KeyboardKey,KeyRepeatTimer>,
    pub button_repeat_delay: f64,
    pub button_repeat_initial_delay: f64,
    pub listener: Box<dyn InputListener>,
    pub modifier_keys: keyboard::ModifierKey,
}

impl InputHandler {
    pub fn new<F>(button_repeat_delay: (f64, f64), listener: F) -> Self where F: InputListener + 'static {
        let (initial, thereafter) = button_repeat_delay;

        InputHandler {
            pressed: HashMap::new(),
            button_repeat_initial_delay: initial,
            button_repeat_delay: thereafter,
            listener: Box::new(listener),
            modifier_keys: keyboard::ModifierKey::default(),
        }
    }

    pub fn is_pressed(&self, button: &ButtonArgs) -> bool {
        self.pressed.contains_key(&KeyboardKey::from(button))
    }

    pub fn press(&mut self, button: &ButtonArgs) {
        match self.pressed.entry(KeyboardKey::from(button)) {
            Entry::Vacant(v) => {
                self.listener.button_event(InputEvent {
                    state: InputEventType::KeyDown,
                    key: InputEventKey::new(*v.key(), self.modifier_keys),
                    modifiers: self.modifier_keys
                });
                v.insert(KeyRepeatTimer::new(self.button_repeat_initial_delay));                
            },
            _ => {}
        }
    }

    pub fn unpress(&mut self, button: &ButtonArgs) {
        if self.pressed.remove(&KeyboardKey::from(button)).is_some() {
            self.listener.button_event(InputEvent {
                state: InputEventType::KeyUp,
                key: InputEventKey::new(KeyboardKey::from(button), self.modifier_keys),
                modifiers: self.modifier_keys
            });
        }
    }

    pub fn event(&mut self, e: &Event) {
        self.modifier_keys.event(e);

        if let Some(b) = e.button_args() {
            self.button(&b);
        }

        if let Some(u) = e.update_args() {
            self.update(&u);
        }
    }

    pub fn button(&mut self, args: &ButtonArgs) {
        match args.state {
            ButtonState::Press => {
                self.press(args);
            },
            ButtonState::Release => {
                self.unpress(args);
            }
        }
    }

    pub fn update(&mut self, args: &UpdateArgs) {
        for (k,v) in self.pressed.iter_mut() {
            v.elapsed += args.dt;

            if v.elapsed >= v.button_repeat_delay {
                v.elapsed -= v.button_repeat_delay;
                v.button_repeat_delay = self.button_repeat_delay;

                self.listener.button_event(InputEvent { state: InputEventType::KeyRepeat, key: InputEventKey::new(*k, self.modifier_keys), modifiers: self.modifier_keys });
            }
        }
    }
}
