use cortex_m::prelude::_embedded_hal_blocking_delay_DelayUs;
use trellis_m4 as bsp;

use bsp::hal::ehal::digital::v2::InputPin;
use usbd_midi::midi_types;

use crate::{
    board::Board,
    keys::*,
    music_theory::{Note, Scale},
    usb::send_midi,
};

pub const MAX_OCTAVE: u8 = 8;

pub struct State {
    pub board: Board,

    pub keys: [KeyState; bsp::NEOPIXEL_COUNT],

    pub mode: Mode,
    pub keyboard: Keyboard,

    pub brightness: u8,

    pub scale: Scale,
    pub root: Note,
    pub octave: u8,
    pub velocity: u8,

    pub sustain: bool,

    active_notes: [bool; 127],
    pub sustained_notes: [bool; 127],
}
impl State {
    pub fn new() -> Self {
        Self {
            board: Board::new(),

            keys: [KeyState::Unpressed; bsp::NEOPIXEL_COUNT],

            mode: Mode::Normal,
            keyboard: Keyboard::Scale,

            brightness: 30,

            scale: Scale::Ionian,
            root: Note::C,
            octave: 3,
            velocity: 70,

            sustain: false,

            active_notes: [false; 127],
            sustained_notes: [false; 127],
        }
    }

    /// Updates the KeyState of every key
    pub fn update_keys(&mut self) {
        let keypad_inputs = self.board.keypad.decompose();

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

    pub fn update_sustain(&mut self) {
        if self.key_just_pressed((0, 3)) {
            self.sustain = !self.sustain;
            for i in 0..127u8 {
                if self.sustained_notes[i as usize] {
                    self.send_midi(i, false);
                }
            }
        }
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;

        for i in 0..127u8 {
            if self.active_notes[i as usize] {
                self.send_midi(i, false);
            }
        }
    }

    pub fn send_midi(&mut self, midi_num: u8, on: bool) {
        let midi_num = midi_num.min(126);

        // dont do anything if the note is already active
        if self.active_notes[midi_num as usize] && on {
            return;
        }

        // TODO make sustain pedal only sustian the chords that are played while it's held
        // this way we can sustain some notes and not others

        if on {
            self.active_notes[midi_num as usize] = true;
            if self.sustain {
                self.sustained_notes[midi_num as usize] = true;
            }

            let note = midi_types::Note::new(midi_num);
            send_midi(note, self.velocity, true);
        } else if !self.sustain {
            self.active_notes[midi_num as usize] = false;
            self.sustained_notes[midi_num as usize] = false;

            let note = midi_types::Note::new(midi_num);
            send_midi(note, 0, false);
        } else {
            self.sustained_notes[midi_num as usize] = true;
        }

        self.board.delay.delay_us(150u8)
    }

    pub fn note_off_all(&mut self) {
        self.sustain = false;
        for i in 0..127u8 {
            self.send_midi(i, false);
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

#[derive(Copy, Clone)]
pub enum Mode {
    Normal,
    SelectRoot { hold: bool },
    Config,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Keyboard {
    Scale,
    Chords,
    Sampler,
    Bass,
    Waffletone,
}
