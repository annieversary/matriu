#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum KeyState {
    #[default]
    Unpressed,
    JustPressed,
    Pressed,
    JustReleased,
}

impl KeyState {
    pub fn pressed(self) -> bool {
        match self {
            KeyState::Unpressed | KeyState::JustReleased => false,
            KeyState::JustPressed | KeyState::Pressed => true,
        }
    }
}

pub trait KeyIndex {
    fn into_index(self) -> usize;
}
impl KeyIndex for u8 {
    fn into_index(self) -> usize {
        self as usize
    }
}
impl KeyIndex for (u8, u8) {
    fn into_index(self) -> usize {
        let (col, row) = self;
        col as usize + row as usize * 8
    }
}
