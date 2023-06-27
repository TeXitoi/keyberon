//! The different actions that can be executed via any given key.

use crate::key_code::KeyCode;
use crate::layout::{StackedIter, WaitingAction};
use core::fmt::Debug;

/// The different types of actions we support for key sequences/macros
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SequenceEvent<K> {
    /// No operation action: just do nothing (a placeholder).
    NoOp,
    /// A keypress/keydown
    Press(K),
    /// Key release/keyup
    Release(K),
    /// A shortcut for `Press(K), Release(K)`
    Tap(K),
    /// For sequences that need to wait a bit before continuing
    Delay {
        /// How long (in ticks) this Delay will last
        duration: u32, // NOTE: This isn't a u16 because that's only max ~65 seconds (assuming 1000 ticks/sec)
    },
    /// Cancels the running sequence and can be used to mark the end of a sequence
    /// instead of using a number of Release() events
    Complete,
}

/// Behavior configuration of HoldTap.
#[non_exhaustive]
#[derive(Clone, Copy)]
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
    /// If there is a press and release of another key, the hold
    /// action is activated.
    ///
    /// This behavior is interesting for fast typist: the different
    /// between hold and tap would more be based on the sequence of
    /// events than on timing. Be aware that doing the good succession
    /// of key might require some training.
    PermissiveHold,
    /// A custom configuration. Allows the behavior to be controlled by a caller
    /// supplied handler function.
    ///
    /// The input to the custom handler will be an iterator that returns
    /// [Stacked] [Events](Event). The order of the events matches the order the
    /// corresponding key was pressed/released, i.e. the first event is the
    /// event first received after the HoldTap action key is pressed.
    ///
    /// The return value should be the intended action that should be used. A
    /// [Some] value will cause one of: [WaitingAction::Tap] for the configured
    /// tap action, [WaitingAction::Hold] for the hold action, and
    /// [WaitingAction::NoOp] to drop handling of the key press. A [None]
    /// value will cause a fallback to the timeout-based approach. If the
    /// timeout is not triggered, the next tick will call the custom handler
    /// again.
    ///
    /// # Example:
    /// Hold events can be prevented from triggering when pressing multiple keys
    /// on the same side of the keyboard (but does not prevent multiple hold
    /// events).
    /// ```
    /// use keyberon::action::{Action, HoldTapConfig, HoldTapAction};
    /// use keyberon::key_code::KeyCode;
    /// use keyberon::layout::{StackedIter, WaitingAction, Event};
    ///
    /// /// Trigger a `Tap` action on the left side of the keyboard if another
    /// /// key on the left side of the keyboard is pressed.
    /// fn left_mod(stacked_iter: StackedIter) -> Option<WaitingAction> {
    ///     match stacked_iter.map(|s| s.event()).find(|e| e.is_press()) {
    ///         Some(Event::Press(_, j)) if j < 6 => Some(WaitingAction::Tap),
    ///         _ => None,
    ///     }
    /// }
    ///
    /// /// Trigger a `Tap` action on the right side of the keyboard if another
    /// /// key on the right side of the keyboard is pressed.
    /// fn right_mod(stacked_iter: StackedIter) -> Option<WaitingAction> {
    ///     match stacked_iter.map(|s| s.event()).find(|e| e.is_press()) {
    ///         Some(Event::Press(_, j)) if j > 5 => Some(WaitingAction::Tap),
    ///         _ => None,
    ///     }
    /// }
    ///
    /// // Assuming a standard QWERTY layout, the left shift hold action will
    /// // not be triggered when pressing Tab-T, CapsLock-G, nor Shift-B.
    /// const A_SHIFT: Action = Action::HoldTap(&HoldTapAction {
    ///     timeout: 200,
    ///     hold: Action::KeyCode(KeyCode::LShift),
    ///     tap: Action::KeyCode(KeyCode::A),
    ///     config: HoldTapConfig::Custom(left_mod),
    ///     tap_hold_interval: 0,
    /// });
    ///
    /// // Assuming a standard QWERTY layout, the right shift hold action will
    /// // not be triggered when pressing Y-Pipe, H-Enter, nor N-Shift.
    /// const SEMI_SHIFT: Action = Action::HoldTap(&HoldTapAction {
    ///     timeout: 200,
    ///     hold: Action::KeyCode(KeyCode::RShift),
    ///     tap: Action::KeyCode(KeyCode::SColon),
    ///     config: HoldTapConfig::Custom(right_mod),
    ///     tap_hold_interval: 0,
    /// });
    /// ```
    Custom(fn(StackedIter) -> Option<WaitingAction>),
}

impl Debug for HoldTapConfig {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            HoldTapConfig::Default => f.write_str("Default"),
            HoldTapConfig::HoldOnOtherKeyPress => f.write_str("HoldOnOtherKeyPress"),
            HoldTapConfig::PermissiveHold => f.write_str("PermissiveHold"),
            HoldTapConfig::Custom(func) => f
                .debug_tuple("Custom")
                .field(&(*func as fn(StackedIter<'static>) -> Option<WaitingAction>) as &dyn Debug)
                .finish(),
        }
    }
}

impl PartialEq for HoldTapConfig {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (HoldTapConfig::Default, HoldTapConfig::Default)
            | (HoldTapConfig::HoldOnOtherKeyPress, HoldTapConfig::HoldOnOtherKeyPress)
            | (HoldTapConfig::PermissiveHold, HoldTapConfig::PermissiveHold) => true,
            (HoldTapConfig::Custom(self_func), HoldTapConfig::Custom(other_func)) => {
                *self_func as fn(StackedIter<'static>) -> Option<WaitingAction> == *other_func
            }
            _ => false,
        }
    }
}

impl Eq for HoldTapConfig {}

/// Perform different actions on key hold/tap.
///
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
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct HoldTapAction<T, K>
where
    T: 'static,
    K: 'static,
{
    /// The duration, in ticks (usually milliseconds) giving the
    /// difference between a hold and a tap.
    pub timeout: u16,
    /// The hold action.
    pub hold: Action<T, K>,
    /// The tap action.
    pub tap: Action<T, K>,
    /// Behavior configuration.
    pub config: HoldTapConfig,
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
    pub tap_hold_interval: u16,
}

/// The different actions that can be done.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Action<T = core::convert::Infallible, K = KeyCode>
where
    T: 'static,
    K: 'static,
{
    /// No operation action: just do nothing.
    NoOp,
    /// Transparent, i.e. get the action from the default layer. On
    /// the default layer, it is equivalent to `NoOp`.
    Trans,
    /// A key code, i.e. a classic key.
    KeyCode(K),
    /// Multiple key codes sent at the same time, as if these keys
    /// were pressed at the same time. Useful to send a shifted key,
    /// or complex shortcuts like Ctrl+Alt+Del in a single key press.
    MultipleKeyCodes(&'static &'static [K]),
    /// Multiple actions sent at the same time.
    MultipleActions(&'static &'static [Action<T, K>]),
    /// While pressed, change the current layer. That's the classic
    /// Fn key. If several layer actions are hold at the same time,
    /// the last pressed defines the current layer.
    Layer(usize),
    /// Change the default layer.
    DefaultLayer(usize),
    /// Perform different actions on key hold/tap (see [`HoldTapAction`]).
    HoldTap(&'static HoldTapAction<T, K>),
    /// A sequence of SequenceEvents
    Sequence(
        /// An array of SequenceEvents that will be triggered (in order)
        &'static &'static [SequenceEvent<K>],
    ),
    /// Custom action.
    ///
    /// Define a user defined action. This enum can be anything you
    /// want, as long as it has the `'static` lifetime. It can be used
    /// to drive any non keyboard related actions that you might
    /// manage with key events.
    Custom(T),
}
impl<T, K: Clone> Action<T, K> {
    /// Gets the layer number if the action is the `Layer` action.
    pub fn layer(self) -> Option<usize> {
        match self {
            Action::Layer(l) => Some(l),
            _ => None,
        }
    }
    /// Returns an iterator on the `KeyCode` corresponding to the action.
    pub fn key_codes(&self) -> impl Iterator<Item = K> + '_ {
        match self {
            Action::KeyCode(kc) => core::slice::from_ref(kc).iter().cloned(),
            Action::MultipleKeyCodes(kcs) => kcs.iter().cloned(),
            _ => [].iter().cloned(),
        }
    }
}

/// A shortcut to create a `Action::KeyCode`, useful to create compact
/// layout.
pub const fn k<T, K>(kc: K) -> Action<T, K> {
    Action::KeyCode(kc)
}

/// A shortcut to create a `Action::Layer`, useful to create compact
/// layout.
pub const fn l<T, K>(layer: usize) -> Action<T, K> {
    Action::Layer(layer)
}

/// A shortcut to create a `Action::DefaultLayer`, useful to create compact
/// layout.
pub const fn d<T, K>(layer: usize) -> Action<T, K> {
    Action::DefaultLayer(layer)
}

/// A shortcut to create a `Action::MultipleKeyCodes`, useful to
/// create compact layout.
pub const fn m<T, K>(kcs: &'static &'static [K]) -> Action<T, K> {
    Action::MultipleKeyCodes(kcs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem;

    #[test]
    fn size_of_action() {
        const PTR_SIZE: usize = mem::size_of::<&()>();
        assert_eq!(mem::size_of::<Action::<(), ()>>(), 2 * PTR_SIZE);
    }
}
