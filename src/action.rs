//! The different actions that can be executed via any given key.

use crate::key_code::KeyCode;

/// The different types of actions we support for key macros
#[non_exhaustive] // Definitely NOT exhaustive!  Let's add more! Mouse events maybe? :)
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SequenceEvent {
    /// A keypress/keydown
    Press(KeyCode),
    /// Key release/keyup
    Release(KeyCode),
    /// A shortcut for `Press(KeyCode), Release(KeyCode)`
    Tap(KeyCode),
    /// For sequences that need to wait a bit before continuing
    Delay {
        /// A delay (in ticks) to wait before executing the next SequenceEvent
        since: u32,
        /// Number of ticks to wait before removing the Delay
        ticks: u32,
    },
    /// A marker that indicates there's more of the macro than would fit
    /// in the 'sequenced' ArrayDeque
    Continue {
        /// The current chunk
        index: usize,
        /// The full list of Sequence Events (that aren't Continue())
        events: &'static [SequenceEvent],
    },
    /// Cancels the running sequence and can be used to mark the end of a sequence
    /// instead of using a number of Release() events
    Complete,
}

impl SequenceEvent {
    /// Returns the keycode associated with the given Press/Release event
    pub fn keycode(&self) -> Option<KeyCode> {
        match *self {
            SequenceEvent::Press(keycode) => Some(keycode),
            SequenceEvent::Release(keycode) => Some(keycode),
            _ => None,
        }
    }
}

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
    /// A key code, i.e. the classical key.
    KeyCode(KeyCode),
    /// Multiple key codes send at the same time, as it you press all
    /// these keys at the same time.  Useful to send shifted key, or
    /// complex short cuts as Ctrl+Alt+Del in a single key press.
    MultipleKeyCodes(&'static [KeyCode]),
    /// Multiple actions send at the same time.
    MultipleActions(&'static [Action<T>]),
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
        hold: &'static Action<T>,
        /// The tap action.
        tap: &'static Action<T>,
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
    /// A sequence of SequenceEvents
    Sequence {
        /// An array of SequenceEvents that will be triggered (in order)
        events: &'static [SequenceEvent],
    },
    /// Cancels any running sequences
    CancelSequence,
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
