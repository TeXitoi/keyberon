use stm32f1xx_hal::gpio::{gpioa::*, gpiob::*, Input, Output, PullUp, PushPull};
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
    pub PB12<Input<PullUp>>,
    pub PB13<Input<PullUp>>,
    pub PB14<Input<PullUp>>,
    pub PB15<Input<PullUp>>,
    pub PA8<Input<PullUp>>,
    pub PA9<Input<PullUp>>,
    pub PA10<Input<PullUp>>,
    pub PB5<Input<PullUp>>,
    pub PB6<Input<PullUp>>,
    pub PB7<Input<PullUp>>,
    pub PB8<Input<PullUp>>,
    pub PB9<Input<PullUp>>,
);
impl_getter! {
    Cols,
    dyn _embedded_hal_digital_InputPin,
    12,
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
}

pub struct Rows(
    pub PB11<Output<PushPull>>,
    pub PB10<Output<PushPull>>,
    pub PB1<Output<PushPull>>,
    pub PB0<Output<PushPull>>,
    pub PA7<Output<PushPull>>,
);
impl_getter! {
    Rows,
    dyn _embedded_hal_digital_OutputPin,
    5,
    [0, 1, 2, 3, 4]
}

pub struct Matrix {
    cols: Cols,
    rows: Rows,
}
impl Matrix {
    pub fn new(cols: Cols, mut rows: Rows) -> Self {
        rows.map_mut(|c| c.set_high());
        Self { cols, rows }
    }
    pub fn get(&mut self) -> PressedKeys {
        let cols = &self.cols;
        PressedKeys(self.rows.map_mut(|c| {
            c.set_low();
            cortex_m::asm::delay(5 * 48); // 5µs
            let col = cols.map(|r| r.is_low());
            c.set_high();
            col
        }))
    }
}

#[derive(PartialEq, Eq)]
pub struct PressedKeys(pub [[bool; 12]; 5]);
impl PressedKeys {
    pub const fn new() -> Self {
        Self([[false; 12]; 5])
    }
    pub fn iter_pressed<'a>(&'a self) -> impl Iterator<Item = (usize, usize)> + Clone + 'a {
        self.0.iter().enumerate().flat_map(|(i, r)| {
            r.iter()
                .enumerate()
                .filter_map(move |(j, &b)| if b { Some((i, j)) } else { None })
        })
    }
}
