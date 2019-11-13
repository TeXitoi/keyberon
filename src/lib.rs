#![no_std]

use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;

pub mod action;
pub mod debounce;
pub mod hid;
pub mod key_code;
pub mod keyboard;
pub mod layout;
pub mod matrix;

pub type Class<'a, B, L> = hid::HidClass<'a, B, keyboard::Keyboard<L>>;
pub type Device<'a, B> = UsbDevice<'a, B>;

// Generic keyboard from
// https://github.com/obdev/v-usb/blob/master/usbdrv/USB-IDs-for-free.txt
const VID: u16 = 0x27db;
const PID: u16 = 0x16c0;

pub fn new_class<B, L>(bus: &UsbBusAllocator<B>, leds: L) -> Class<'_, B, L>
where
    B: usb_device::bus::UsbBus,
    L: keyboard::Leds,
{
    hid::HidClass::new(keyboard::Keyboard::new(leds), bus)
}

pub fn new_device<B>(bus: &UsbBusAllocator<B>) -> Device<'_, B>
where
    B: usb_device::bus::UsbBus,
{
    UsbDeviceBuilder::new(bus, UsbVidPid(VID, PID))
        .manufacturer("RIIR Task Force")
        .product("Keyberon")
        .serial_number(env!("CARGO_PKG_VERSION"))
        .build()
}
