//! Hardware pin switch matrix handling.

use embedded_hal::digital::v2::{InputPin, OutputPin};

/// Describes the hardware-level matrix of switches.
///
/// Generic parameters are in order: The type of column pins,
/// the type of row pins, the number of columns and rows.
/// **NOTE:** In order to be able to put different pin structs
/// in an array they have to be downgraded (stripped of their
/// numbers etc.). Most HAL-s have a method of downgrading pins
/// to a common (erased) struct. (for example see
/// [stm32f0xx_hal::gpio::PA0::downgrade](https://docs.rs/stm32f0xx-hal/0.17.1/stm32f0xx_hal/gpio/gpioa/struct.PA0.html#method.downgrade))
#[allow(non_upper_case_globals)]
pub struct Matrix<I, O, const InN: usize, const OutN: usize>
where
    I: InputPin,
    O: OutputPin,
{
    ins: [I; InN],
    outs: [O; OutN],
}

#[allow(non_upper_case_globals)]
impl<I, O, const InN: usize, const OutN: usize, E> Matrix<I, O, InN, OutN>
where
    I: InputPin<Error = E>,
    O: OutputPin<Error = E>,
{
    /// Creates a new Matrix.
    ///
    /// Assumes columns are pull-up inputs,
    /// and rows are output pins which are set high when not being scanned.
    pub fn new(ins: [I; InN], outs: [O; OutN]) -> Result<Self, E>
    where
        I: InputPin<Error = E>,
        O: OutputPin<Error = E>,
    {
        let mut res = Self { ins, outs };
        res.clear()?;
        Ok(res)
    }
    fn clear(&mut self) -> Result<(), E>
    where
        I: InputPin<Error = E>,
        O: OutputPin<Error = E>,
    {
        for r in self.outs.iter_mut() {
            r.set_high()?;
        }
        Ok(())
    }

    /// For each out-pin, sets it to lo-then-high. If an input follows this cycle, then
    /// we can deduce that the key connecting these two pins is pressed
    pub fn down_keys(&mut self) -> Result<[[bool; InN]; OutN], E> {
        self.down_keys_with_delay(|| ())
    }

    /// Same as `down_keys`, with a delay following the set_low() to allow the switch to settle
    pub fn down_keys_with_delay<F: FnMut()>(
        &mut self,
        mut delay: F,
    ) -> Result<[[bool; InN]; OutN], E> {
        let mut keys = [[false; InN]; OutN];

        for (out_idx, out_pin) in self.outs.iter_mut().enumerate() {
            out_pin.set_low()?;
            delay();
            for (in_idx, in_pin) in self.ins.iter().enumerate() {
                if in_pin.is_low()? {
                    keys[out_idx][in_idx] = true;
                }
            }
            out_pin.set_high()?;
        }
        Ok(keys)
    }
}

/// Matrix-representation of switches directly attached to the pins ("diodeless").
///
/// Generic parameters are in order: The type of column pins,
/// the number of columns and rows.
pub struct DirectPinMatrix<P, const CS: usize, const RS: usize>
where
    P: InputPin,
{
    pins: [[Option<P>; CS]; RS],
}

impl<P, const CS: usize, const RS: usize> DirectPinMatrix<P, CS, RS>
where
    P: InputPin,
{
    /// Creates a new DirectPinMatrix.
    ///
    /// Assumes pins are pull-up inputs. Spots in the matrix that are
    /// not corresponding to any pins use ´None´.
    pub fn new<E>(pins: [[Option<P>; CS]; RS]) -> Result<Self, E>
    where
        P: InputPin<Error = E>,
    {
        let res = Self { pins };
        Ok(res)
    }

    /// Scans the pins and checks which keys are pressed (state is "low").
    pub fn get<E>(&mut self) -> Result<[[bool; CS]; RS], E>
    where
        P: InputPin<Error = E>,
    {
        let mut keys = [[false; CS]; RS];

        for (ri, row) in self.pins.iter_mut().enumerate() {
            for (ci, col_option) in row.iter().enumerate() {
                if let Some(col) = col_option {
                    if col.is_low()? {
                        keys[ri][ci] = true;
                    }
                }
            }
        }
        Ok(keys)
    }
}
