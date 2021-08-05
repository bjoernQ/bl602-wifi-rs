// Needs more to be implemented see
// check https://github.com/eupn/stm32wb55
// check https://github.com/danielgallagher0/bluenrg
//
// We don't know any vendor specific commands so a lot of this is just
// here to make the code compile.

use bluetooth_hci::event::{VendorEvent, VendorReturnParameters};

use crate::ble::{read_hci, send_hci, HCI_PIPE};

pub struct BleController;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BusError {
    Other,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Bl602Status {}

impl core::convert::TryFrom<u8> for Bl602Status {
    type Error = bluetooth_hci::BadStatusError;

    fn try_from(_value: u8) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl From<Bl602Status> for u8 {
    fn from(_: Bl602Status) -> Self {
        todo!()
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Bl602Event {}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Bl602ReturnParameters;

impl VendorReturnParameters for Bl602ReturnParameters {
    type Error = BusError;

    fn new(_buffer: &[u8]) -> Result<Self, bluetooth_hci::event::Error<Self::Error>>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl VendorEvent for Bl602Event {
    type Error = BusError;
    type Status = Bl602Status;
    type ReturnParameters = Bl602ReturnParameters;

    fn new(_buffer: &[u8]) -> Result<Self, bluetooth_hci::event::Error<Self::Error>>
    where
        Self: Sized,
    {
        todo!()
    }
}

pub struct Bl602Types;

impl bluetooth_hci::Vendor for Bl602Types {
    type Status = Bl602Status;
    type Event = Bl602Event;
}

impl BleController {
    pub fn new() -> BleController {
        BleController {}
    }
}

impl bluetooth_hci::Controller for BleController {
    type Error = BusError;
    type Header = bluetooth_hci::host::uart::CommandHeader;
    type Vendor = Bl602Types;

    fn write(&mut self, header: &[u8], payload: &[u8]) -> nb::Result<(), Self::Error> {
        send_hci(header);
        send_hci(payload);
        Ok(())
    }

    fn read_into(&mut self, buffer: &mut [u8]) -> nb::Result<(), Self::Error> {
        let count = read_hci(buffer);

        if count == buffer.len() {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    fn peek(&mut self, n: usize) -> nb::Result<u8, Self::Error> {
        let r = unsafe { (*HCI_PIPE.as_mut_ptr()).host_peek(n) };

        match r {
            Some(b) => Ok(b),
            None => Err(nb::Error::WouldBlock),
        }
    }
}
