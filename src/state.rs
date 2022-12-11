use trellis_m4 as bsp;

use bsp::{hal::ehal::digital::v2::InputPin, Keypad};

pub struct State {
    pub keys: [KeyState; bsp::NEOPIXEL_COUNT],
}
impl State {
    pub fn new() -> Self {
        Self {
            keys: [KeyState::Unpressed; bsp::NEOPIXEL_COUNT],
        }
    }

    pub fn update(&mut self, keypad: &Keypad) {
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

    pub fn key_pressed(&self, i: usize) -> bool {
        self.keys[i].pressed()
    }
}

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum KeyState {
    #[default]
    Unpressed,
    JustPressed,
    Pressed,
    JustReleased,
}

impl KeyState {
    fn pressed(self) -> bool {
        match self {
            KeyState::Unpressed | KeyState::JustReleased => false,
            KeyState::JustPressed | KeyState::Pressed => true,
        }
    }

    fn unpressed(self) -> bool {
        !self.pressed()
    }
}
