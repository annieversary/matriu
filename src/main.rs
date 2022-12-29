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
use state::{Keyboard, Mode, State, MAX_OCTAVE};

use crate::music_theory::Chord;

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
        Mode::Normal => {
            if state.key_pressed((0, 0)) {
                state.set_mode(Mode::SelectRoot { hold: false });
            }
            if state.key_pressed((0, 1)) {
                state.set_mode(Mode::Config);
            }

            state.update_sustain();

            match state.keyboard {
                Keyboard::Scale => {
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
                Keyboard::Chords | Keyboard::ChordsExtra => {
                    for col in 1..8 {
                        macro_rules! chords {
                            ($row:expr, $chord:path) => {
                                if state.key_just_pressed((col, $row)) {
                                    let root = (state.octave) * 12
                                        + state.scale.get(col - 1)
                                        + state.root as u8;

                                    for note in $chord.notes() {
                                        state.send_midi(note + root, true);
                                    }
                                } else if state.key_just_released((col, $row)) {
                                    let root = (state.octave) * 12
                                        + state.scale.get(col - 1)
                                        + state.root as u8;

                                    for note in $chord.notes() {
                                        state.send_midi(note + root, false);
                                    }
                                }
                            };
                        }
                        if state.keyboard == Keyboard::Chords {
                            chords!(0, Chord::Major);
                            chords!(1, Chord::Minor);
                            chords!(2, Chord::Diminished);
                            chords!(3, Chord::Power);
                        } else if state.keyboard == Keyboard::ChordsExtra {
                            // TODO maj7, min7, others
                        }
                    }
                }
                Keyboard::Bass => {
                    macro_rules! bass {
                        ($row:expr, $offset:expr) => {
                            for col in 0..7 {
                                let note = state.octave * 12 + state.root as u8 + $offset + col;
                                if state.key_just_pressed((col + 1, $row)) {
                                    state.send_midi(note, true);
                                } else if state.key_just_released((col + 1, $row)) {
                                    state.send_midi(note, false);
                                }
                            }
                        };
                    }

                    bass!(3, 0);
                    bass!(2, 5);
                    bass!(1, 10);
                    bass!(0, 15);
                }
                Keyboard::Waffletone => {
                    for col in 0..7 {
                        for row in 0..4 {
                            let note = state.octave * 12 + state.root as u8 - row + col * 3;
                            if state.key_just_pressed((col + 1, row)) {
                                state.send_midi(note, true);
                            } else if state.key_just_released((col + 1, row)) {
                                state.send_midi(note, false);
                            }
                        }
                    }
                }
            }
        }
        Mode::SelectRoot { hold } => {
            if state.key_just_pressed((0, 1)) {
                state.mode = Mode::SelectRoot { hold: true }
            }

            if hold {
                if state.key_just_pressed((0, 0)) || state.key_just_pressed((0, 1)) {
                    state.set_mode(Mode::Normal);
                }
            } else if !state.key_pressed((0, 0)) {
                state.set_mode(Mode::Normal);
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
                state.set_mode(Mode::Normal);
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

            macro_rules! keyboard {
                ($i:expr, $k:expr) => {
                    if state.key_just_pressed(($i, 3)) {
                        state.keyboard = $k;
                    }
                };
            }
            keyboard!(1, Keyboard::Scale);
            keyboard!(2, Keyboard::Chords);
            keyboard!(3, Keyboard::ChordsExtra);
            keyboard!(4, Keyboard::Bass);
            keyboard!(5, Keyboard::Waffletone);
        }
    }
}

fn update_colors(state: &mut State) {
    let mut colors = [colors::BLACK; bsp::NEOPIXEL_COUNT];

    macro_rules! color {
        ($($color:path => [ $($n:expr),* ] ),*) => {
            $(
                $(
                    colors[$n] = $color;
                )*
            )*
        };
    }

    match state.mode {
        Mode::Normal => {
            color! {
                colors::BLUE => [0, 8]
            }

            colors[24] = if state.sustain {
                colors::BLUE
            } else {
                colors::CYAN
            };

            match state.keyboard {
                Keyboard::Scale | Keyboard::Chords | Keyboard::ChordsExtra => {
                    for col in 1..8 {
                        for row in 0..4 {
                            colors[(col, row).into_index()] = if state.key_pressed((col, row)) {
                                hue(row * 64)
                            } else {
                                colors::BLACK
                            };
                        }
                    }
                }
                Keyboard::Bass => {
                    let notes = state.scale.notes();
                    macro_rules! bass {
                        ($row:expr, $offset:expr) => {
                            for n in 0..7 {
                                let v = (n + $offset) % 12;
                                if notes.contains(&v) {
                                    colors[($row * 8 + n + 1) as usize] = colors::YELLOW;
                                }
                            }
                        };
                    }
                    bass!(3, 0);
                    bass!(2, 5);
                    bass!(1, 10);
                    bass!(0, 15);

                    color!(
                        colors::RED => [11, 25]
                    );
                }
                Keyboard::Waffletone => {
                    let notes = state.scale.notes();
                    for col in 0..7 {
                        for row in 0..4 {
                            let v = ((12 - row) + col * 3) % 12;
                            if notes.contains(&v) {
                                colors[(row * 8 + col + 1) as usize] = colors::YELLOW;
                            }
                        }
                    }
                    color!(
                        colors::RED => [1, 5, 26, 30]
                    );
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

            macro_rules! keyboard {
                ($i:expr, $k:expr) => {
                    if state.keyboard == $k {
                        colors[$i + 3 * 8] = colors::RED;
                    }
                };
            }
            keyboard!(1, Keyboard::Scale);
            keyboard!(2, Keyboard::Chords);
            keyboard!(3, Keyboard::ChordsExtra);
            keyboard!(4, Keyboard::Bass);
            keyboard!(5, Keyboard::Waffletone);
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
