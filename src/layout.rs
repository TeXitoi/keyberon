use crate::action::Action::{self, *};
use crate::action::{k, l};
use crate::key_code::KeyCode::{self, *};

#[rustfmt::skip]
pub static LAYOUT: Layout = Layout([
    [
        [k(Grave),   k(Kb1),k(Kb2),k(Kb3), k(Kb4),  k(Kb5),   k(Kb6),   k(Kb7),  k(Kb8), k(Kb9),  k(Kb0),   k(Minus)   ],
        [k(Tab),     k(Q),  k(W),  k(E),   k(R),    k(T),     k(Y),     k(U),    k(I),   k(O),    k(P),     k(LBracket)],
        [k(RBracket),k(A),  k(S),  k(D),   k(F),    k(G),     k(H),     k(J),    k(K),   k(L),    k(SColon),k(Quote)   ],
        [k(Equal),   k(Z),  k(X),  k(C),   k(V),    k(B),     k(N),     k(M),    k(Comma),k(Dot), k(Slash), k(Bslash)  ],
        [k(LCtrl),   l(1), k(LGui),k(LAlt),k(Space),k(LShift),k(RShift),k(Enter),k(RAlt),k(BSpace),k(Delete),k(RCtrl)  ],
    ], [
        [k(F1),      k(F2),    k(F3),k(F4),k(F5),k(F6),k(F7),k(F8),k(F9),k(F10), k(F11), k(F12)   ],
        [k(Escape),  Trans,    Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans,  Trans,  k(PgUp)  ],
        [Trans,      Trans,    Trans,Trans,Trans,Trans,Trans,Trans,Trans,Trans,  Trans,  k(PgDown)],
        [k(CapsLock),k(Delete),Trans,Trans,Trans,Trans,Trans,Trans,Trans,k(Home),k(Up),  k(End)   ],
        [Trans,      Trans,    Trans,Trans,Trans,Trans,Trans,Trans,Trans,k(Left),k(Down),k(Right) ],
    ]
]);

pub struct Layout([[[Action; 12]; 5]; 2]);

impl Layout {
    pub fn key_codes<'a>(
        &'a self,
        kp: impl Iterator<Item = (usize, usize)> + Clone + 'a,
    ) -> impl Iterator<Item = KeyCode> + 'a {
        let layer: usize = kp
            .clone()
            .filter_map(|(i, j)| self.0[0][i][j].layout())
            .sum();
        kp.filter_map(move |(i, j)| match self.0[layer][i][j] {
            Trans => self.0[0][i][j].key_code(),
            kc => kc.key_code(),
        })
    }
}
