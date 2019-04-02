use crate::hid::{HidDevice, Subclass, Protocol, ReportType};
use crate::Led;
use stm32f1xx_hal::prelude::*;

const REPORT_DESCRIPTOR: &[u8] = &[
    0x05, 0x01, 0x09, 0x06, 0xA1, 0x01, 0x05, 0x07, 0x19, 0xE0, 0x29, 0xE7, 0x15, 0x00, 0x25, 0x01,
    0x75, 0x01, 0x95, 0x08, 0x81, 0x02, 0x95, 0x01, 0x75, 0x08, 0x81, 0x03, 0x95, 0x05, 0x75, 0x01,
    0x05, 0x08, 0x19, 0x01, 0x29, 0x05, 0x91, 0x02, 0x95, 0x01, 0x75, 0x03, 0x91, 0x03, 0x95, 0x06,
    0x75, 0x08, 0x15, 0x00, 0x25, 0x65, 0x05, 0x07, 0x19, 0x00, 0x29, 0x65, 0x81, 0x00, 0x09, 0x03,
    0x75, 0x08, 0x95, 0x40, 0xB1, 0x02, 0xC0,
];

pub struct Keyboard {
    iter: core::slice::Iter<'static, u8>,
    report: [u8; 8],
    led: Led,
}
impl Keyboard {
    pub fn new(led: Led) -> Keyboard {
        Keyboard {
            iter: [0x68, 0x65, 0x5c].iter(),
            report: [0; 8],
            led,
        }
    }
}

impl HidDevice for Keyboard {
    fn subclass(&self) -> Subclass {
        Subclass::BootInterface
    }

    fn protocol(&self) -> Protocol {
        Protocol::Keyboard
    }

    fn report_descriptor(&self) -> &[u8] {
        REPORT_DESCRIPTOR
    }

    fn get_report(&mut self, report_type: ReportType, report_id: u8) -> Result<&[u8], ()> {
        if report_type == ReportType::Output {
            self.report[2] = *self.iter.next().unwrap_or(&0);
        }
        Ok(&self.report)
    }

    fn set_report(
        &mut self,
        report_type: ReportType,
        report_id: u8,
        data: &[u8],
    ) -> Result<(), ()> {
        if data.get(0).map_or(false, |c| c & 1 << 1 != 0) {
            self.led.set_low()
        } else {
            self.led.set_high()
        }
        Ok(())
    }
}
