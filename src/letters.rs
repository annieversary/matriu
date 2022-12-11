use crate::music_theory::Note;

/// returns an array describing how to display a letter in a 4x4 square
pub fn letter(note: Note) -> [u8; 16] {
    match note {
        Note::C => [
            0, 1, 1, 1, //
            1, 0, 0, 0, //
            1, 0, 0, 0, //
            0, 1, 1, 1, //
        ],
        Note::Cs => [
            0, 1, 1, 1, //
            1, 0, 0, 2, //
            1, 0, 0, 0, //
            0, 1, 1, 1, //
        ],
        Note::D => [
            1, 1, 1, 0, //
            1, 0, 0, 1, //
            1, 0, 0, 1, //
            1, 1, 1, 0, //
        ],
        Note::Ds => [
            1, 1, 1, 2, //
            1, 0, 0, 1, //
            1, 0, 0, 1, //
            1, 1, 1, 0, //
        ],
        Note::E => [
            1, 1, 0, 0, //
            1, 0, 0, 1, //
            1, 1, 1, 0, //
            1, 1, 1, 1, //
        ],
        Note::F => [
            1, 1, 1, 1, //
            1, 0, 0, 0, //
            1, 1, 1, 0, //
            1, 0, 0, 0, //
        ],
        Note::Fs => [
            1, 1, 1, 1, //
            1, 0, 0, 2, //
            1, 1, 1, 0, //
            1, 0, 0, 0, //
        ],
        Note::G => [
            0, 1, 1, 1, //
            1, 0, 0, 0, //
            1, 0, 1, 1, //
            0, 1, 0, 1, //
        ],
        Note::Gs => [
            0, 1, 1, 1, //
            1, 0, 0, 2, //
            1, 0, 1, 1, //
            0, 1, 0, 1, //
        ],
        Note::A => [
            0, 1, 1, 0, //
            1, 0, 0, 1, //
            1, 1, 1, 1, //
            1, 0, 0, 1, //
        ],
        Note::As => [
            0, 1, 1, 2, //
            1, 0, 0, 1, //
            1, 1, 1, 1, //
            1, 0, 0, 1, //
        ],
        Note::B => [
            1, 1, 1, 0, //
            1, 0, 1, 0, //
            1, 1, 0, 1, //
            1, 1, 1, 0, //
        ],
    }
}
