#![allow(missing_docs)]

use embedded_hal::digital::v2::{InputPin, OutputPin};

/// Describes the hardware-level matrix of switches.
/// Generic parameters are in order: The type of column pins,
/// the type of row pins, the number of columns and rows.
/// **NOTE:** In order to be able to put different pin structs
/// in an array they have to be downgraded (stripped of their
/// numbers etc.). Most HAL-s have a method of downgrading pins
/// to a common (erased) struct. (for example see
/// [stm32f0xx_hal::gpio::PA0::downgrade](https://docs.rs/stm32f0xx-hal/0.17.1/stm32f0xx_hal/gpio/gpioa/struct.PA0.html#method.downgrade))
pub struct Matrix<C, R, const CS: usize, const RS: usize>
where
    C: InputPin,
    R: OutputPin,
{
    cols: [C; CS],
    rows: [R; RS],
}

impl<C, R, const CS: usize, const RS: usize> Matrix<C, R, CS, RS>
where
    C: InputPin,
    R: OutputPin,
{
    // Creates a new Matrix; assumes columns are pull-up inputs,
    // and rows are output pins which are set high when not being scanned.
    pub fn new<E>(cols: [C; CS], rows: [R; RS]) -> Result<Self, E>
    where
        C: InputPin<Error = E>,
        R: OutputPin<Error = E>,
    {
        let mut res = Self { cols, rows };
        res.clear()?;
        Ok(res)
    }
    pub fn clear<E>(&mut self) -> Result<(), E>
    where
        C: InputPin<Error = E>,
        R: OutputPin<Error = E>,
    {
        for r in self.rows.iter_mut() {
            r.set_high()?;
        }
        Ok(())
    }
    // Scans the matrix and checks which keys are pressed.
    // Every row pin in order is pulled low, and then each column
    // pin is tested; if it's low, the key is marked as pressed.
    pub fn get<E>(&mut self) -> Result<PressedKeys<CS, RS>, E>
    where
        C: InputPin<Error = E>,
        R: OutputPin<Error = E>,
    {
        let mut keys = PressedKeys::default();

        for (ri, row) in (&mut self.rows).iter_mut().enumerate() {
            row.set_low()?;
            for (ci, col) in (&self.cols).iter().enumerate() {
                if col.is_low()? {
                    keys.0[ri][ci] = true;
                }
            }
            row.set_high()?;
        }
        Ok(keys)
    }
}

#[derive(PartialEq, Eq)]
pub struct PressedKeys<const C: usize, const R: usize>(pub [[bool; C]; R]);

impl<const C: usize, const R: usize> PressedKeys<C, R> {
    pub fn iter_pressed(&self) -> impl Iterator<Item = (usize, usize)> + Clone + '_ {
        self.0.iter().enumerate().flat_map(|(i, r)| {
            r.iter()
                .enumerate()
                .filter_map(move |(j, &b)| if b { Some((i, j)) } else { None })
        })
    }
}

impl<const C: usize, const R: usize> Default for PressedKeys<C, R> {
    fn default() -> Self {
        PressedKeys([[false; C]; R])
    }
}

impl<'a, const C: usize, const R: usize> IntoIterator for &'a PressedKeys<C, R> {
    type IntoIter = core::slice::Iter<'a, [bool; C]>;
    type Item = &'a [bool; C];
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
