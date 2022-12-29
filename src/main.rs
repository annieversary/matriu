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

use music_theory::{Note, Scale};
use state::{Mode, State, MAX_OCTAVE};

mod board;
mod keys;
mod letters;
mod music_theory;
mod state;
mod usb;

#[entry]
fn main() -> ! {
    let mut state = State::new();

    loop {
        state.board.delay.delay_ms(5u8);
        state.update_keys();

        run(&mut state);
        update_colors(&mut state);
    }
}

fn run(state: &mut State) {
    match state.mode {
        Mode::Normal(notes_mode) => {
            if state.key_pressed((0, 0)) {
                state.set_mode(Mode::SelectRoot { hold: false });
            }
            if state.key_pressed((0, 1)) {
                state.set_mode(Mode::Config);
            }
            if state.key_just_pressed((0, 2)) {
                state.set_mode(Mode::Normal(notes_mode.next()));
            }

            state.update_sustain();

            match notes_mode {
                state::NotesMode::Notes => {
                    for col in 1..8 {
                        for row in 0..4 {
                            let note = (state.octave + row) * 12
                                + state.scale.get(col - 1)
                                + state.root as u8;
                            if state.key_just_pressed((col, row)) {
                                state.send_midi(note, true);
                            } else if state.key_just_released((col, row)) {
                                state.send_midi(note, false);
                            }
                        }
                    }
                }
                state::NotesMode::Chords => {
                    for col in 1..8 {
                        macro_rules! chords {
                            ($row:expr, $fun:ident) => {
                                if state.key_just_pressed((col, $row)) {
                                    let root = (state.octave) * 12
                                        + state.scale.get(col - 1)
                                        + state.root as u8;
                                    let chord = state.scale.chords()[(col - 1) as usize];

                                    for note in chord.$fun() {
                                        state.send_midi(note + root, true);
                                    }
                                } else if state.key_just_released((col, $row)) {
                                    let root = (state.octave) * 12
                                        + state.scale.get(col - 1)
                                        + state.root as u8;
                                    let chord = state.scale.chords()[(col - 1) as usize];

                                    for note in chord.$fun() {
                                        state.send_midi(note + root, false);
                                    }
                                }
                            };
                        }
                        chords!(0, notes);
                        chords!(1, first_inv);
                        chords!(2, second_inv);
                    }
                }
                state::NotesMode::ChordsExtra => {
                    // TODO
                }
            }
        }
        Mode::SelectRoot { hold } => {
            if state.key_just_pressed((0, 1)) {
                state.mode = Mode::SelectRoot { hold: true }
            }

            if hold {
                if state.key_just_pressed((0, 0)) || state.key_just_pressed((0, 1)) {
                    state.set_prev_mode();
                }
            } else if !state.key_pressed((0, 0)) {
                state.set_prev_mode();
            }

            macro_rules! select_note {
                ($note:expr, $pos:expr) => {
                    if state.key_just_pressed($pos) {
                        state.root = $note;
                        state.send_midi(6 * 12 + $note as u8, true);
                    } else if state.key_just_released($pos) {
                        state.send_midi(6 * 12 + $note as u8, false);
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
        Mode::Config => {
            if !state.key_pressed((0, 1)) {
                state.set_prev_mode();
            }

            for i in 0..7 {
                if state.key_pressed((i + 1, 0)) {
                    state.scale = Scale::from(i);
                }
            }

            if state.key_just_pressed((0, 3)) {
                state.note_off_all();
            }

            if state.key_just_pressed((6, 1)) {
                state.velocity = state.velocity.saturating_sub(5).max(5);
            }
            if state.key_just_pressed((7, 1)) {
                state.velocity = state.velocity.saturating_add(5).min(126);
            }
            if state.key_just_pressed((6, 2)) {
                state.octave = state.octave.saturating_sub(1);
            }
            if state.key_just_pressed((7, 2)) {
                // TODO not sure what the maximum number should be here
                state.octave = state.octave.saturating_add(1).min(MAX_OCTAVE);
            }
            if state.key_just_pressed((6, 3)) {
                state.brightness = state.brightness.saturating_sub(5).max(5);
            }
            if state.key_just_pressed((7, 3)) {
                state.brightness = state.brightness.saturating_add(5);
            }
        }
    }
}

fn update_colors(state: &mut State) {
    let mut colors = [colors::BLACK; bsp::NEOPIXEL_COUNT];

    match state.mode {
        Mode::Normal(notes_mode) => {
            colors[0] = colors::BLUE;
            colors[8] = colors::BLUE;

            colors[24] = if state.sustain {
                colors::BLUE
            } else {
                colors::CYAN
            };

            for col in 1..8 {
                for row in 0..4 {
                    colors[(col, row).into_index()] = if state.key_pressed((col, row)) {
                        hue(row * 64)
                    } else {
                        colors::BLACK
                    };
                }
            }

            match notes_mode {
                state::NotesMode::Notes => {}
                state::NotesMode::Chords => {
                    colors[16] = colors::LIME_GREEN;
                }
                state::NotesMode::ChordsExtra => {
                    colors[16] = colors::GREEN;
                }
            }
        }
        Mode::SelectRoot { hold } => {
            colors[0] = colors::BLUE;
            colors[8] = if hold { colors::PURPLE } else { colors::PINK };

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
        Mode::Config => {
            colors[8] = colors::BLUE;

            for i in 0..7 {
                colors[i + 1] = if state.scale as usize == i {
                    colors::RED
                } else {
                    colors::LIME_GREEN
                };
            }

            colors[24] = colors::YELLOW;

            colors[6 + 8] = hue(((state.velocity as f32 / 127f32) * 255.0) as u8);
            colors[7 + 8] = hue((((5 + state.velocity) as f32 / 127f32) * 255.0) as u8);
            colors[6 + 2 * 8] =
                hue(((state.octave as f32 / (1 + MAX_OCTAVE) as f32) * 255.0) as u8);
            colors[7 + 2 * 8] =
                hue((((1 + state.octave) as f32 / (1 + MAX_OCTAVE) as f32) * 255.0) as u8);
            colors[6 + 3 * 8] = colors::CYAN;
            colors[7 + 3 * 8] = colors::BLUE;
        }
    }

    state
        .board
        .neopixel
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
