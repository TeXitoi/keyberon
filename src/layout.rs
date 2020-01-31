use crate::action::Action;
use crate::key_code::KeyCode;
use arraydeque::ArrayDeque;
use heapless::consts::U64;
use heapless::Vec;

use State::*;
use WaitingState::*;

pub type Layers = &'static [&'static [&'static [Action]]];

pub struct Layout {
    layers: Layers,
    default_layer: usize,
    states: Vec<State, U64>,
    waiting: Option<WaitingState>,
    stacked: ArrayDeque<[Stacked; 16], arraydeque::behavior::Wrapping>,
}

#[derive(Debug)]
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
enum WaitingState {
    HoldTap {
        coord: (usize, usize),
        timeout: u16,
        hold: KeyCode,
        tap: KeyCode,
    },
    LayerTap {
        coord: (usize, usize),
        timeout: u16,
        layer: usize,
        tap: KeyCode,
    },
}
impl WaitingState {
    fn tick(&mut self) -> bool {
        match self {
            HoldTap { timeout, .. } | LayerTap { timeout, .. } => {
                *timeout = timeout.saturating_sub(1);
                *timeout == 0
            }
        }
    }
    fn as_hold(&self) -> State {
        match self {
            HoldTap { hold, coord, .. } => NormalKey {
                keycode: *hold,
                coord: *coord,
            },
            LayerTap { layer, coord, .. } => LayerModifier {
                value: *layer,
                coord: *coord,
            },
        }
    }
    fn as_tap(&self) -> State {
        match self {
            HoldTap { tap, coord, .. } | LayerTap { tap, coord, .. } => NormalKey {
                keycode: *tap,
                coord: *coord,
            },
        }
    }
    fn coord(&self) -> (usize, usize) {
        match self {
            HoldTap { coord, .. } | LayerTap { coord, .. } => *coord,
        }
    }
    fn is_corresponding_release(&self, event: &Event) -> bool {
        match event {
            Event::Release(x, y) if (*x, *y) == self.coord() => true,
            _ => false,
        }
    }
    fn try_from_action(action: &Action, coord: (usize, usize), since: u16) -> Option<Self> {
        let timeout = 200u16.saturating_sub(since);
        match *action {
            Action::HoldTap(hold, tap) => Some(WaitingState::HoldTap {
                coord,
                timeout,
                hold,
                tap,
            }),
            Action::LayerTap(layer, tap) => Some(WaitingState::LayerTap {
                coord,
                timeout,
                layer,
                tap,
            }),
            _ => None,
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
        if let Some(w) = &mut self.waiting {
            let _ = self.states.push(w.as_hold());
            self.waiting = None;
        }
    }
    fn waiting_into_tap(&mut self) {
        if let Some(w) = &mut self.waiting {
            let _ = self.states.push(w.as_tap());
            self.waiting = None;
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
                let _ = self.stacked.pop_front().map(|s| self.unstack(s));
            }
        }
        self.keycodes()
    }
    fn unstack(&mut self, stacked: Stacked) {
        assert!(self.waiting.is_none());
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
                match WaitingState::try_from_action(action, (x, y), stacked.since) {
                    None => self.do_action(action, (x, y)),
                    Some(w) => {
                        self.waiting = Some(w);
                        if let Some(Stacked { since, .. }) = self
                            .stacked
                            .iter()
                            .find(|s| w.is_corresponding_release(&s.event))
                        {
                            if 200 >= stacked.since - since {
                                self.waiting_into_tap();
                            } else {
                                self.waiting_into_hold();
                            }
                        }
                    }
                }
            }
        }
    }
    pub fn event<'a>(&'a mut self, event: Event) -> impl Iterator<Item = KeyCode> + 'a {
        if self
            .waiting
            .as_ref()
            .map(|w| w.is_corresponding_release(&event))
            .unwrap_or(false)
        {
            self.waiting_into_tap();
        }
        if let Some(stacked) = self.stacked.push_back(event.into()) {
            self.waiting_into_hold();
            self.unstack(stacked);
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
            None => &No,
            Some(Trans) => {
                if layer != self.default_layer {
                    self.press_as_action(coord, self.default_layer)
                } else {
                    &No
                }
            }
            Some(action) => action,
        }
    }
    fn do_action(&mut self, action: &Action, coord: (usize, usize)) {
        use Action::*;
        match *action {
            No | Trans => (),
            KeyCode(keycode) | HoldTap(keycode, _) => {
                let _ = self.states.push(NormalKey { coord, keycode });
            }
            MultipleKeyCodes(v) => {
                for &keycode in v {
                    let _ = self.states.push(NormalKey { coord, keycode });
                }
            }
            Layer(value) | LayerTap(value, _) => {
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
    use crate::action::m;
    use crate::action::Action::*;
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
            &[&[LayerTap(1, Space), HoldTap(LCtrl, Enter)]],
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
