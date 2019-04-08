#![no_main]
#![no_std]

extern crate panic_semihosting;

macro_rules! dbg {
    ($val:expr) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                use core::fmt::Write;
                let mut out = cortex_m_semihosting::hio::hstdout().unwrap();
                writeln!(out, "[{}:{}] {} = {:#?}",
                    file!(), line!(), stringify!($val), &tmp).unwrap();
                tmp
            }
        }
    }
}

mod hid;
mod keyboard;

use crate::keyboard::Keyboard;
use rtfm::app;
use stm32f103xx_usb::UsbBus;
use stm32f1xx_hal::gpio;
use stm32f1xx_hal::prelude::*;
use usb_device::bus;
use usb_device::class::UsbClass;
use usb_device::prelude::*;

type KeyboardHidClass = hid::HidClass<'static, UsbBus, Keyboard>;
type Led = gpio::gpioc::PC13<gpio::Output<gpio::PushPull>>;

#[app(device = stm32f1xx_hal::stm32)]
const APP: () = {
    static mut USB_DEV: UsbDevice<'static, UsbBus> = ();
    static mut USB_CLASS: KeyboardHidClass = ();

    #[init]
    fn init() -> init::LateResources {
        static mut USB_BUS: Option<bus::UsbBusAllocator<UsbBus>> = None;

        let mut flash = device.FLASH.constrain();
        let mut rcc = device.RCC.constrain();

        let clocks = rcc
            .cfgr
            .use_hse(8.mhz())
            .sysclk(48.mhz())
            .pclk1(24.mhz())
            .freeze(&mut flash.acr);

        let mut gpioc = device.GPIOC.split(&mut rcc.apb2);
        let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
        led.set_high();

        let mut gpioa = device.GPIOA.split(&mut rcc.apb2);

        *USB_BUS = Some(UsbBus::usb_with_reset(
            device.USB,
            &mut rcc.apb1,
            &clocks,
            &mut gpioa.crh,
            gpioa.pa12,
        ));
        let usb_bus = USB_BUS.as_ref().unwrap();

        let usb_class = hid::HidClass::new(Keyboard::new(led), &usb_bus);
        let mut usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x1209, 0xa1e5))
            .manufacturer("RIIR Task Force")
            .product("Keyboard")
            .serial_number(env!("CARGO_PKG_VERSION"))
            .build();

        usb_dev.force_reset().expect("reset failed");

        init::LateResources {
            USB_DEV: usb_dev,
            USB_CLASS: usb_class,
        }
    }

    #[interrupt(priority = 2, resources = [USB_DEV, USB_CLASS])]
    fn USB_HP_CAN_TX() {
        usb_poll(&mut resources.USB_DEV, &mut resources.USB_CLASS);
    }

    #[interrupt(priority = 2, resources = [USB_DEV, USB_CLASS])]
    fn USB_LP_CAN_RX0() {
        static mut CNT: u32 = 0;
        usb_poll(&mut resources.USB_DEV, &mut resources.USB_CLASS);
        if *CNT > 100 {
            rtfm::pend(stm32f1xx_hal::stm32::Interrupt::EXTI1);
        }
        *CNT += 1;
    }

    #[interrupt(priority = 1, resources = [USB_CLASS])]
    fn EXTI1() {
        dbg!("trying...");
        while let Ok(0) = resources.USB_CLASS.lock(|k| k.write(&[0,0,0x4,0,0,0,0,0])) {}
        while let Ok(0) = resources.USB_CLASS.lock(|k| k.write(&[0,0,0,0,0,0,0,0])) {}
        dbg!("done");
    }
};

fn usb_poll(usb_dev: &mut UsbDevice<'static, UsbBus>, keyboard: &mut KeyboardHidClass) {
    if usb_dev.poll(&mut [keyboard]) {
        keyboard.poll();
    }
}
