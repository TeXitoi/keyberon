use crate::action::Action;
use crate::key_code::KeyCode;
use heapless::consts::U64;
use heapless::Vec;

use Event::*;
use State::*;

pub type Layers = &'static [&'static [&'static [Action]]];

pub struct Layout {
    layers: Layers,
    default_layer: usize,
    current_layer: usize,
    states: Vec<State, U64>,
}

pub enum Event {
    Press(usize, usize),
    Release(usize, usize),
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum State {
    NormalKey {
        keycode: KeyCode,
        coord: (usize, usize),
    },
    LayerModifier {
        value: usize,
        coord: (usize, usize),
    },
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
    PhantomKey {
        keycode: KeyCode,
    },
}
impl State {
    fn keycode(&self) -> Option<KeyCode> {
        match self {
            NormalKey { keycode, .. } | PhantomKey { keycode } => Some(*keycode),
            _ => None,
        }
    }
    fn tick(&self) -> Option<Self> {
        match *self {
            PhantomKey { .. } => None,
            HoldTap {
                timeout,
                hold,
                tap,
                coord,
            } => {
                if timeout > 0 {
                    Some(HoldTap {
                        timeout: timeout - 1,
                        hold,
                        tap,
                        coord,
                    })
                } else {
                    Some(NormalKey {
                        coord,
                        keycode: hold,
                    })
                }
            }
            LayerTap {
                timeout,
                layer,
                tap,
                coord,
            } => {
                if timeout > 0 {
                    Some(LayerTap {
                        timeout: timeout - 1,
                        layer,
                        tap,
                        coord,
                    })
                } else {
                    Some(LayerModifier {
                        coord,
                        value: layer,
                    })
                }
            }
            _ => Some(*self),
        }
    }
    fn release(&self, c: (usize, usize)) -> Option<Self> {
        match *self {
            NormalKey { coord, .. } | LayerModifier { coord, .. } if coord == c => None,
            HoldTap { coord, tap, .. } if coord == c => Some(PhantomKey { keycode: tap }),
            LayerTap { coord, tap, .. } if coord == c => Some(PhantomKey { keycode: tap }),
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

impl Layout {
    pub fn new(layers: Layers) -> Self {
        Self {
            layers,
            default_layer: 0,
            current_layer: 0,
            states: Vec::new(),
        }
    }
    pub fn keycodes<'a>(&'a self) -> impl Iterator<Item = KeyCode> + 'a {
        self.states.iter().filter_map(State::keycode)
    }
    pub fn tick<'a>(&'a mut self) -> impl Iterator<Item = KeyCode> + 'a {
        self.states = self.states.iter().filter_map(State::tick).collect();
        self.update_layer();
        self.keycodes()
    }
    pub fn event<'a>(&'a mut self, event: Event) -> impl Iterator<Item = KeyCode> + 'a {
        match event {
            Release(x, y) => {
                self.states = self
                    .states
                    .iter()
                    .filter_map(|s| s.release((x, y)))
                    .collect()
            }
            Press(x, y) => self.press((x, y), self.current_layer),
        }
        self.update_layer();
        self.keycodes()
    }
    fn press(&mut self, coord: (usize, usize), layer: usize) {
        use crate::action::Action::*;
        let action = match self
            .layers
            .get(layer)
            .and_then(|l| l.get(coord.0))
            .and_then(|l| l.get(coord.1))
        {
            None => return,
            Some(a) => *a,
        };
        match action {
            No => (),
            Trans => {
                if layer != self.default_layer {
                    self.press(coord, self.default_layer)
                }
            }
            KeyCode(keycode) => drop(self.states.push(NormalKey { coord, keycode })),
            MultipleKeyCodes(v) => {
                for &keycode in v {
                    drop(self.states.push(NormalKey { coord, keycode }));
                }
            }
            Layer(value) => drop(self.states.push(LayerModifier { value, coord })),
            DefaultLayer(value) => {
                if value < self.layers.len() {
                    self.default_layer = value
                }
            }
            HoldTap(hold, tap) => drop(self.states.push(State::HoldTap {
                timeout: 200,
                coord,
                hold,
                tap,
            })),
            LayerTap(layer, tap) => drop(self.states.push(State::LayerTap {
                timeout: 200,
                coord,
                layer,
                tap,
            })),
        }
    }
    fn update_layer(&mut self) {
        let mut iter = self.states.iter().filter_map(State::get_layer);
        let mut layer = match iter.next() {
            None => self.default_layer,
            Some(l) => l,
        };
        for l in iter {
            layer += l;
        }
        self.current_layer = layer;
    }
}
