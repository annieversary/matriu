#![allow(deprecated)]
#![no_std]
#![no_main]

use keys::KeyIndex;
use trellis_m4 as bsp;

#[cfg(not(feature = "use_semihosting"))]
use panic_halt as _;
#[cfg(feature = "use_semihosting")]
use panic_semihosting as _;

use bsp::{entry, hal::prelude::*};
use smart_leds::{
    brightness, colors,
    hsv::{hsv2rgb, Hsv, RGB8},
    SmartLedsWrite,
};
use usb::send_midi;

use board::{Board, Neopixel};
use music_theory::{Note, Scale};
use state::{Mode, State};
use usbd_midi::midi_types;

mod board;
mod keys;
mod letters;
mod music_theory;
mod state;
mod usb;

#[entry]
fn main() -> ! {
    let mut board = Board::new();
    let mut state = State::new();

    loop {
        board.delay.delay_ms(5u8);
        state.update_keys(&board.keypad);

        run(&mut state);
        update_colors(&state, &mut board.neopixel);
    }
}

fn run(state: &mut State) {
    match state.mode {
        // TODO make something for easy keymaps
        Mode::Normal => {
            if state.key_pressed((0, 0)) {
                state.mode = Mode::SelectRoot;
            }
            if state.key_pressed((0, 1)) {
                state.mode = Mode::SelectScale;
            }

            for col in 1..8 {
                for row in 0..4 {
                    let note = midi_types::Note::new(
                        (state.octave + row) * 12 + state.scale.get(col - 1) + state.root as u8,
                    );
                    if state.key_just_pressed((col, row)) {
                        send_midi(note, state.velocity, true);
                    } else if state.key_just_released((col, row)) {
                        send_midi(note, state.velocity, false);
                    }
                }
            }
        }
        Mode::SelectRoot => {
            if !state.key_pressed((0, 0)) {
                state.mode = Mode::Normal;
            }

            macro_rules! select_note {
                ($note:expr, $pos:expr) => {
                    if state.key_pressed($pos) {
                        state.root = $note;
                    }
                };
            }
            use Note::*;
            select_note!(C, (5, 0));
            select_note!(Cs, (6, 0));
            select_note!(D, (7, 0));
            select_note!(Ds, (5, 1));
            select_note!(E, (6, 1));
            select_note!(F, (7, 1));
            select_note!(Fs, (5, 2));
            select_note!(G, (6, 2));
            select_note!(Gs, (7, 2));
            select_note!(A, (5, 3));
            select_note!(As, (6, 3));
            select_note!(B, (7, 3));
        }
        Mode::SelectScale => {
            if !state.key_pressed((0, 1)) {
                state.mode = Mode::Normal;
            }

            for i in 0..7 {
                if state.key_pressed((i + 1, 0)) {
                    state.scale = Scale::from(i);
                }
            }
        }
    }
}

fn update_colors(state: &State, neopixel: &mut Neopixel) {
    let mut colors = [colors::BLACK; bsp::NEOPIXEL_COUNT];

    match state.mode {
        Mode::Normal => {
            for row in 0..4 {
                colors[(0, row).into_index()] = colors::BLUE;
            }
            for col in 1..8 {
                for row in 0..4 {
                    colors[(col, row).into_index()] = if state.key_pressed((col, row)) {
                        hue(0)
                    } else {
                        colors::BLACK
                    };
                }
            }
        }
        Mode::SelectRoot => {
            colors[0] = colors::BLUE;

            macro_rules! color_note {
                ($note:expr, ($col:expr, $row:expr)) => {
                    colors[$col + $row * 8] = if state.root == $note {
                        colors::RED
                    } else if $note.sharp() {
                        colors::LIME_GREEN
                    } else {
                        colors::GREEN
                    };
                };
            }
            use Note::*;

            color_note!(C, (5, 0));
            color_note!(Cs, (6, 0));
            color_note!(D, (7, 0));
            color_note!(Ds, (5, 1));
            color_note!(E, (6, 1));
            color_note!(F, (7, 1));
            color_note!(Fs, (5, 2));
            color_note!(G, (6, 2));
            color_note!(Gs, (7, 2));
            color_note!(A, (5, 3));
            color_note!(As, (6, 3));
            color_note!(B, (7, 3));

            let letter = letters::letter(state.root);
            for i in 0..4 {
                for j in 0..4 {
                    colors[1 + i + j * 8] = match letter[i + j * 4] {
                        1 => colors::YELLOW,
                        2 => colors::RED,
                        _ => colors::BLACK,
                    };
                }
            }
        }
        Mode::SelectScale => {
            colors[8] = colors::BLUE;

            for i in 0..7 {
                colors[i + 1] = if state.scale as usize == i {
                    colors::RED
                } else {
                    colors::LIME_GREEN
                };
            }
        }
    }

    neopixel
        .write(brightness(colors.into_iter(), state.brightness))
        .unwrap();
}

fn hue(hue: u8) -> RGB8 {
    hsv2rgb(Hsv {
        hue,
        sat: 255,
        val: 255,
    })
}
