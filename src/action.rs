use crate::key_code::KeyCode;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Action {
    No,
    Trans,
    KeyCode(KeyCode),
    Layer(usize),
    DefaultLayer(usize),
}
impl Action {
    pub fn layout(self) -> Option<usize> {
        match self {
            Action::Layer(l) => Some(l),
            _ => None,
        }
    }
    pub fn key_code(self) -> Option<KeyCode> {
        match self {
            Action::KeyCode(kc) => Some(kc),
            _ => None,
        }
    }
}
pub const fn k(kc: KeyCode) -> Action {
    Action::KeyCode(kc)
}
pub const fn l(layer: usize) -> Action {
    Action::Layer(layer)
}
pub const fn d(layer: usize)-> Action {
    Action::DefaultLayer(layer)
}
