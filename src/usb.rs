//! Keyberon USB constructors.

use crate::{hid, keyboard};
use usb_device::bus::UsbBusAllocator;
use usb_device::device::StringDescriptors;
use usb_device::prelude::*;

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
