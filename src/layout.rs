use crate::action::Action;
use crate::key_code::KeyCode;
use arraydeque::ArrayDeque;
use heapless::consts::U64;
use heapless::Vec;

use State::*;

pub type Layers = &'static [&'static [&'static [Action]]];

pub struct Layout {
    layers: Layers,
    default_layer: usize,
    states: Vec<State, U64>,
    waiting: Option<WaitingState>,
    stacked: ArrayDeque<[Stacked; 16], arraydeque::behavior::Wrapping>,
}

#[derive(Debug, Copy, Clone)]
pub enum Event {
    Press(usize, usize),
    Release(usize, usize),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum State {
    NormalKey {
        keycode: KeyCode,
        coord: (usize, usize),
    },
    LayerModifier {
        value: usize,
        coord: (usize, usize),
    },
}
impl State {
    fn keycode(&self) -> Option<KeyCode> {
        match self {
            NormalKey { keycode, .. } => Some(*keycode),
            _ => None,
        }
    }
    fn tick(&self) -> Option<Self> {
        match *self {
            _ => Some(*self),
        }
    }
    fn release(&self, c: (usize, usize)) -> Option<Self> {
        match *self {
            NormalKey { coord, .. } | LayerModifier { coord, .. } if coord == c => None,
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
    coord: (usize, usize),
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
            Event::Release(x, y) if (*x, *y) == self.coord => true,
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
    pub fn new(layers: Layers) -> Self {
        Self {
            layers,
            default_layer: 0,
            states: Vec::new(),
            waiting: None,
            stacked: ArrayDeque::new(),
        }
    }
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
        self.keycodes()
    }
    fn unstack(&mut self, stacked: Stacked) {
        use Event::*;
        match stacked.event {
            Release(x, y) => {
                self.states = self
                    .states
                    .iter()
                    .filter_map(|s| s.release((x, y)))
                    .collect()
            }
            Press(x, y) => {
                let action = self.press_as_action((x, y), self.current_layer());
                self.do_action(action, (x, y), stacked.since);
            }
        }
    }
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
    fn press_as_action(&self, coord: (usize, usize), layer: usize) -> &'static Action {
        use crate::action::Action::*;
        let action = self
            .layers
            .get(layer)
            .and_then(|l| l.get(coord.0))
            .and_then(|l| l.get(coord.1));
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
    fn do_action(&mut self, action: &Action, coord: (usize, usize), delay: u16) {
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
}
