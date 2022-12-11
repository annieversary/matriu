use trellis_m4 as bsp;

use bsp::{hal::ehal::digital::v2::InputPin, Keypad};

use crate::keys::*;

pub struct State {
    pub keys: [KeyState; bsp::NEOPIXEL_COUNT],
    pub mode: Mode,

    pub brightness: u8,

    pub scale: Scale,
    pub root: Note,
    pub octave: u8,
    pub velocity: u8,
}
impl State {
    pub fn new() -> Self {
        Self {
            keys: [KeyState::Unpressed; bsp::NEOPIXEL_COUNT],
            mode: Mode::Normal,

            brightness: 30,

            scale: Scale::Major,
            root: Note::C,
            octave: 3,
            velocity: 70,
        }
    }

    /// Updates the KeyState of every key
    pub fn update_keys(&mut self, keypad: &Keypad) {
        let keypad_inputs = keypad.decompose();

        for i in 0..bsp::NEOPIXEL_COUNT {
            let keypad_column = i % 8;
            let keypad_row = i / 8;
            let keypad_button: &dyn InputPin<Error = ()> =
                &keypad_inputs[keypad_row][keypad_column];

            let pressed = !keypad_button.is_high().unwrap();
            self.keys[i] = match self.keys[i] {
                KeyState::Unpressed | KeyState::JustReleased if pressed => KeyState::JustPressed,
                KeyState::JustPressed if pressed => KeyState::Pressed,
                KeyState::Pressed | KeyState::JustPressed if !pressed => KeyState::JustReleased,
                KeyState::JustReleased if !pressed => KeyState::Unpressed,
                keep => keep,
            };
        }
    }

    pub fn key_pressed(&self, i: impl KeyIndex) -> bool {
        self.keys[i.into_index()].pressed()
    }

    pub fn key_just_pressed(&self, i: impl KeyIndex) -> bool {
        self.keys[i.into_index()] == KeyState::JustPressed
    }

    pub fn key_just_released(&self, i: impl KeyIndex) -> bool {
        self.keys[i.into_index()] == KeyState::JustReleased
    }
}

pub enum Mode {
    Normal,
    SelectKey,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Note {
    C,
    Cs,
    D,
    Ds,
    E,
    F,
    Fs,
    G,
    Gs,
    A,
    As,
    B,
}

impl Note {
    pub const fn sharp(self) -> bool {
        match self {
            Note::C => false,
            Note::Cs => true,
            Note::D => false,
            Note::Ds => true,
            Note::E => false,
            Note::F => false,
            Note::Fs => true,
            Note::G => false,
            Note::Gs => true,
            Note::A => false,
            Note::As => true,
            Note::B => false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Scale {
    Major,
}

impl Scale {
    // `i` is from 0 to 7
    pub const fn get(self, i: u8) -> u8 {
        assert!(i <= 7);
        self.notes()[i as usize]
    }
    pub const fn notes(self) -> [u8; 7] {
        match self {
            Scale::Major => [0, 2, 4, 5, 7, 9, 11],
        }
    }
}
