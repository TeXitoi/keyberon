#![no_std]

pub mod action;
pub mod debounce;
pub mod hid;
pub mod key_code;
pub mod keyboard;
pub mod layout;
pub mod matrix;

pub type Class<'a, B, L> = hid::HidClass<'a, B, keyboard::Keyboard<L>>;

pub fn new_class<B, L>(bus: &usb_device::bus::UsbBusAllocator<B>, leds: L) -> Class<'_, B, L>
where
    B: usb_device::bus::UsbBus,
    L: keyboard::Leds,
{
    hid::HidClass::new(keyboard::Keyboard::new(leds), bus)
}
