//! The different actions that can be done.

#![deny(missing_docs)]

use crate::key_code::KeyCode;

/// The different actions that can be done.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Action {
    /// No operation action: just do nothing.
    NoOp,
    /// Transparent, i.e. get the action from the default layer. On
    /// the default layer, it is equivalent to `NoOp`.
    Trans,
    /// A key code, i.e. the classical key.
    KeyCode(KeyCode),
    /// Multiple key codes send at the same time, as it you press all
    /// these keys at the same time.  Useful to send shifted key, or
    /// complex short cuts as Ctrl+Alt+Del in a single key press.
    MultipleKeyCodes(&'static [KeyCode]),
    /// While pressed, change the current layer. That's the classical
    /// Fn key. If several layer actions are active at the same time,
    /// their number are summed. For example, if you press at the same
    /// time `Layer(1)` and `Layer(2)`, layer 3 will be active.
    Layer(usize),
    /// Change the default layer.
    DefaultLayer(usize),
    /// If the key is hold more than `timeout` units of time (usually
    /// milliseconds), performs the `hold` action, else performs the
    /// `tap` action.  Mostly used with a modifier for the hold action
    /// and a classical key on the tap action. Any action can be
    /// performed, but using a `HoldTap` in an `HoldTap` is not
    /// specified (but guaranteed to not crash).
    HoldTap {
        /// The duration, in ticks (usually milliseconds) giving the
        /// difference between a hold and a tap.
        timeout: u16,
        /// The hold action.
        hold: &'static Action,
        /// The tap action.
        tap: &'static Action,
    },
}
impl Action {
    /// Gets the layer number if the action is the `Layer` action.
    pub fn layer(self) -> Option<usize> {
        match self {
            Action::Layer(l) => Some(l),
            _ => None,
        }
    }
    /// Returns an iterator on the `KeyCode` corresponding to the action.
    pub fn key_codes<'a>(&'a self) -> impl Iterator<Item = KeyCode> + 'a {
        match self {
            Action::KeyCode(kc) => core::slice::from_ref(kc).iter().cloned(),
            Action::MultipleKeyCodes(kcs) => kcs.iter().cloned(),
            _ => [].iter().cloned(),
        }
    }
}

/// A shortcut to create a `Action::KeyCode`, useful to create compact
/// layout.
pub const fn k(kc: KeyCode) -> Action {
    Action::KeyCode(kc)
}

/// A shortcut to create a `Action::Layer`, useful to create compact
/// layout.
pub const fn l(layer: usize) -> Action {
    Action::Layer(layer)
}

/// A shortcut to create a `Action::DefaultLayer`, useful to create compact
/// layout.
pub const fn d(layer: usize) -> Action {
    Action::DefaultLayer(layer)
}

/// A shortcut to create a `Action::KeyCode`, useful to create compact
/// layout.
pub const fn m(kcs: &'static [KeyCode]) -> Action {
    Action::MultipleKeyCodes(kcs)
}
