use ble_hci::HciConnector;

use crate::timer::timestamp;

use super::{read_hci, send_hci};

pub struct BleConnector {}

impl HciConnector for BleConnector {
    fn read(&self) -> Option<u8> {
        let mut buffer = [0u8];
        let len = read_hci(&mut buffer);

        if len == 0 {
            None
        } else {
            Some(buffer[0])
        }
    }

    fn write(&self, data: u8) {
        send_hci(&[data]);
    }

    fn millis(&self) -> u64 {
        timestamp().millis as u64
    }
}
