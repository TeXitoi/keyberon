//! The different actions that can be done.

use crate::key_code::KeyCode;

/// Behavior configuration of HoldTap.
#[non_exhaustive]
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
pub enum Action<T = core::convert::Infallible>
where
    T: 'static,
{
    /// No operation action: just do nothing.
    NoOp,
    /// Transparent, i.e. get the action from the default layer. On
    /// the default layer, it is equivalent to `NoOp`.
    Trans,
    /// A key code, i.e. a classic key.
    KeyCode(KeyCode),
    /// Multiple key codes sent at the same time, as if these keys
    /// were pressed at the same time. Useful to send a shifted key,
    /// or complex shortcuts like Ctrl+Alt+Del in a single key press.
    MultipleKeyCodes(&'static [KeyCode]),
    /// Multiple actions sent at the same time.
    MultipleActions(&'static [Action<T>]),
    /// While pressed, change the current layer. That's the classic
    /// Fn key. If several layer actions are hold at the same time,
    /// the last pressed defines the current layer.
    Layer(usize),
    /// Change the default layer.
    DefaultLayer(usize),
    /// If the key is held more than `timeout` ticks (usually
    /// milliseconds), performs the `hold` action, else performs the
    /// `tap` action.  Mostly used with a modifier for the hold action
    /// and a normal key on the tap action. Any action can be
    /// performed, but using a `HoldTap` in a `HoldTap` is not
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
        hold: &'static Action<T>,
        /// The tap action.
        tap: &'static Action<T>,
        /// Behavior configuration.
        config: HoldTapConfig,
        /// Configuration of the tap and hold holds the tap action.
        ///
        /// If you press and release the key in such a way that the tap
        /// action is performed, and then press it again in less than
        /// `tap_hold_interval` ticks, the tap action will
        /// be held. This allows the tap action to be held by
        /// pressing, releasing and holding the key, allowing the computer
        /// to auto repeat the tap behavior. The timeout starts on the
        /// first press of the key, NOT on the release.
        ///
        /// Pressing a different key in between will not result in the
        /// behaviour described above; the HoldTap key must be pressed twice
        /// in a row.
        ///
        /// To deactivate the functionality, set this to 0.
        tap_hold_interval: u16,
    },
    /// Custom action.
    ///
    /// Define a user defined action. This enum can be anything you
    /// want, as long as it has the `'static` lifetime. It can be used
    /// to drive any non keyboard related actions that you might
    /// manage with key events.
    Custom(T),
}
impl<T> Action<T> {
    /// Gets the layer number if the action is the `Layer` action.
    pub fn layer(self) -> Option<usize> {
        match self {
            Action::Layer(l) => Some(l),
            _ => None,
        }
    }
    /// Returns an iterator on the `KeyCode` corresponding to the action.
    pub fn key_codes(&self) -> impl Iterator<Item = KeyCode> + '_ {
        match self {
            Action::KeyCode(kc) => core::slice::from_ref(kc).iter().cloned(),
            Action::MultipleKeyCodes(kcs) => kcs.iter().cloned(),
            _ => [].iter().cloned(),
        }
    }
}

/// A shortcut to create a `Action::KeyCode`, useful to create compact
/// layout.
pub const fn k<T>(kc: KeyCode) -> Action<T> {
    Action::KeyCode(kc)
}

/// A shortcut to create a `Action::Layer`, useful to create compact
/// layout.
pub const fn l<T>(layer: usize) -> Action<T> {
    Action::Layer(layer)
}

/// A shortcut to create a `Action::DefaultLayer`, useful to create compact
/// layout.
pub const fn d<T>(layer: usize) -> Action<T> {
    Action::DefaultLayer(layer)
}

/// A shortcut to create a `Action::MultipleKeyCodes`, useful to
/// create compact layout.
pub const fn m<T>(kcs: &'static [KeyCode]) -> Action<T> {
    Action::MultipleKeyCodes(kcs)
}
