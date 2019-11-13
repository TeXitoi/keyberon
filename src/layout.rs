use crate::action::Action;
use crate::key_code::KeyCode;

pub type Layers = &'static [&'static [&'static [Action]]];

pub struct Layout {
    layers: Layers,
    default_layer: usize,
}

impl Layout {
    pub const fn new(layers: Layers) -> Self {
        Self {
            layers,
            default_layer: 0,
        }
    }
    pub fn key_codes<'a>(
        &'a mut self,
        kp: impl Iterator<Item = (usize, usize)> + Clone + 'a,
    ) -> impl Iterator<Item = KeyCode> + 'a {
        let layer = self.layer(kp.clone()).unwrap_or(self.default_layer);
        kp.flat_map(move |(i, j)| match self.layers[layer][i][j] {
            Action::Trans => self.layers[self.default_layer][i][j].key_codes(),
            Action::DefaultLayer(default) => {
                self.default_layer = default;
                Action::DefaultLayer(default).key_codes()
            }
            kc => kc.key_codes(),
        })
    }
    fn layer(&self, kp: impl Iterator<Item = (usize, usize)>) -> Option<usize> {
        let mut iter = kp.filter_map(|(i, j)| self.layers[self.default_layer][i][j].layout());
        let first = iter.next()?;
        Some(first + iter.sum::<usize>())
    }
}
