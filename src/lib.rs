//! Keyberon is a rust crate to create a pure rust keyboard firmware.
//!
//! It is exposed as a library giving you the different building
//! blocks to create a featureful keyboard firmware. As the different
//! functionality are interconected by the user of the crate, you can
//! use only the parts you are interested in or easily insert your own
//! code in between.
//!
//! This crate is a no_std crate, running on stable rust. To use it on
//! a given MCU, you need GPIO throw the [embedded hal
//! crate](https://crates.io/crates/embedded-hal) to read the key
//! states, and the [usb-device
//! crate](https://crates.io/crates/usb-device) for USB communication.

#![no_std]
#![deny(missing_docs)]

use usb_device::bus::UsbBusAllocator;
use usb_device::device::StringDescriptors;
use usb_device::prelude::*;

pub mod action;
pub mod chording;
pub mod debounce;
pub mod hid;
pub mod key_code;
pub mod keyboard;
pub mod layout;
pub mod matrix;

/// A handly shortcut for the keyberon USB class type.
pub type Class<'a, B, L> = hid::HidClass<'a, B, keyboard::Keyboard<L>>;

/// USB VIP for a generic keyboard from
/// https://github.com/obdev/v-usb/blob/master/usbdrv/USB-IDs-for-free.txt
const VID: u16 = 0x16c0;

/// USB PID for a generic keyboard from
/// https://github.com/obdev/v-usb/blob/master/usbdrv/USB-IDs-for-free.txt
const PID: u16 = 0x27db;

/// Constructor for `Class`.
pub fn new_class<B, L>(bus: &UsbBusAllocator<B>, leds: L) -> Class<'_, B, L>
where
    B: usb_device::bus::UsbBus,
    L: keyboard::Leds,
{
    hid::HidClass::new(keyboard::Keyboard::new(leds), bus)
}

/// Constructor for a keyberon USB device.
pub fn new_device<B>(bus: &UsbBusAllocator<B>) -> usb_device::device::UsbDevice<'_, B>
where
    B: usb_device::bus::UsbBus,
{
    UsbDeviceBuilder::new(bus, UsbVidPid(VID, PID))
        .strings(&[StringDescriptors::default()
            .manufacturer("RIIR Task Force")
            .product("Keyberon")
            .serial_number(env!("CARGO_PKG_VERSION"))])
        .expect("Failed to configure UsbDeviceBuilder")
        .build()
}
