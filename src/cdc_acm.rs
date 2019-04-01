/// Minimal and incomplete CDC-ACM implementation for the examples - this will eventually be a real
/// crate!

use core::cmp::min;
use usb_device::class_prelude::*;
use usb_device::Result;

pub const USB_CLASS_CDC: u8 = 0x02;
const USB_CLASS_DATA: u8 = 0x0a;
const CDC_SUBCLASS_ACM: u8 = 0x02;
const CDC_PROTOCOL_AT: u8 = 0x01;

const CS_INTERFACE: u8 = 0x24;
const CDC_TYPE_HEADER: u8 = 0x00;
const CDC_TYPE_CALL_MANAGEMENT: u8 = 0x01;
const CDC_TYPE_ACM: u8 = 0x02;
const CDC_TYPE_UNION: u8 = 0x06;

const REQ_SET_LINE_CODING: u8 = 0x20;
const REQ_SET_CONTROL_LINE_STATE: u8 = 0x22;

pub struct SerialPort<'a, B: UsbBus> {
    comm_if: InterfaceNumber,
    comm_ep: EndpointIn<'a, B>,
    data_if: InterfaceNumber,
    read_ep: EndpointOut<'a, B>,
    write_ep: EndpointIn<'a, B>,
    buf: [u8; 64],
    len: usize,
    need_zlp: bool,
}

impl<B: UsbBus> SerialPort<'_, B> {
    pub fn new(alloc: &UsbBusAllocator<B>) -> SerialPort<'_, B> {
        SerialPort {
            comm_if: alloc.interface(),
            comm_ep: alloc.interrupt(8, 255),
            data_if: alloc.interface(),
            read_ep: alloc.bulk(64),
            write_ep: alloc.bulk(64),
            buf: [0; 64],
            len: 0,
            need_zlp: false,
        }
    }

    pub fn write(&mut self, data: &[u8]) -> Result<usize> {
        if self.need_zlp {
            return Ok(0);
        }

        if data.len() == 64 {
            self.need_zlp = true;
        }

        match self.write_ep.write(data) {
            Ok(count) => Ok(count),
            Err(UsbError::WouldBlock) => Ok(0),
            e => e,
        }
    }

    pub fn read(&mut self, data: &mut [u8]) -> Result<usize> {
        // Terrible buffering implementation for brevity's sake

        if self.len == 0 {
            self.len = match self.read_ep.read(&mut self.buf) {
                Ok(0) | Err(UsbError::WouldBlock) => return Ok(0),
                Ok(count) => count,
                e => return e,
            };
        }

        let count = min(data.len(), self.len);

        &data[..count].copy_from_slice(&self.buf[0..count]);

        self.buf.rotate_left(count);
        self.len -= count;

        Ok(count)
    }
}

impl<B: UsbBus> UsbClass<B> for SerialPort<'_, B> {
    fn get_configuration_descriptors(&self, writer: &mut DescriptorWriter) -> Result<()> {
        writer.interface(
            self.comm_if,
            USB_CLASS_CDC,
            CDC_SUBCLASS_ACM,
            CDC_PROTOCOL_AT)?;

        writer.write(
            CS_INTERFACE,
            &[CDC_TYPE_HEADER, 0x10, 0x01])?;

        writer.write(
            CS_INTERFACE,
            &[CDC_TYPE_CALL_MANAGEMENT, 0x00, self.data_if.into()])?;

        writer.write(
            CS_INTERFACE,
            &[CDC_TYPE_ACM, 0x00])?;

        writer.write(
            CS_INTERFACE,
            &[CDC_TYPE_UNION, self.comm_if.into(), self.data_if.into()])?;

        writer.endpoint(&self.comm_ep)?;

        writer.interface(
            self.data_if,
            USB_CLASS_DATA,
            0x00,
            0x00)?;

        writer.endpoint(&self.write_ep)?;
        writer.endpoint(&self.read_ep)?;

        Ok(())
    }

    fn endpoint_in_complete(&mut self, addr: EndpointAddress) {
        if self.need_zlp && addr == self.write_ep.address() {
            self.need_zlp = false;
            self.write_ep.write(&[]).ok();
        }
    }

    fn control_out(&mut self, xfer: ControlOut<B>) {
        let req = *xfer.request();

        if req.request_type == control::RequestType::Class
            && req.recipient == control::Recipient::Interface
        {
            return match req.request {
                REQ_SET_LINE_CODING => xfer.accept().unwrap(),
                REQ_SET_CONTROL_LINE_STATE => xfer.accept().unwrap(),
                _ => xfer.reject().unwrap(),
            };
        }
    }
}