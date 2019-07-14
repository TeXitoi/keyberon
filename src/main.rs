#![no_main]
#![no_std]

extern crate panic_semihosting;

#[allow(unused)]
macro_rules! dbg {
    ($val:expr) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                use core::fmt::Write;
                let mut out = cortex_m_semihosting::hio::hstdout().unwrap();
                writeln!(
                    out,
                    "[{}:{}] {} = {:#?}",
                    file!(),
                    line!(),
                    stringify!($val),
                    &tmp
                )
                .unwrap();
                tmp
            }
        }
    };
}

pub mod action;
pub mod debounce;
pub mod hid;
pub mod key_code;
pub mod keyboard;
pub mod layout;
pub mod matrix;

use crate::debounce::Debouncer;
use crate::keyboard::Keyboard;
use crate::matrix::{Matrix, PressedKeys};
use rtfm::app;
use stm32_usbd::{UsbBus, UsbBusType};
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::stm32;
use stm32f1xx_hal::{gpio, timer};
use usb_device::bus;
use usb_device::class::UsbClass;
use usb_device::prelude::*;

type KeyboardHidClass = hid::HidClass<'static, UsbBusType, Keyboard>;
type Led = gpio::gpioc::PC13<gpio::Output<gpio::PushPull>>;

// Generic keyboard from
// https://github.com/obdev/v-usb/blob/master/usbdrv/USB-IDs-for-free.txt
const VID: u16 = 0x27db;
const PID: u16 = 0x16c0;

#[app(device = stm32f1xx_hal::stm32)]
const APP: () = {
    static mut USB_DEV: UsbDevice<'static, UsbBusType> = ();
    static mut USB_CLASS: KeyboardHidClass = ();
    static mut MATRIX: Matrix = ();
    static mut DEBOUNCER: Debouncer<PressedKeys> =
        Debouncer::new(PressedKeys::new(), PressedKeys::new(), 5);
    static mut LAYOUT: layout::Layout = layout::Layout::new(layout::LAYERS);
    static mut TIMER: timer::Timer<stm32::TIM3> = ();

    #[init]
    fn init() -> init::LateResources {
        static mut USB_BUS: Option<bus::UsbBusAllocator<UsbBusType>> = None;

        let mut flash = device.FLASH.constrain();
        let mut rcc = device.RCC.constrain();

        let clocks = rcc
            .cfgr
            .use_hse(8.mhz())
            .sysclk(48.mhz())
            .pclk1(24.mhz())
            .freeze(&mut flash.acr);

        let mut gpioa = device.GPIOA.split(&mut rcc.apb2);
        let mut gpiob = device.GPIOB.split(&mut rcc.apb2);
        let mut gpioc = device.GPIOC.split(&mut rcc.apb2);

        let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
        led.set_high();

        // BluePill board has a pull-up resistor on the D+ line.
        // Pull the D+ pin down to send a RESET condition to the USB bus.
        let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
        usb_dp.set_low();
        cortex_m::asm::delay(clocks.sysclk().0 / 100);

        let usb_dm = gpioa.pa11;
        let usb_dp = usb_dp.into_floating_input(&mut gpioa.crh);

        *USB_BUS = Some(UsbBus::new(device.USB, (usb_dm, usb_dp)));
        let usb_bus = USB_BUS.as_ref().unwrap();

        let usb_class = hid::HidClass::new(Keyboard::new(led), &usb_bus);
        let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(VID, PID))
            .manufacturer("RIIR Task Force")
            .product("Keyberon")
            .serial_number(env!("CARGO_PKG_VERSION"))
            .build();

        let mut timer = timer::Timer::tim3(device.TIM3, 1.khz(), clocks, &mut rcc.apb1);
        timer.listen(timer::Event::Update);

        init::LateResources {
            USB_DEV: usb_dev,
            USB_CLASS: usb_class,
            TIMER: timer,
            MATRIX: matrix::Matrix::new(
                matrix::Cols(
                    gpiob.pb12.into_pull_up_input(&mut gpiob.crh),
                    gpiob.pb13.into_pull_up_input(&mut gpiob.crh),
                    gpiob.pb14.into_pull_up_input(&mut gpiob.crh),
                    gpiob.pb15.into_pull_up_input(&mut gpiob.crh),
                    gpioa.pa8.into_pull_up_input(&mut gpioa.crh),
                    gpioa.pa9.into_pull_up_input(&mut gpioa.crh),
                    gpioa.pa10.into_pull_up_input(&mut gpioa.crh),
                    gpiob.pb5.into_pull_up_input(&mut gpiob.crl),
                    gpiob.pb6.into_pull_up_input(&mut gpiob.crl),
                    gpiob.pb7.into_pull_up_input(&mut gpiob.crl),
                    gpiob.pb8.into_pull_up_input(&mut gpiob.crh),
                    gpiob.pb9.into_pull_up_input(&mut gpiob.crh),
                ),
                matrix::Rows(
                    gpiob.pb11.into_push_pull_output(&mut gpiob.crh),
                    gpiob.pb10.into_push_pull_output(&mut gpiob.crh),
                    gpiob.pb1.into_push_pull_output(&mut gpiob.crl),
                    gpiob.pb0.into_push_pull_output(&mut gpiob.crl),
                    gpioa.pa7.into_push_pull_output(&mut gpioa.crl),
                ),
            ),
        }
    }

    #[interrupt(priority = 2, resources = [USB_DEV, USB_CLASS])]
    fn USB_HP_CAN_TX() {
        usb_poll(&mut resources.USB_DEV, &mut resources.USB_CLASS);
    }

    #[interrupt(priority = 2, resources = [USB_DEV, USB_CLASS])]
    fn USB_LP_CAN_RX0() {
        usb_poll(&mut resources.USB_DEV, &mut resources.USB_CLASS);
    }

    #[interrupt(priority = 1, resources = [USB_CLASS, MATRIX, DEBOUNCER, LAYOUT, TIMER])]
    fn TIM3() {
        resources.TIMER.clear_update_interrupt_flag();

        if resources.DEBOUNCER.update(resources.MATRIX.get()) {
            let data = resources.DEBOUNCER.get();
            let mut report = key_code::KbHidReport::default();
            for kc in resources.LAYOUT.key_codes(data.iter_pressed()) {
                report.pressed(kc);
            }
            while let Ok(0) = resources.USB_CLASS.lock(|k| k.write(report.as_bytes())) {}
        }
    }
};

fn usb_poll(usb_dev: &mut UsbDevice<'static, UsbBusType>, keyboard: &mut KeyboardHidClass) {
    if usb_dev.poll(&mut [keyboard]) {
        keyboard.poll();
    }
}
