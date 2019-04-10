use crate::key_code::KeyCode::{self, *};

#[rustfmt::skip]
pub static LAYOUT: [[KeyCode; 12]; 5] = [
    [Grave,    Kb1, Kb2, Kb3, Kb4,   Kb5,  Kb6,   Kb7,   Kb8, Kb9,  Kb0,   Minus   ],
    [Tab,      Q,   W,   E,   R,     T,    Y,     U,     I,   O,    P,     LBracket],
    [RBracket, A,   S,   D,   F,     G,    H,     J,     K,   L,    SColon,Quote   ],
    [Equal,    Z,   X,   C,   V,     B,    N,     M,    Comma,Dot,  Slash, Bslash  ],
    [LCtrl,CapsLock,LGui,LAlt,LShift,Space,BSpace,RShift,RAlt,Enter,Delete,RCtrl   ],
];
