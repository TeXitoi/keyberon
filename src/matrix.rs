use stm32f1xx_hal::gpio::{gpioa::*, gpiob::*, Input, Output, PullDown, PushPull};
use stm32f1xx_hal::prelude::*;

macro_rules! impl_getter {
    ($s:ident, $t:ty, $len:tt, [$($idx:tt),+]) => {
        impl $s {
            pub fn get(&self, i: usize) -> Option<&$t> {
                match i {
                    $(
                        $idx => Some(&self.$idx as &$t),
                    )+
                        _ => None,
                }
            }
            pub fn get_mut(&mut self, i: usize) -> Option<&mut $t> {
                match i {
                    $(
                        $idx => Some(&mut self.$idx as &mut $t),
                    )+
                        _ => None,
                }
            }
            pub fn len(&self) -> usize {
                $len
            }
            pub fn map<T>(&self, mut f: impl FnMut(&$t) -> T) -> [T; $len] {
                [
                    $(
                        f(self.get($idx).unwrap()),
                    )+
                ]
            }
            pub fn map_mut<T>(&mut self, mut f: impl FnMut(&mut $t) -> T) -> [T; $len] {
                [
                    $(
                        f(self.get_mut($idx).unwrap()),
                    )+
                ]
            }
        }
    }
}

pub struct Cols(
    pub PB12<Output<PushPull>>,
    pub PB13<Output<PushPull>>,
    pub PB14<Output<PushPull>>,
    pub PB15<Output<PushPull>>,
    pub PA8<Output<PushPull>>,
    pub PA9<Output<PushPull>>,
    pub PA10<Output<PushPull>>,
    pub PB5<Output<PushPull>>,
    pub PB6<Output<PushPull>>,
    pub PB7<Output<PushPull>>,
    pub PB8<Output<PushPull>>,
    pub PB9<Output<PushPull>>,
);
impl_getter! {
    Cols,
    _embedded_hal_digital_OutputPin,
    12,
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
}

pub struct Rows(
    pub PB11<Input<PullDown>>,
    pub PB10<Input<PullDown>>,
    pub PB1<Input<PullDown>>,
    pub PB0<Input<PullDown>>,
    pub PA7<Input<PullDown>>,
);
impl_getter! {
    Rows,
    _embedded_hal_digital_InputPin,
    5,
    [0, 1, 2, 3, 4]
}

pub struct Matrix {
    cols: Cols,
    rows: Rows,
}
impl Matrix {
    pub fn new(mut cols: Cols, rows: Rows) -> Self {
        cols.map_mut(|c| c.set_low());
        Self { cols, rows }
    }
    pub fn get(&mut self) -> PressedKeys {
        let rows = &self.rows;
        PressedKeys(self.cols.map_mut(|c| {
            c.set_high();
            let row = rows.map(|r| r.is_high());
            c.set_low();
            row
        }))
    }
}

#[derive(PartialEq, Eq)]
pub struct PressedKeys(pub [[bool; 5]; 12]);
impl PressedKeys {
    pub const fn new() -> Self {
        Self([[false; 5]; 12])
    }
    pub fn iter_pressed<'a>(&'a self) -> impl Iterator<Item = (usize, usize)> + 'a {
        self.0.iter()
            .enumerate()
            .flat_map(|(j, r)| {
                r.iter()
                    .enumerate()
                    .filter_map(move |(i, &b)| if b { Some((i, j)) } else { None })
            })
    }
}
