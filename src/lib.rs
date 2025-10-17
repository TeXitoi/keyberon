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
//! crate](https://crates.io/crates/usb-device) for USB communication
//! if the `usb` feature is enabled.

#![no_std]
#![deny(missing_docs)]

pub mod action;
pub mod chording;
pub mod debounce;
#[cfg(feature = "usb")]
pub mod hid;
pub mod key_code;
#[cfg(feature = "usb")]
pub mod keyboard;
pub mod layout;
pub mod matrix;
#[cfg(feature = "usb")]
mod usb;

#[cfg(feature = "usb")]
pub use usb::{new_class, new_device, Class};
