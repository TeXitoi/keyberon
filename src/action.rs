use crate::key_code::KeyCode;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Action {
    No,
    Trans,
    KC(KeyCode),
    Lt(usize),
}
impl Action {
    pub fn layout(self) -> Option<usize> {
        match self {
            Action::Lt(l) => Some(l),
            _ => None,
        }
    }
    pub fn key_code(self) -> Option<KeyCode> {
        match self {
            Action::KC(kc) => Some(kc),
            _ => None,
        }
    }
}
pub const fn k(kc: KeyCode) -> Action {
    Action::KC(kc)
}
pub const fn l(layer: usize) -> Action {
    Action::Lt(layer)
}
