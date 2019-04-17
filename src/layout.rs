use crate::action::Action::{self, *};
use crate::action::{k, l, d};
use crate::key_code::KeyCode::{self, *};

#[rustfmt::skip]
pub static LAYERS: [[[Action; 12]; 5]; 2] = [
    [
        [k(Grave),   k(Kb1),k(Kb2),k(Kb3), k(Kb4),  k(Kb5),   k(Kb6),   k(Kb7),  k(Kb8), k(Kb9),  k(Kb0),   k(Minus)   ],
        [k(Tab),     k(Q),  k(W),  k(E),   k(R),    k(T),     k(Y),     k(U),    k(I),   k(O),    k(P),     k(LBracket)],
        [k(RBracket),k(A),  k(S),  k(D),   k(F),    k(G),     k(H),     k(J),    k(K),   k(L),    k(SColon),k(Quote)   ],
        [k(Equal),   k(Z),  k(X),  k(C),   k(V),    k(B),     k(N),     k(M),    k(Comma),k(Dot), k(Slash), k(Bslash)  ],
        [k(LCtrl),   l(1), k(LGui),k(LAlt),k(Space),k(LShift),k(RShift),k(Enter),k(RAlt),k(BSpace),k(Escape),k(RCtrl)  ],
    ], [
        [k(F1),      k(F2),    k(F3),k(F4),k(F5),k(F6),k(F7),k(F8),k(F9),k(F10), k(F11), k(F12)   ],
        [k(Escape),  Trans,    Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans,  Trans,  k(PgUp)  ],
        [d(0),       d(1),     Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans,  Trans,  k(PgDown)],
        [k(CapsLock),k(Delete),Trans,Trans,Trans,Trans,Trans,Trans,Trans,k(Home),k(Up),  k(End)   ],
        [Trans,      Trans,    Trans,Trans,Trans,Trans,Trans,Trans,Trans,k(Left),k(Down),k(Right) ],
    ]
];

pub struct Layout {
    layers: [[[Action; 12]; 5]; 2],
    default_layer: usize,
}

impl Layout {
    pub const fn new(layers: [[[Action; 12]; 5]; 2]) -> Self {
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
        kp.filter_map(move |(i, j)| match self.layers[layer][i][j] {
            Trans => self.layers[self.default_layer][i][j].key_code(),
            DefaultLayer(default) => {
                self.default_layer = default;
                None
            }
            kc => kc.key_code(),
        })
    }
    fn layer(&self, kp: impl Iterator<Item = (usize, usize)>) -> Option<usize> {
        let mut iter = kp.filter_map(|(i, j)| self.layers[self.default_layer][i][j].layout());
        let first = iter.next()?;
        Some(first + iter.sum::<usize>())
    }
}
