//! Layout management.

use crate::action::{Action, SequenceEvent};
use crate::key_code::KeyCode;
use arraydeque::ArrayDeque;
use heapless::consts::U64;
use heapless::Vec;

use State::*;

/// The Layers type.
///
/// The first level correspond to the layer, the two others to the
/// switch matrix.  For example, `layers[1][2][3]` correspond to the
/// key i=2, j=3 on the layer 1.
pub type Layers = &'static [&'static [&'static [Action]]];

/// The layout manager. It takes `Event`s and `tick`s as input, and
/// generate keyboard reports.
pub struct Layout {
    layers: Layers,
    default_layer: usize,
    states: Vec<State, U64>,
    waiting: Option<WaitingState>,
    stacked: ArrayDeque<[Stacked; 16], arraydeque::behavior::Wrapping>,
    sequenced: ArrayDeque<[SequenceEvent; 32], arraydeque::behavior::Wrapping>,
    // Riskable NOTE: Wish we didn't have to preallocate sequenced like this.
    //       I want to be able to have my keyboard type long sentences/quotes!
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
}

/// The various states used internally by Keyberon
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum State {
    /// Normal keys
    NormalKey {
        /// Keycode
        keycode: KeyCode,
        /// Coordinates in the matrix
        coord: (u8, u8),
    },
    /// Layer modifier keys
    LayerModifier {
        /// Value
        value: usize,
        /// Coordinates of this layer modifier key in the matrix
        coord: (u8, u8),
    },
    /// Fake key event for sequences
    FakeKey {
        /// The key to everything!
        keycode: KeyCode,
    },
}
impl State {
    fn keycode(&self) -> Option<KeyCode> {
        match self {
            NormalKey { keycode, .. } => Some(*keycode),
            FakeKey { keycode } => Some(*keycode),
            _ => None,
        }
    }
    fn tick(&self) -> Option<Self> {
        match *self {
            _ => Some(*self),
        }
    }
    fn release(&self, c: (u8, u8)) -> Option<Self> {
        match *self {
            NormalKey { coord, .. } | LayerModifier { coord, .. } if coord == c => None,
            _ => Some(*self),
        }
    }
    fn seq_release(&self, kc: KeyCode) -> Option<Self> {
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

#[derive(Debug, Copy, Clone)]
struct WaitingState {
    coord: (u8, u8),
    timeout: u16,
    hold: &'static Action,
    tap: &'static Action,
}
impl WaitingState {
    fn tick(&mut self) -> bool {
        self.timeout = self.timeout.saturating_sub(1);
        self.timeout == 0
    }
    fn is_corresponding_release(&self, event: &Event) -> bool {
        match event {
            Event::Release(i, j) if (*i, *j) == self.coord => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
struct Stacked {
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
}

impl Layout {
    /// Creates a new `Layout` object.
    pub fn new(layers: Layers) -> Self {
        Self {
            layers,
            default_layer: 0,
            states: Vec::new(),
            waiting: None,
            stacked: ArrayDeque::new(),
            sequenced: ArrayDeque::new(),
        }
    }
    /// Iterates on the key codes of the current state.
    pub fn keycodes<'a>(&'a self) -> impl Iterator<Item = KeyCode> + 'a {
        self.states.iter().filter_map(State::keycode)
    }
    fn waiting_into_hold(&mut self) {
        if let Some(w) = &self.waiting {
            let hold = w.hold;
            let coord = w.coord;
            self.waiting = None;
            self.do_action(hold, coord, 0);
        }
    }
    fn waiting_into_tap(&mut self) {
        if let Some(w) = &self.waiting {
            let tap = w.tap;
            let coord = w.coord;
            self.waiting = None;
            self.do_action(tap, coord, 0);
        }
    }
    /// A time event.
    ///
    /// This method must be called regularly, typically every millisecond.
    ///
    /// Returns an iterator on the current key code state.
    pub fn tick<'a>(&'a mut self) -> impl Iterator<Item = KeyCode> + 'a {
        self.states = self.states.iter().filter_map(State::tick).collect();
        self.stacked.iter_mut().for_each(Stacked::tick);
        match &mut self.waiting {
            Some(w) => {
                if w.tick() {
                    self.waiting_into_hold();
                }
            }
            None => {
                if let Some(s) = self.stacked.pop_front() {
                    self.unstack(s);
                }
            }
        }
        // Process sequences
        if let Some(event) = self.sequenced.pop_front() {
            match event {
                SequenceEvent::Press(keycode) => {
                    // Start tracking this fake key Press() event
                    let _ = self.states.push(FakeKey { keycode: keycode });
                }
                SequenceEvent::Release(keycode) => {
                    // Clear out the Press() matching this Release's keycode
                    self.states = self
                        .states
                        .iter()
                        .filter_map(|s| s.seq_release(keycode))
                        .collect()
                }
                SequenceEvent::Delay { since, ticks } => {
                    if since < ticks {
                        // Increment and put it back
                        self.sequenced.push_front(SequenceEvent::Delay {
                            since: since.saturating_add(1),
                            ticks: ticks,
                        });
                    }
                }
            }
        }
        self.keycodes()
    }
    fn unstack(&mut self, stacked: Stacked) {
        use Event::*;
        match stacked.event {
            Release(i, j) => {
                self.states = self
                    .states
                    .iter()
                    .filter_map(|s| s.release((i, j)))
                    .collect()
            }
            Press(i, j) => {
                let action = self.press_as_action((i, j), self.current_layer());
                self.do_action(action, (i, j), stacked.since);
            }
        }
    }
    /// A key event.
    ///
    /// Returns an iterator on the current key code state.
    pub fn event<'a>(&'a mut self, event: Event) -> impl Iterator<Item = KeyCode> + 'a {
        if let Some(stacked) = self.stacked.push_back(event.into()) {
            self.waiting_into_hold();
            self.unstack(stacked);
        }
        if self
            .waiting
            .as_ref()
            .map(|w| w.is_corresponding_release(&event))
            .unwrap_or(false)
        {
            self.waiting_into_tap();
        }
        self.keycodes()
    }
    fn press_as_action(&self, coord: (u8, u8), layer: usize) -> &'static Action {
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
    fn do_action(&mut self, action: &Action, coord: (u8, u8), delay: u16) {
        assert!(self.waiting.is_none());
        use Action::*;
        match *action {
            NoOp | Trans => (),
            HoldTap { timeout, hold, tap } => {
                let waiting = WaitingState {
                    coord,
                    timeout: timeout.saturating_sub(delay),
                    hold,
                    tap,
                };
                self.waiting = Some(waiting);
                if let Some(Stacked { since, .. }) = self
                    .stacked
                    .iter()
                    .find(|s| waiting.is_corresponding_release(&s.event))
                {
                    if timeout >= delay - since {
                        self.waiting_into_tap();
                    } else {
                        self.waiting_into_hold();
                    }
                }
            }
            KeyCode(keycode) => {
                let _ = self.states.push(NormalKey { coord, keycode });
            }
            MultipleKeyCodes(v) => {
                for &keycode in v {
                    let _ = self.states.push(NormalKey { coord, keycode });
                }
            }
            MultipleActions(v) => {
                for action in v {
                    self.do_action(action, coord, delay);
                }
            }
            Sequence { events } => {
                // Copy the contents of the sequence events into the sequenced ArrayDeque
                for key_event in events {
                    match *key_event {
                        SequenceEvent::Press(keycode) => {
                            self.sequenced.push_back(SequenceEvent::Press(keycode));
                        }
                        SequenceEvent::Release(keycode) => {
                            self.sequenced.push_back(SequenceEvent::Release(keycode));
                        }
                        SequenceEvent::Delay { since, ticks } => {
                            self.sequenced.push_back(SequenceEvent::Delay {
                                since: since,
                                ticks: ticks,
                            });
                        }
                    }
                }
            }
            Layer(value) => {
                let _ = self.states.push(LayerModifier { value, coord });
            }
            DefaultLayer(value) => {
                if value < self.layers.len() {
                    self.default_layer = value
                }
            }
        }
    }
    fn current_layer(&self) -> usize {
        let mut iter = self.states.iter().filter_map(State::get_layer);
        let mut layer = match iter.next() {
            None => self.default_layer,
            Some(l) => l,
        };
        for l in iter {
            layer += l;
        }
        layer
    }
}

#[cfg(test)]
mod test {
    extern crate std;
    use super::{Event::*, Layers, Layout};
    use crate::action::Action::*;
    use crate::action::{k, l, m};
    use crate::key_code::KeyCode;
    use crate::key_code::KeyCode::*;
    use std::collections::BTreeSet;

    //#[track_caller]
    fn assert_keys(expected: &[KeyCode], iter: impl Iterator<Item = KeyCode>) {
        let expected: BTreeSet<_> = expected.iter().copied().collect();
        let tested = iter.collect();
        assert_eq!(expected, tested);
    }

    #[test]
    fn test() {
        static LAYERS: Layers = &[
            &[&[
                HoldTap {
                    timeout: 200,
                    hold: &l(1),
                    tap: &k(Space),
                },
                HoldTap {
                    timeout: 200,
                    hold: &k(LCtrl),
                    tap: &k(Enter),
                },
            ]],
            &[&[Trans, m(&[LCtrl, Enter])]],
        ];
        let mut layout = Layout::new(LAYERS);
        assert_keys(&[], layout.tick());
        assert_keys(&[], layout.event(Press(0, 1)));
        assert_keys(&[], layout.tick());
        assert_keys(&[], layout.event(Press(0, 0)));
        assert_keys(&[], layout.tick());
        assert_keys(&[], layout.event(Release(0, 0)));
        for _ in 0..197 {
            assert_keys(&[], layout.tick());
        }
        assert_keys(&[LCtrl], layout.tick());
        assert_keys(&[LCtrl, Space], layout.tick());
        assert_keys(&[LCtrl], layout.tick());
        assert_keys(&[LCtrl], layout.event(Release(0, 1)));
        assert_keys(&[], layout.tick());
    }

    #[test]
    fn multiple_actions() {
        static LAYERS: Layers = &[
            &[&[MultipleActions(&[l(1), k(LShift)]), k(F)]],
            &[&[Trans, k(E)]],
        ];
        let mut layout = Layout::new(LAYERS);
        assert_keys(&[], layout.tick());
        assert_keys(&[], layout.event(Press(0, 0)));
        assert_keys(&[LShift], layout.tick());
        assert_keys(&[LShift], layout.event(Press(0, 1)));
        assert_keys(&[LShift, E], layout.tick());
        assert_keys(&[LShift, E], layout.event(Release(0, 1)));
        assert_keys(&[LShift, E], layout.event(Release(0, 0)));
        assert_keys(&[LShift], layout.tick());
        assert_keys(&[], layout.tick());
    }
}
