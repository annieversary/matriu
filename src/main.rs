#![allow(deprecated)]
#![no_std]
#![no_main]

use trellis_m4 as bsp;

#[cfg(not(feature = "use_semihosting"))]
use panic_halt as _;
#[cfg(feature = "use_semihosting")]
use panic_semihosting as _;

use bsp::{entry, hal::prelude::*};
use smart_leds::{
    brightness,
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite,
};
use usbd_midi::data::midi::notes::Note;

use board::{Board, Neopixel};
use state::State;

mod board;
mod state;
mod usb;

#[entry]
fn main() -> ! {
    let mut board = Board::new();
    let mut state = State::new();

    loop {
        board.delay.delay_ms(100u8);
        state.update(&board.keypad);

        update_colors(&state, &mut board.neopixel);

        if state.key_pressed(1) {
            usb::send_midi(Note::C4, 70);
        }
    }
}

fn update_colors(state: &State, neopixel: &mut Neopixel) {
    // TODO make something that shows the actual colors
    let colors = state
        .keys
        .iter()
        .map(|key| match key {
            state::KeyState::Unpressed => 0,
            state::KeyState::JustPressed => 50,
            state::KeyState::Pressed => 100,
            state::KeyState::JustReleased => 150,
        })
        .map(|hue| {
            hsv2rgb(Hsv {
                hue,
                sat: 255,
                val: 255,
            })
        });

    neopixel.write(brightness(colors, 32)).unwrap();
}
