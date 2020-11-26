//! The different actions that can be done.

use crate::key_code::KeyCode;

/// Behavior configuration of HoldTap.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum HoldTapConfig {
    /// Only the timeout will determine between hold and tap action.
    ///
    /// This is a sane default.
    Default,
    /// If there is a key press, the hold action is activated.
    ///
    /// This behavior is interesting for a key which the tap action is
    /// not used in the flow of typing, like escape for example. If
    /// you are annoyed by accidental tap, you can try this behavior.
    HoldOnOtherKeyPress,
    /// If there is a release and a press of another key, the hold
    /// action is activated.
    ///
    /// This behavior is interesting for fast typist: the different
    /// between hold and tap would more be based on the sequence of
    /// events than on timing. Be aware that doing the good succession
    /// of key might require some training.
    PermissiveHold,
}

/// The different actions that can be done.
#[non_exhaustive]
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
    /// Multiple actions send at the same time.
    MultipleActions(&'static [Action]),
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
    ///
    /// Different behaviors can be configured using the config field,
    /// but whatever the configuration is, if the key is pressed more
    /// than `timeout`, the hold action is activated (if no other
    /// action was determined before).
    HoldTap {
        /// The duration, in ticks (usually milliseconds) giving the
        /// difference between a hold and a tap.
        timeout: u16,
        /// The hold action.
        hold: &'static Action,
        /// The tap action.
        tap: &'static Action,
        /// Behavior configuration.
        config: HoldTapConfig,
        /// Configuration of the tap and hold holds the tap action.
        ///
        /// If you press, release the key in such a configuration that
        /// the tap behavior is done, and then press again the key in
        /// less than `tap_hold_interval` ticks, the tap action will
        /// be used. This allow to have a tap action holded by
        /// "tap+hold" the key, allowing the computer to auto repeat
        /// the tap behavior.
        ///
        /// To desactivate the functionnality, set this to 0.
        ///
        /// Not implemented yet, to not have behavior change with an
        /// update, set this to 0.
        tap_hold_interval: u16,
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
