use crate::action::Action;
use crate::key_code::{KbHidReport, KeyCode};

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
    pub fn report_from_pressed<'a>(
        &'a mut self,
        kp: impl Iterator<Item = (usize, usize)> + Clone + 'a,
    ) -> KbHidReport {
        let mut report = KbHidReport::default();
        for kc in self.key_codes(kp) {
            report.pressed(kc);
        }
        report
    }
    pub fn key_codes<'a>(
        &'a mut self,
        kp: impl Iterator<Item = (usize, usize)> + Clone + 'a,
    ) -> impl Iterator<Item = KeyCode> + 'a {
        let layer = self.layer(kp.clone()).unwrap_or(self.default_layer);
        kp.flat_map(move |(i, j)| {
            let action = self
                .layers
                .get(layer)
                .unwrap_or(&self.layers[self.default_layer])
                .get(i)
                .and_then(|c| c.get(j).copied());
            match action {
                None => Action::No.key_codes(),
                Some(Action::Trans) => self.layers[self.default_layer]
                    .get(i)
                    .and_then(|c| c.get(j).copied())
                    .unwrap_or(Action::No)
                    .key_codes(),
                Some(Action::DefaultLayer(default)) => {
                    if default < self.layers.len() {
                        self.default_layer = default;
                    }
                    Action::DefaultLayer(default).key_codes()
                }
                Some(kc) => kc.key_codes(),
            }
        })
    }
    fn layer(&self, kp: impl Iterator<Item = (usize, usize)>) -> Option<usize> {
        let mut iter =
            kp.filter_map(|(i, j)| self.layers[self.default_layer].get(i)?.get(j)?.layout());
        let first = iter.next()?;
        Some(first + iter.sum::<usize>())
    }
}
