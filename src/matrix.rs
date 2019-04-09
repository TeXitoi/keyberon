use stm32f1xx_hal::gpio;
use stm32f1xx_hal::prelude::*;

pub struct Matrix {
    button0: gpio::gpiob::PB12<gpio::Input<gpio::PullUp>>,
    button1: gpio::gpiob::PB13<gpio::Input<gpio::PullUp>>,
    button2: gpio::gpiob::PB14<gpio::Input<gpio::PullUp>>,
    button3: gpio::gpiob::PB15<gpio::Input<gpio::PullUp>>,
    button4: gpio::gpioa::PA8<gpio::Input<gpio::PullUp>>,
    button5: gpio::gpioa::PA9<gpio::Input<gpio::PullUp>>,
}

impl Matrix {
    pub fn new(
        button0: gpio::gpiob::PB12<gpio::Input<gpio::PullUp>>,
        button1: gpio::gpiob::PB13<gpio::Input<gpio::PullUp>>,
        button2: gpio::gpiob::PB14<gpio::Input<gpio::PullUp>>,
        button3: gpio::gpiob::PB15<gpio::Input<gpio::PullUp>>,
        button4: gpio::gpioa::PA8<gpio::Input<gpio::PullUp>>,
        button5: gpio::gpioa::PA9<gpio::Input<gpio::PullUp>>,
    ) -> Self {
        Self {
            button0,
            button1,
            button2,
            button3,
            button4,
            button5,
        }
    }
    pub fn get(&self) -> [bool; 6] {
        [
            self.button0.is_low(),
            self.button1.is_low(),
            self.button2.is_low(),
            self.button3.is_low(),
            self.button4.is_low(),
            self.button5.is_low(),
        ]
    }
}
