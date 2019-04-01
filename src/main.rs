#![no_main]
#![no_std]
#![allow(non_snake_case)]

/// CDC-ACM serial port example using cortex-m-rtfm.

extern crate panic_semihosting;

mod cdc_acm;
mod hid;

use rtfm::app;
use stm32f1xx_hal::prelude::*;

use usb_device::prelude::*;
use stm32f103xx_usb::UsbBus;
use usb_device::bus;

#[app(device = stm32f1xx_hal::stm32)]
const APP: () = {

    static mut USB_DEV: UsbDevice<'static, UsbBus> = ();
    static mut SERIAL: cdc_acm::SerialPort<'static, UsbBus> = ();

    #[init]
    fn init() {
        static mut USB_BUS: Option<bus::UsbBusAllocator<UsbBus>> = None;

        let mut flash = device.FLASH.constrain();
        let mut rcc = device.RCC.constrain();

        let clocks = rcc.cfgr
            .use_hse(8.mhz())
            .sysclk(48.mhz())
            .pclk1(24.mhz())
            .freeze(&mut flash.acr);

        assert!(clocks.usbclk_valid());

        let mut gpioa = device.GPIOA.split(&mut rcc.apb2);

        *USB_BUS = Some(UsbBus::usb_with_reset(
            device.USB, &mut rcc.apb1,
            &clocks, &mut gpioa.crh, gpioa.pa12));

        let serial = cdc_acm::SerialPort::new(USB_BUS.as_ref().unwrap());

        let mut usb_dev = UsbDeviceBuilder::new(
                USB_BUS.as_ref().unwrap(),
                UsbVidPid(0x5824, 0x27dd))
            .manufacturer("RIIR Task Force")
            .product("Keyboard")
            .serial_number(env!("CARGO_PKG_VERSION"))
            .device_class(cdc_acm::USB_CLASS_CDC)
            .build();

        usb_dev.force_reset().expect("reset failed");

        USB_DEV = usb_dev;
        SERIAL = serial;
    }

    #[interrupt(resources = [USB_DEV, SERIAL])]
    fn USB_HP_CAN_TX() {
        usb_poll(&mut resources.USB_DEV, &mut resources.SERIAL);
    }

    #[interrupt(resources = [USB_DEV, SERIAL])]
    fn USB_LP_CAN_RX0() {
        usb_poll(&mut resources.USB_DEV, &mut resources.SERIAL);
    }
};

fn usb_poll<B: bus::UsbBus>(
    usb_dev: &mut UsbDevice<'static, B>,
    serial: &mut cdc_acm::SerialPort<'static, B>)
{
    if !usb_dev.poll(&mut [serial]) {
        return;
    }

    let mut buf = [0u8; 8];

    match serial.read(&mut buf) {
        Ok(count) if count > 0 => {
            // Echo back in upper case
            for c in buf[0..count].iter_mut() {
                if 0x61 <= *c && *c <= 0x7a {
                    *c &= !0x20;
                }
            }

            serial.write(&buf[0..count]).ok();
        },
        _ => { },
    }
}
