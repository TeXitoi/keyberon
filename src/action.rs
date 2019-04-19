use crate::key_code::KeyCode;
use core::iter;
use either::Either;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Action {
    No,
    Trans,
    KeyCode(KeyCode),
    MultipleKeyCodes(&'static [KeyCode]),
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
    pub fn key_codes(self) -> impl Iterator<Item = KeyCode> {
        match self {
            Action::KeyCode(kc) => Either::Left(iter::once(kc)),
            Action::MultipleKeyCodes(kcs) => Either::Right(kcs.iter().cloned()),
            _ => Either::Right([].iter().cloned()),
        }
    }
}
pub const fn k(kc: KeyCode) -> Action {
    Action::KeyCode(kc)
}
pub const fn l(layer: usize) -> Action {
    Action::Layer(layer)
}
pub const fn d(layer: usize) -> Action {
    Action::DefaultLayer(layer)
}
pub const fn m(kcs: &'static [KeyCode]) -> Action {
    Action::MultipleKeyCodes(kcs)
}
