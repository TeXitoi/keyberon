//! Layout management.

/// A procedural macro to generate [Layers](type.Layers.html)
/// ## Syntax
/// Items inside the macro are converted to Actions as such:
/// - [`Action::KeyCode`]: Idents are automatically understood as keycodes: `A`, `RCtrl`, `Space`
///     - Punctuation, numbers and other literals that aren't special to the rust parser are converted
///       to KeyCodes as well: `,` becomes `KeyCode::Commma`, `2` becomes `KeyCode::Kb2`, `/` becomes `KeyCode::Slash`
///     - Characters which require shifted keys are converted to `Action::MultipleKeyCodes(&[LShift, <character>])`:
///       `!` becomes `Action::MultipleKeyCodes(&[LShift, Kb1])` etc
///     - Characters special to the rust parser (parentheses, brackets, braces, quotes, apostrophes, underscores, backslashes and backticks)
///       left alone cause parsing errors and as such have to be enclosed by apostrophes: `'['` becomes `KeyCode::LBracket`,
///       `'\''` becomes `KeyCode::Quote`, `'\\'` becomes `KeyCode::BSlash`
/// - [`Action::NoOp`]: Lowercase `n`
/// - [`Action::Trans`]: Lowercase `t`
/// - [`Action::Layer`]: A number in parentheses: `(1)`, `(4 - 2)`, `(0x4u8 as usize)`
/// - [`Action::MultipleActions`]: Actions in brackets: `[LCtrl S]`, `[LAlt LCtrl C]`, `[(2) B {Action::NoOp}]`
/// - Other `Action`s: anything in braces (`{}`) is copied unchanged to the final layout - `{ Action::Custom(42) }`
///   simply becomes `Action::Custom(42)`
///
/// **Important note**: comma (`,`) is a keycode on its own, and can't be used to separate keycodes as one would have
/// to do when not using a macro.
///
/// ## Usage example:
/// Example layout for a 12x4 split keyboard:
/// ```
/// use keyberon::action::Action;
/// use keyberon::layout::Layers;
/// static DLAYER: Action = Action::DefaultLayer(5);
///
/// pub static LAYERS: Layers<12, 4, 2> = keyberon::layout::layout! {
///     {
///         [ Tab    Q W E R T   Y U I O P BSpace ]
///         [ LCtrl  A S D F G   H J K L ; Quote  ]
///         [ LShift Z X C V B   N M , . / Escape ]
///         [ n n LGui {DLAYER} Space Escape   BSpace Enter (1) RAlt n n ]
///     }
///     {
///         [ Tab    1 2 3 4 5   6 7 8 9 0 BSpace  ]
///         [ LCtrl  ! @ # $ %   ^ & * '(' ')' -   ]
///         [ LShift n n n n n   n n n n n [LAlt A]]
///         [ n n LGui (2) t t   t t t RAlt n n    ]
///     }
///     // ...
/// };
/// ```
pub use keyberon_macros::*;

use crate::action::{Action, HoldTapAction, HoldTapConfig, SequenceEvent};
use crate::key_code::KeyCode;
use arraydeque::ArrayDeque;
use heapless::Vec;

use State::*;

/// The Layers type.
///
/// `Layers` type is an array of layers which contain the description
/// of actions on the switch matrix. For example `layers[1][2][3]`
/// corresponds to the key on the first layer, row 2, column 3.
/// The generic parameters are in order: the number of columns, rows and layers,
/// and the type contained in custom actions.
pub type Layers<
    const C: usize,
    const R: usize,
    const L: usize,
    T = core::convert::Infallible,
    K = KeyCode,
> = [[[Action<T, K>; C]; R]; L];

/// The current event stack.
///
/// Events can be retrieved by iterating over this struct and calling [Stacked::event].
type Stack = ArrayDeque<[Stacked; 16], arraydeque::behavior::Wrapping>;

// The maximum number of simultaneously-executing Squences:
const MAX_SEQUENCES: usize = 4;

/// The layout manager. It takes `Event`s and `tick`s as input, and
/// generate keyboard reports.
pub struct Layout<
    const C: usize,
    const R: usize,
    const L: usize,
    T = core::convert::Infallible,
    K = KeyCode,
> where
    T: 'static,
    K: 'static + Copy,
{
    layers: &'static [[[Action<T, K>; C]; R]; L],
    default_layer: usize,
    states: Vec<State<T, K>, 64>,
    waiting: Option<WaitingState<T, K>>,
    stacked: Stack,
    tap_hold_tracker: TapHoldTracker,
    active_sequences: ArrayDeque<[SequenceState<K>; MAX_SEQUENCES], arraydeque::behavior::Wrapping>,
}

/// An event on the key matrix.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Event {
    /// Press event with coordinates (i, j).
    Press(u8, u8),
    /// Release event with coordinates (i, j).
    Release(u8, u8),
}
impl Event {
    /// Returns the coordinates (i, j) of the event.
    pub fn coord(self) -> (u8, u8) {
        match self {
            Event::Press(i, j) => (i, j),
            Event::Release(i, j) => (i, j),
        }
    }

    /// Transforms the coordinates of the event.
    ///
    /// # Example
    ///
    /// ```
    /// # use keyberon::layout::Event;
    /// assert_eq!(
    ///     Event::Press(3, 10),
    ///     Event::Press(3, 1).transform(|i, j| (i, 11 - j)),
    /// );
    /// ```
    pub fn transform(self, f: impl FnOnce(u8, u8) -> (u8, u8)) -> Self {
        match self {
            Event::Press(i, j) => {
                let (i, j) = f(i, j);
                Event::Press(i, j)
            }
            Event::Release(i, j) => {
                let (i, j) = f(i, j);
                Event::Release(i, j)
            }
        }
    }

    /// Returns `true` if the event is a key press.
    pub fn is_press(self) -> bool {
        match self {
            Event::Press(..) => true,
            Event::Release(..) => false,
        }
    }

    /// Returns `true` if the event is a key release.
    pub fn is_release(self) -> bool {
        match self {
            Event::Release(..) => true,
            Event::Press(..) => false,
        }
    }
}

/// Event from custom action.
#[derive(Debug, PartialEq, Eq, Default)]
pub enum CustomEvent<T: 'static> {
    /// No custom action.
    #[default]
    NoEvent,
    /// The given custom action key is pressed.
    Press(&'static T),
    /// The given custom action key is released.
    Release(&'static T),
}
impl<T> CustomEvent<T> {
    /// Update an event according to a new event.
    ///
    ///The event can only be modified in the order `NoEvent < Press <
    /// Release`
    fn update(&mut self, e: Self) {
        use CustomEvent::*;
        match (&e, &self) {
            (Release(_), NoEvent) | (Release(_), Press(_)) => *self = e,
            (Press(_), NoEvent) => *self = e,
            _ => (),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum State<T: 'static, K: 'static + Copy> {
    NormalKey { keycode: K, coord: (u8, u8) },
    LayerModifier { value: usize, coord: (u8, u8) },
    Custom { value: &'static T, coord: (u8, u8) },
    FakeKey { keycode: K }, // Fake key event for sequences
}
impl<T: 'static, K: 'static + Copy> Copy for State<T, K> {}
impl<T: 'static, K: 'static + Copy> Clone for State<T, K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: 'static, K: 'static + Copy + Eq> State<T, K> {
    fn keycode(&self) -> Option<K> {
        match self {
            NormalKey { keycode, .. } => Some(*keycode),
            FakeKey { keycode } => Some(*keycode),
            _ => None,
        }
    }
    fn tick(&self) -> Option<Self> {
        Some(*self)
    }
    fn release(&self, c: (u8, u8), custom: &mut CustomEvent<T>) -> Option<Self> {
        match *self {
            NormalKey { coord, .. } | LayerModifier { coord, .. } if coord == c => None,
            Custom { value, coord } if coord == c => {
                custom.update(CustomEvent::Release(value));
                None
            }
            _ => Some(*self),
        }
    }
    fn seq_release(&self, kc: K) -> Option<Self> {
        match *self {
            FakeKey { keycode, .. } if keycode == kc => None,
            _ => Some(*self),
        }
    }
    fn get_layer(&self) -> Option<usize> {
        match self {
            LayerModifier { value, .. } => Some(*value),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct WaitingState<T: 'static, K: 'static> {
    coord: (u8, u8),
    timeout: u16,
    delay: u16,
    hold: &'static Action<T, K>,
    tap: &'static Action<T, K>,
    config: HoldTapConfig,
}

/// Actions that can be triggered for a key configured for HoldTap.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WaitingAction {
    /// Trigger the holding event.
    Hold,
    /// Trigger the tapping event.
    Tap,
    /// Drop this event. It will act as if no key was pressed.
    NoOp,
}

impl<T, K> WaitingState<T, K> {
    fn tick(&mut self, stacked: &Stack) -> Option<WaitingAction> {
        self.timeout = self.timeout.saturating_sub(1);
        match self.config {
            HoldTapConfig::Default => (),
            HoldTapConfig::HoldOnOtherKeyPress => {
                if stacked.iter().any(|s| s.event.is_press()) {
                    return Some(WaitingAction::Hold);
                }
            }
            HoldTapConfig::PermissiveHold => {
                for (x, s) in stacked.iter().enumerate() {
                    if s.event.is_press() {
                        let (i, j) = s.event.coord();
                        let target = Event::Release(i, j);
                        if stacked.iter().skip(x + 1).any(|s| s.event == target) {
                            return Some(WaitingAction::Hold);
                        }
                    }
                }
            }
            HoldTapConfig::Custom(func) => {
                if let waiting_action @ Some(_) = (func)(StackedIter(stacked.iter())) {
                    return waiting_action;
                }
            }
        }
        if let Some(&Stacked { since, .. }) = stacked
            .iter()
            .find(|s| self.is_corresponding_release(&s.event))
        {
            if self.timeout >= self.delay - since {
                Some(WaitingAction::Tap)
            } else {
                Some(WaitingAction::Hold)
            }
        } else if self.timeout == 0 {
            Some(WaitingAction::Hold)
        } else {
            None
        }
    }
    fn is_corresponding_release(&self, event: &Event) -> bool {
        matches!(event, Event::Release(i, j) if (*i, *j) == self.coord)
    }
}

/// An iterator over the currently stacked events.
///
/// Events can be retrieved by iterating over this struct and calling [Stacked::event].
pub struct StackedIter<'a>(arraydeque::Iter<'a, Stacked>);

impl<'a> Iterator for StackedIter<'a> {
    type Item = &'a Stacked;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

#[derive(Debug, Copy, Clone)]
struct SequenceState<K: 'static> {
    cur_event: Option<SequenceEvent<K>>,
    delay: u32,        // Keeps track of SequenceEvent::Delay time remaining
    tapped: Option<K>, // Keycode of a key that should be released at the next tick
    remaining_events: &'static [SequenceEvent<K>],
}

/// An event, waiting in a stack to be processed.
#[derive(Debug)]
pub struct Stacked {
    event: Event,
    since: u16,
}
impl From<Event> for Stacked {
    fn from(event: Event) -> Self {
        Stacked { event, since: 0 }
    }
}
impl Stacked {
    fn tick(&mut self) {
        self.since = self.since.saturating_add(1);
    }

    /// Get the [Event] from this object.
    pub fn event(&self) -> Event {
        self.event
    }
}

#[derive(Default)]
struct TapHoldTracker {
    coord: (u8, u8),
    timeout: u16,
}

impl TapHoldTracker {
    fn tick(&mut self) {
        self.timeout = self.timeout.saturating_sub(1);
    }
}

impl<const C: usize, const R: usize, const L: usize, T: 'static, K: 'static + Copy + Eq>
    Layout<C, R, L, T, K>
{
    /// Creates a new `Layout` object.
    pub fn new(layers: &'static [[[Action<T, K>; C]; R]; L]) -> Self {
        Self {
            layers,
            default_layer: 0,
            states: Vec::new(),
            waiting: None,
            stacked: ArrayDeque::new(),
            tap_hold_tracker: Default::default(),
            active_sequences: ArrayDeque::new(),
        }
    }
    /// Iterates on the key codes of the current state.
    pub fn keycodes(&self) -> impl Iterator<Item = K> + '_ {
        self.states.iter().filter_map(State::keycode)
    }
    fn waiting_into_hold(&mut self) -> CustomEvent<T> {
        if let Some(w) = &self.waiting {
            let hold = w.hold;
            let coord = w.coord;
            self.waiting = None;
            if coord == self.tap_hold_tracker.coord {
                self.tap_hold_tracker.timeout = 0;
            }
            self.do_action(hold, coord, 0)
        } else {
            CustomEvent::NoEvent
        }
    }
    fn waiting_into_tap(&mut self) -> CustomEvent<T> {
        if let Some(w) = &self.waiting {
            let tap = w.tap;
            let coord = w.coord;
            self.waiting = None;
            self.do_action(tap, coord, 0)
        } else {
            CustomEvent::NoEvent
        }
    }
    fn drop_waiting(&mut self) -> CustomEvent<T> {
        self.waiting = None;
        CustomEvent::NoEvent
    }
    /// A time event.
    ///
    /// This method must be called regularly, typically every millisecond.
    ///
    /// Returns the corresponding `CustomEvent`, allowing to manage
    /// custom actions thanks to the `Action::Custom` variant.
    pub fn tick(&mut self) -> CustomEvent<T> {
        self.states = self.states.iter().filter_map(State::tick).collect();
        self.stacked.iter_mut().for_each(Stacked::tick);
        self.tap_hold_tracker.tick();
        self.process_sequences();
        match &mut self.waiting {
            Some(w) => match w.tick(&self.stacked) {
                Some(WaitingAction::Hold) => self.waiting_into_hold(),
                Some(WaitingAction::Tap) => self.waiting_into_tap(),
                Some(WaitingAction::NoOp) => self.drop_waiting(),
                None => CustomEvent::NoEvent,
            },
            None => match self.stacked.pop_front() {
                Some(s) => self.unstack(s),
                None => CustomEvent::NoEvent,
            },
        }
    }
    /// Takes care of draining and populating the `active_sequences` ArrayDeque,
    /// giving us sequences (aka macros) of nearly limitless length!
    fn process_sequences(&mut self) {
        // Iterate over all active sequence events
        for _ in 0..self.active_sequences.len() {
            if let Some(mut seq) = self.active_sequences.pop_front() {
                // If we've encountered a SequenceEvent::Delay we must count
                // that down completely before doing anything else...
                if seq.delay > 0 {
                    seq.delay = seq.delay.saturating_sub(1);
                } else if let Some(keycode) = seq.tapped {
                    // Clear out the Press() matching this Tap()'s keycode
                    self.states = self
                        .states
                        .iter()
                        .filter_map(|s| s.seq_release(keycode))
                        .collect();
                    seq.tapped = None;
                } else {
                    // Pull the next SequenceEvent
                    match seq.remaining_events {
                        [e, tail @ ..] => {
                            seq.cur_event = Some(*e);
                            seq.remaining_events = tail;
                        }
                        [] => (),
                    }
                    // Process it (SequenceEvent)
                    match seq.cur_event {
                        Some(SequenceEvent::Complete) => {
                            for fake_key in self.states.clone().iter() {
                                if let FakeKey { keycode } = *fake_key {
                                    self.states = self
                                        .states
                                        .iter()
                                        .filter_map(|s| s.seq_release(keycode))
                                        .collect();
                                }
                            }
                            seq.remaining_events = &[];
                        }
                        Some(SequenceEvent::Press(keycode)) => {
                            // Start tracking this fake key Press() event
                            let _ = self.states.push(FakeKey { keycode });
                        }
                        Some(SequenceEvent::Tap(keycode)) => {
                            // Same as Press() except we track it for one tick via seq.tapped:
                            let _ = self.states.push(FakeKey { keycode });
                            seq.tapped = Some(keycode);
                        }
                        Some(SequenceEvent::Release(keycode)) => {
                            // Clear out the Press() matching this Release's keycode
                            self.states = self
                                .states
                                .iter()
                                .filter_map(|s| s.seq_release(keycode))
                                .collect()
                        }
                        Some(SequenceEvent::Delay { duration }) => {
                            // Setup a delay that will be decremented once per tick until 0
                            if duration > 0 {
                                // -1 to start since this tick counts
                                seq.delay = duration - 1;
                            }
                        }
                        _ => {} // We'll never get here
                    }
                }
                if !seq.remaining_events.is_empty() {
                    // Put it back
                    self.active_sequences.push_back(seq);
                }
            }
        }
    }
    fn unstack(&mut self, stacked: Stacked) -> CustomEvent<T> {
        use Event::*;
        match stacked.event {
            Release(i, j) => {
                let mut custom = CustomEvent::NoEvent;
                self.states = self
                    .states
                    .iter()
                    .filter_map(|s| s.release((i, j), &mut custom))
                    .collect();
                custom
            }
            Press(i, j) => {
                let action = self.press_as_action((i, j), self.current_layer());
                self.do_action(action, (i, j), stacked.since)
            }
        }
    }
    /// Register a key event.
    pub fn event(&mut self, event: Event) {
        if let Some(stacked) = self.stacked.push_back(event.into()) {
            self.waiting_into_hold();
            self.unstack(stacked);
        }
    }
    fn press_as_action(&self, coord: (u8, u8), layer: usize) -> &'static Action<T, K> {
        use crate::action::Action::*;
        let action = self
            .layers
            .get(layer)
            .and_then(|l| l.get(coord.0 as usize))
            .and_then(|l| l.get(coord.1 as usize));
        match action {
            None => &NoOp,
            Some(Trans) => {
                if layer != self.default_layer {
                    self.press_as_action(coord, self.default_layer)
                } else {
                    &NoOp
                }
            }
            Some(action) => action,
        }
    }
    fn do_action(
        &mut self,
        action: &'static Action<T, K>,
        coord: (u8, u8),
        delay: u16,
    ) -> CustomEvent<T> {
        assert!(self.waiting.is_none());
        use Action::*;
        match action {
            NoOp | Trans => (),
            HoldTap(HoldTapAction {
                timeout,
                hold,
                tap,
                config,
                tap_hold_interval,
            }) => {
                if *tap_hold_interval == 0
                    || coord != self.tap_hold_tracker.coord
                    || self.tap_hold_tracker.timeout == 0
                {
                    let waiting: WaitingState<T, K> = WaitingState {
                        coord,
                        timeout: *timeout,
                        delay,
                        hold,
                        tap,
                        config: *config,
                    };
                    self.waiting = Some(waiting);
                    self.tap_hold_tracker.timeout = *tap_hold_interval;
                } else {
                    self.tap_hold_tracker.timeout = 0;
                    self.do_action(tap, coord, delay);
                }
                // Need to set tap_hold_tracker coord AFTER the checks.
                self.tap_hold_tracker.coord = coord;
            }
            &KeyCode(keycode) => {
                self.tap_hold_tracker.coord = coord;
                let _ = self.states.push(NormalKey { coord, keycode });
            }
            &MultipleKeyCodes(v) => {
                self.tap_hold_tracker.coord = coord;
                for &keycode in *v {
                    let _ = self.states.push(NormalKey { coord, keycode });
                }
            }
            &MultipleActions(v) => {
                self.tap_hold_tracker.coord = coord;
                let mut custom = CustomEvent::NoEvent;
                for action in *v {
                    custom.update(self.do_action(action, coord, delay));
                }
                return custom;
            }
            Sequence(events) => {
                self.active_sequences.push_back(SequenceState {
                    cur_event: None,
                    delay: 0,
                    tapped: None,
                    remaining_events: events,
                });
            }
            &Layer(value) => {
                self.tap_hold_tracker.coord = coord;
                let _ = self.states.push(LayerModifier { value, coord });
            }
            DefaultLayer(value) => {
                self.tap_hold_tracker.coord = coord;
                self.set_default_layer(*value);
            }
            Custom(value) => {
                self.tap_hold_tracker.coord = coord;
                if self.states.push(State::Custom { value, coord }).is_ok() {
                    return CustomEvent::Press(value);
                }
            }
        }
        CustomEvent::NoEvent
    }

    /// Obtain the index of the current active layer
    pub fn current_layer(&self) -> usize {
        self.states
            .iter()
            .rev()
            .find_map(State::get_layer)
            .unwrap_or(self.default_layer)
    }

    /// Sets the default layer for the layout
    pub fn set_default_layer(&mut self, value: usize) {
        if value < self.layers.len() {
            self.default_layer = value
        }
    }
}

#[cfg(test)]
mod test {
    extern crate std;
    use super::{Event::*, Layout, *};
    use crate::action::Action::*;
    use crate::action::HoldTapConfig;
    use crate::action::SequenceEvent;
    use crate::action::{k, l, m};
    use crate::key_code::KeyCode;
    use crate::key_code::KeyCode::*;
    use std::collections::BTreeSet;

    #[track_caller]
    fn assert_keys(expected: &[KeyCode], iter: impl Iterator<Item = KeyCode>) {
        let expected: BTreeSet<_> = expected.iter().copied().collect();
        let tested = iter.collect();
        assert_eq!(expected, tested);
    }

    #[test]
    fn basic_hold_tap() {
        static LAYERS: Layers<2, 1, 2> = [
            [[
                HoldTap(&HoldTapAction {
                    timeout: 200,
                    hold: l(1),
                    tap: k(Space),
                    config: HoldTapConfig::Default,
                    tap_hold_interval: 0,
                }),
                HoldTap(&HoldTapAction {
                    timeout: 200,
                    hold: k(LCtrl),
                    tap: k(Enter),
                    config: HoldTapConfig::Default,
                    tap_hold_interval: 0,
                }),
            ]],
            [[Trans, m(&[LCtrl, Enter].as_slice())]],
        ];
        let mut layout = Layout::new(&LAYERS);
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        layout.event(Press(0, 1));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        layout.event(Press(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        layout.event(Release(0, 0));
        for _ in 0..197 {
            assert_eq!(CustomEvent::NoEvent, layout.tick());
            assert_keys(&[], layout.keycodes());
        }
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LCtrl], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LCtrl], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LCtrl, Space], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LCtrl], layout.keycodes());
        layout.event(Release(0, 1));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
    }

    #[test]
    fn hold_tap_interleaved_timeout() {
        static LAYERS: Layers<2, 1, 1> = [[[
            HoldTap(&HoldTapAction {
                timeout: 200,
                hold: k(LAlt),
                tap: k(Space),
                config: HoldTapConfig::Default,
                tap_hold_interval: 0,
            }),
            HoldTap(&HoldTapAction {
                timeout: 20,
                hold: k(LCtrl),
                tap: k(Enter),
                config: HoldTapConfig::Default,
                tap_hold_interval: 0,
            }),
        ]]];
        let mut layout = Layout::new(&LAYERS);
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        layout.event(Press(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        layout.event(Press(0, 1));
        for _ in 0..15 {
            assert_eq!(CustomEvent::NoEvent, layout.tick());
            assert_keys(&[], layout.keycodes());
        }
        layout.event(Release(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[Space], layout.keycodes());
        for _ in 0..10 {
            assert_eq!(CustomEvent::NoEvent, layout.tick());
            assert_keys(&[Space], layout.keycodes());
        }
        layout.event(Release(0, 1));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[Space, LCtrl], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LCtrl], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
    }

    #[test]
    fn hold_on_press() {
        static LAYERS: Layers<2, 1, 1> = [[[
            HoldTap(&HoldTapAction {
                timeout: 200,
                hold: k(LAlt),
                tap: k(Space),
                config: HoldTapConfig::HoldOnOtherKeyPress,
                tap_hold_interval: 0,
            }),
            k(Enter),
        ]]];
        let mut layout = Layout::new(&LAYERS);

        // Press another key before timeout
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        layout.event(Press(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        layout.event(Press(0, 1));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LAlt], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LAlt, Enter], layout.keycodes());
        layout.event(Release(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[Enter], layout.keycodes());
        layout.event(Release(0, 1));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());

        // Press another key after timeout
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        layout.event(Press(0, 0));
        for _ in 0..200 {
            assert_eq!(CustomEvent::NoEvent, layout.tick());
            assert_keys(&[], layout.keycodes());
        }
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LAlt], layout.keycodes());
        layout.event(Press(0, 1));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LAlt, Enter], layout.keycodes());
        layout.event(Release(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[Enter], layout.keycodes());
        layout.event(Release(0, 1));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
    }

    #[test]
    fn permissive_hold() {
        static LAYERS: Layers<2, 1, 1> = [[[
            HoldTap(&HoldTapAction {
                timeout: 200,
                hold: k(LAlt),
                tap: k(Space),
                config: HoldTapConfig::PermissiveHold,
                tap_hold_interval: 0,
            }),
            k(Enter),
        ]]];
        let mut layout = Layout::new(&LAYERS);

        // Press and release another key before timeout
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        layout.event(Press(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        layout.event(Press(0, 1));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        layout.event(Release(0, 1));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LAlt], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LAlt, Enter], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LAlt], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LAlt], layout.keycodes());
        layout.event(Release(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
    }

    #[test]
    fn multiple_actions() {
        static LAYERS: Layers<2, 1, 2> = [
            [[MultipleActions(&[l(1), k(LShift)].as_slice()), k(F)]],
            [[Trans, k(E)]],
        ];
        let mut layout = Layout::new(&LAYERS);
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        layout.event(Press(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift], layout.keycodes());
        layout.event(Press(0, 1));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift, E], layout.keycodes());
        layout.event(Release(0, 1));
        layout.event(Release(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
    }

    #[test]
    fn custom() {
        static LAYERS: Layers<1, 1, 1, u8> = [[[Action::Custom(42)]]];
        let mut layout = Layout::new(&LAYERS);
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());

        // Custom event
        layout.event(Press(0, 0));
        assert_eq!(CustomEvent::Press(&42), layout.tick());
        assert_keys(&[], layout.keycodes());

        // nothing more
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());

        // release custom
        layout.event(Release(0, 0));
        assert_eq!(CustomEvent::Release(&42), layout.tick());
        assert_keys(&[], layout.keycodes());
    }

    #[test]
    fn multiple_layers() {
        static LAYERS: Layers<2, 1, 4> = [
            [[l(1), l(2)]],
            [[k(A), l(3)]],
            [[l(0), k(B)]],
            [[k(C), k(D)]],
        ];
        let mut layout = Layout::new(&LAYERS);
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_eq!(0, layout.current_layer());
        assert_keys(&[], layout.keycodes());

        // press L1
        layout.event(Press(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_eq!(1, layout.current_layer());
        assert_keys(&[], layout.keycodes());
        // press L3 on L1
        layout.event(Press(0, 1));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_eq!(3, layout.current_layer());
        assert_keys(&[], layout.keycodes());
        // release L1, still on l3
        layout.event(Release(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_eq!(3, layout.current_layer());
        assert_keys(&[], layout.keycodes());
        // press and release C on L3
        layout.event(Press(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[C], layout.keycodes());
        layout.event(Release(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        // release L3, back to L0
        layout.event(Release(0, 1));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_eq!(0, layout.current_layer());
        assert_keys(&[], layout.keycodes());

        // back to empty, going to L2
        layout.event(Press(0, 1));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_eq!(2, layout.current_layer());
        assert_keys(&[], layout.keycodes());
        // and press the L0 key on L2
        layout.event(Press(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_eq!(0, layout.current_layer());
        assert_keys(&[], layout.keycodes());
        // release the L0, back to L2
        layout.event(Release(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_eq!(2, layout.current_layer());
        assert_keys(&[], layout.keycodes());
        // release the L2, back to L0
        layout.event(Release(0, 1));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_eq!(0, layout.current_layer());
        assert_keys(&[], layout.keycodes());
    }

    #[test]
    fn custom_handler() {
        fn always_tap(_: StackedIter) -> Option<WaitingAction> {
            Some(WaitingAction::Tap)
        }
        fn always_hold(_: StackedIter) -> Option<WaitingAction> {
            Some(WaitingAction::Hold)
        }
        fn always_nop(_: StackedIter) -> Option<WaitingAction> {
            Some(WaitingAction::NoOp)
        }
        fn always_none(_: StackedIter) -> Option<WaitingAction> {
            None
        }
        static LAYERS: Layers<4, 1, 1> = [[[
            HoldTap(&HoldTapAction {
                timeout: 200,
                hold: k(Kb1),
                tap: k(Kb0),
                config: HoldTapConfig::Custom(always_tap),
                tap_hold_interval: 0,
            }),
            HoldTap(&HoldTapAction {
                timeout: 200,
                hold: k(Kb3),
                tap: k(Kb2),
                config: HoldTapConfig::Custom(always_hold),
                tap_hold_interval: 0,
            }),
            HoldTap(&HoldTapAction {
                timeout: 200,
                hold: k(Kb5),
                tap: k(Kb4),
                config: HoldTapConfig::Custom(always_nop),
                tap_hold_interval: 0,
            }),
            HoldTap(&HoldTapAction {
                timeout: 200,
                hold: k(Kb7),
                tap: k(Kb6),
                config: HoldTapConfig::Custom(always_none),
                tap_hold_interval: 0,
            }),
        ]]];
        let mut layout = Layout::new(&LAYERS);
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());

        // Custom handler always taps
        layout.event(Press(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[Kb0], layout.keycodes());

        // nothing more
        layout.event(Release(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());

        // Custom handler always holds
        layout.event(Press(0, 1));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[Kb3], layout.keycodes());

        // nothing more
        layout.event(Release(0, 1));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());

        // Custom handler always prevents any event
        layout.event(Press(0, 2));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());

        // even timeout does not trigger
        for _ in 0..200 {
            assert_eq!(CustomEvent::NoEvent, layout.tick());
            assert_keys(&[], layout.keycodes());
        }

        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());

        // nothing more
        layout.event(Release(0, 2));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());

        // Custom handler timeout fallback
        layout.event(Press(0, 3));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());

        for _ in 0..199 {
            assert_eq!(CustomEvent::NoEvent, layout.tick());
            assert_keys(&[], layout.keycodes());
        }

        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[Kb7], layout.keycodes());
    }

    #[test]
    fn tap_hold_interval() {
        static LAYERS: Layers<2, 1, 1> = [[[
            HoldTap(&HoldTapAction {
                timeout: 200,
                hold: k(LAlt),
                tap: k(Space),
                config: HoldTapConfig::Default,
                tap_hold_interval: 200,
            }),
            k(Enter),
        ]]];
        let mut layout = Layout::new(&LAYERS);

        // press and release the HT key, expect tap action
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        layout.event(Press(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        layout.event(Release(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[Space], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());

        // press again within tap_hold_interval, tap action should be in keycode immediately
        layout.event(Press(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[Space], layout.keycodes());

        // tap action should continue to be in keycodes even after timeout
        for _ in 0..300 {
            assert_eq!(CustomEvent::NoEvent, layout.tick());
            assert_keys(&[Space], layout.keycodes());
        }
        layout.event(Release(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());

        // Press again. This is outside the tap_hold_interval window, so should result in hold
        // action.
        layout.event(Press(0, 0));
        for _ in 0..200 {
            assert_eq!(CustomEvent::NoEvent, layout.tick());
            assert_keys(&[], layout.keycodes());
        }
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LAlt], layout.keycodes());
        layout.event(Release(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
    }

    #[test]
    fn tap_hold_interval_interleave() {
        static LAYERS: Layers<3, 1, 1> = [[[
            HoldTap(&HoldTapAction {
                timeout: 200,
                hold: k(LAlt),
                tap: k(Space),
                config: HoldTapConfig::Default,
                tap_hold_interval: 200,
            }),
            k(Enter),
            HoldTap(&HoldTapAction {
                timeout: 200,
                hold: k(LAlt),
                tap: k(Enter),
                config: HoldTapConfig::Default,
                tap_hold_interval: 200,
            }),
        ]]];
        let mut layout = Layout::new(&LAYERS);

        // press and release the HT key, expect tap action
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        layout.event(Press(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        layout.event(Release(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[Space], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());

        // press a different key in between
        layout.event(Press(0, 1));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[Enter], layout.keycodes());
        layout.event(Release(0, 1));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());

        // press HT key again, should result in hold action
        layout.event(Press(0, 0));
        for _ in 0..200 {
            assert_eq!(CustomEvent::NoEvent, layout.tick());
            assert_keys(&[], layout.keycodes());
        }
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LAlt], layout.keycodes());
        layout.event(Release(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());

        // press HT key, press+release diff key, release HT key
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        layout.event(Press(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        layout.event(Press(0, 1));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        layout.event(Release(0, 1));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        layout.event(Release(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[Space], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[Enter, Space], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[Space], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());

        // press HT key again, should result in hold action
        layout.event(Press(0, 0));
        for _ in 0..200 {
            assert_eq!(CustomEvent::NoEvent, layout.tick());
            assert_keys(&[], layout.keycodes());
        }
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LAlt], layout.keycodes());
        layout.event(Release(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());

        // press HT key, press+release diff (HT) key, release HT key
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        layout.event(Press(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        layout.event(Press(0, 2));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        layout.event(Release(0, 2));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        layout.event(Release(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[Space], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[Space], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[Enter, Space], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[Space], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());

        // press HT key again, should result in hold action
        layout.event(Press(0, 0));
        for _ in 0..200 {
            assert_eq!(CustomEvent::NoEvent, layout.tick());
            assert_keys(&[], layout.keycodes());
        }
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LAlt], layout.keycodes());
    }

    #[test]
    fn tap_hold_interval_short_hold() {
        static LAYERS: Layers<1, 1, 1> = [[[HoldTap(&HoldTapAction {
            timeout: 50,
            hold: k(LAlt),
            tap: k(Space),
            config: HoldTapConfig::Default,
            tap_hold_interval: 200,
        })]]];
        let mut layout = Layout::new(&LAYERS);

        // press and hold the HT key, expect hold action
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        layout.event(Press(0, 0));
        for _ in 0..50 {
            assert_eq!(CustomEvent::NoEvent, layout.tick());
            assert_keys(&[], layout.keycodes());
        }
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LAlt], layout.keycodes());
        layout.event(Release(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());

        // press and hold the HT key, expect hold action, even though it's within the
        // tap_hold_interval
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        layout.event(Press(0, 0));
        for _ in 0..50 {
            assert_eq!(CustomEvent::NoEvent, layout.tick());
            assert_keys(&[], layout.keycodes());
        }
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LAlt], layout.keycodes());
        layout.event(Release(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
    }

    #[test]
    fn tap_hold_interval_different_hold() {
        static LAYERS: Layers<2, 1, 1> = [[[
            HoldTap(&HoldTapAction {
                timeout: 50,
                hold: k(LAlt),
                tap: k(Space),
                config: HoldTapConfig::Default,
                tap_hold_interval: 200,
            }),
            HoldTap(&HoldTapAction {
                timeout: 200,
                hold: k(RAlt),
                tap: k(Enter),
                config: HoldTapConfig::Default,
                tap_hold_interval: 200,
            }),
        ]]];
        let mut layout = Layout::new(&LAYERS);

        // press HT1, press HT2, release HT1 after hold timeout, release HT2, press HT2
        layout.event(Press(0, 0));
        layout.event(Press(0, 1));
        for _ in 0..50 {
            assert_eq!(CustomEvent::NoEvent, layout.tick());
            assert_keys(&[], layout.keycodes());
        }
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LAlt], layout.keycodes());
        layout.event(Release(0, 0));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LAlt], layout.keycodes());
        layout.event(Release(0, 1));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LAlt, Enter], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[Enter], layout.keycodes());
        // press HT2 again, should result in tap action
        layout.event(Press(0, 1));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());

        for _ in 0..300 {
            assert_eq!(CustomEvent::NoEvent, layout.tick());
            assert_keys(&[Enter], layout.keycodes());
        }
    }

    #[test]
    fn sequences() {
        static LAYERS: Layers<4, 1, 1> = [[[
            Sequence(
                // Simple Ctrl-C sequence/macro
                &[
                    SequenceEvent::Press(LCtrl),
                    SequenceEvent::Press(C),
                    SequenceEvent::Release(C),
                    SequenceEvent::Release(LCtrl),
                ]
                .as_slice(),
            ),
            Sequence(
                // So we can test that Complete works
                &[
                    SequenceEvent::Press(LCtrl),
                    SequenceEvent::Press(C),
                    SequenceEvent::Complete,
                ]
                .as_slice(),
            ),
            Sequence(
                // YO with a delay in the middle
                &[
                    SequenceEvent::Press(Y),
                    SequenceEvent::Release(Y),
                    // "How many licks does it take to get to the center?"
                    SequenceEvent::Delay { duration: 3 }, // Let's find out
                    SequenceEvent::Press(O),
                    SequenceEvent::Release(O),
                ]
                .as_slice(),
            ),
            Sequence(
                // A long sequence to test the chunking capability
                &[
                    SequenceEvent::Press(LShift), // Important: Shift must remain held
                    SequenceEvent::Press(U),      // ...or the message just isn't the same!
                    SequenceEvent::Release(U),
                    SequenceEvent::Press(N),
                    SequenceEvent::Release(N),
                    SequenceEvent::Press(L),
                    SequenceEvent::Release(L),
                    SequenceEvent::Press(I),
                    SequenceEvent::Release(I),
                    SequenceEvent::Press(M),
                    SequenceEvent::Release(M),
                    SequenceEvent::Press(I),
                    SequenceEvent::Release(I),
                    SequenceEvent::Press(T),
                    SequenceEvent::Release(T),
                    SequenceEvent::Press(E),
                    SequenceEvent::Release(E),
                    SequenceEvent::Press(D),
                    SequenceEvent::Release(D),
                    SequenceEvent::Press(Space),
                    SequenceEvent::Release(Space),
                    SequenceEvent::Press(P),
                    SequenceEvent::Release(P),
                    SequenceEvent::Press(O),
                    SequenceEvent::Release(O),
                    SequenceEvent::Press(W),
                    SequenceEvent::Release(W),
                    SequenceEvent::Press(E),
                    SequenceEvent::Release(E),
                    SequenceEvent::Press(R),
                    SequenceEvent::Release(R),
                    SequenceEvent::Press(Kb1),
                    SequenceEvent::Release(Kb1),
                    SequenceEvent::Press(Kb1),
                    SequenceEvent::Release(Kb1),
                    SequenceEvent::Press(Kb1),
                    SequenceEvent::Release(Kb1),
                    SequenceEvent::Press(Kb1),
                    SequenceEvent::Release(Kb1),
                    SequenceEvent::Release(LShift),
                ]
                .as_slice(),
            ),
        ]]];
        let mut layout = Layout::new(&LAYERS);
        // Test a basic sequence
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        layout.event(Press(0, 0));
        // Sequences take an extra tick to kickoff since the first tick starts the sequence:
        assert_eq!(CustomEvent::NoEvent, layout.tick()); // Sequence detected & added
        assert_eq!(CustomEvent::NoEvent, layout.tick()); // Sequence starts
        assert_keys(&[LCtrl], layout.keycodes()); // First item in the SequenceEvent
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LCtrl, C], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LCtrl], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        // Test the use of Complete()
        assert_keys(&[], layout.keycodes());
        layout.event(Press(0, 1));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LCtrl], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LCtrl, C], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        // Test a sequence with a Delay() (aka The Mr Owl test; duration == 3)
        layout.event(Press(0, 2));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[Y], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick()); // First decrement (2)
        assert_keys(&[], layout.keycodes()); // "Eh Ooone!"
        assert_eq!(CustomEvent::NoEvent, layout.tick()); // Second decrement (1)
        assert_keys(&[], layout.keycodes()); // "Eh two!"
        assert_eq!(CustomEvent::NoEvent, layout.tick()); // Final decrement (0)
        assert_keys(&[], layout.keycodes()); // "Eh three."
        assert_eq!(CustomEvent::NoEvent, layout.tick()); // Press() added for the next tick()
        assert_eq!(CustomEvent::NoEvent, layout.tick()); // FakeKey Press()
        assert_keys(&[O], layout.keycodes()); // CHOMP!
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[], layout.keycodes());
        // // Test really long sequences (aka macros)...
        layout.event(Press(0, 3));
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift, U], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift, N], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift, L], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift, I], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift, M], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift, I], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift, T], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift, E], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift, D], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift, Space], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift, P], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift, O], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift, W], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift, E], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift, R], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift, Kb1], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift, Kb1], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift, Kb1], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift, Kb1], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
        assert_keys(&[LShift], layout.keycodes());
        assert_eq!(CustomEvent::NoEvent, layout.tick());
    }
}
