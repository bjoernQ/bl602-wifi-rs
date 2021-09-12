use core::mem::MaybeUninit;

use crate::compat::circbuf::{BtPacketType, BT_DRIVER, BT_RECEIVE_QUEUE};

pub mod controller;

static mut BLE_INITIALIZED: bool = false;

extern "C" {
    fn bl602_hci_uart_init(uartid: u8);
}

pub fn ble_init() {
    unsafe {
        *(HCI_OUT_COLLECTOR.as_mut_ptr()) = HciOutCollector::new();

        bl602_hci_uart_init(0);

        if let Some(drv) = BT_DRIVER {
            let open = (*drv).open.unwrap();
            open(drv);
        }

        riscv::interrupt::free(|_| {
            BLE_INITIALIZED = true;
        });
    }
}

pub fn send_hci(data: &[u8]) {
    let hci_out = unsafe { &mut *HCI_OUT_COLLECTOR.as_mut_ptr() };
    hci_out.push(data);

    if hci_out.is_ready() {
        let packet = hci_out.packet();

        let packet_type = match packet[0] {
            1 => BtPacketType::BtCmd as u8,
            2 => BtPacketType::BtAclOut as u8,
            3 => BtPacketType::BtAclIn as u8,
            4 => BtPacketType::BtEvt as u8,
            _ => BtPacketType::BtCmd as u8,
        };

        unsafe {
            if let Some(drv) = BT_DRIVER {
                let send = (*drv).send.unwrap();

                let data_ptr = packet as *const _ as *const u8;
                let data_ptr = data_ptr.offset(1);
                send(drv, packet_type, data_ptr, packet.len() - 1);
            }
        }

        hci_out.reset();
    }
}

static mut BLE_HCI_READ_DATA: [u8; 256] = [0u8; 256];
static mut BLE_HCI_READ_DATA_INDEX: usize = 0;
static mut BLE_HCI_READ_DATA_LEN: usize = 0;

pub fn read_hci(data: &mut [u8]) -> usize {
    unsafe {
        if BLE_HCI_READ_DATA_LEN == 0 {
            let dequeued = BT_RECEIVE_QUEUE.dequeue();
            match dequeued {
                Some(packet) => {
                    for i in 0..(packet.len as usize + 1) {
                        BLE_HCI_READ_DATA[i] = packet.data[i];
                    }

                    BLE_HCI_READ_DATA[0] = match packet.packet_type {
                        1 /*BtPacketType::BT_EVT*/ => 4,
                        3 /*BtPacketType::BT_ACL_IN*/ => 2,
                        _ => 4,
                    };

                    BLE_HCI_READ_DATA_LEN = packet.len as usize + 1;
                    BLE_HCI_READ_DATA_INDEX = 0;
                }
                None => (),
            };
        }

        if BLE_HCI_READ_DATA_LEN > 0 {
            data[0] = BLE_HCI_READ_DATA[BLE_HCI_READ_DATA_INDEX];
            BLE_HCI_READ_DATA_INDEX += 1;

            if BLE_HCI_READ_DATA_INDEX >= BLE_HCI_READ_DATA_LEN {
                BLE_HCI_READ_DATA_LEN = 0;
                BLE_HCI_READ_DATA_INDEX = 0;
            }
            return 1;
        }
    }

    0
}

pub extern "C" fn ble_worker() {
    unsafe {
        while !riscv::interrupt::free(|_| BLE_INITIALIZED) {}

        loop {
            // TODO ??? ble_hci_do_rx();
            for _ in 0..20000 {}
        }
    }
}

pub struct HciPipe {
    wbuffer: [u8; 256],
    rbuffer: [u8; 256],
    w_write_idx: usize,
    w_read_idx: usize,
    r_write_idx: usize,
    r_read_idx: usize,
}

impl HciPipe {
    pub fn new() -> HciPipe {
        HciPipe {
            wbuffer: [0u8; 256],
            rbuffer: [0u8; 256],
            w_write_idx: 0,
            w_read_idx: 0,
            r_write_idx: 0,
            r_read_idx: 0,
        }
    }

    pub fn controller_read(&mut self) -> Option<u8> {
        riscv::interrupt::free(|_| {
            if self.r_write_idx == self.r_read_idx {
                None
            } else {
                let r = self.rbuffer[self.r_read_idx];
                self.r_read_idx += 1;
                if self.r_read_idx >= self.rbuffer.len() {
                    self.r_read_idx = 0;
                }
                Some(r)
            }
        })
    }

    pub fn controller_write(&mut self, v: u8) {
        riscv::interrupt::free(|_| {
            self.wbuffer[self.w_write_idx] = v;
            self.w_write_idx += 1;
            if self.w_write_idx >= self.wbuffer.len() {
                self.w_write_idx = 0;
            }

            if self.w_write_idx == self.w_read_idx {
                panic!("Buffer overflow in controller_write");
            }
        })
    }

    pub fn host_read(&mut self) -> Option<u8> {
        riscv::interrupt::free(|_| {
            if self.w_write_idx == self.w_read_idx {
                None
            } else {
                let r = self.wbuffer[self.w_read_idx];
                self.w_read_idx += 1;
                if self.w_read_idx >= self.wbuffer.len() {
                    self.w_read_idx = 0;
                }
                Some(r)
            }
        })
    }

    pub fn host_peek(&mut self, offset: usize) -> Option<u8> {
        riscv::interrupt::free(|_| {
            if self.w_write_idx == self.w_read_idx {
                None
            } else {
                let index = (self.w_read_idx + offset) % self.wbuffer.len();

                // ???
                if index > self.w_write_idx {
                    None
                } else {
                    Some(self.wbuffer[index])
                }
            }
        })
    }

    pub fn host_write(&mut self, v: u8) {
        riscv::interrupt::free(|_| {
            self.rbuffer[self.r_write_idx] = v;
            self.r_write_idx += 1;
            if self.r_write_idx >= self.rbuffer.len() {
                self.r_write_idx = 0;
            }

            if self.r_write_idx == self.r_read_idx {
                panic!("Buffer overflow in host_write");
            }
        })
    }
}

static mut HCI_OUT_COLLECTOR: MaybeUninit<HciOutCollector> = MaybeUninit::uninit();

#[derive(PartialEq, Debug)]
enum HciOutType {
    Unknown,
    Acl,
    Command,
}

struct HciOutCollector {
    data: [u8; 256],
    index: usize,
    ready: bool,
    kind: HciOutType,
}

impl HciOutCollector {
    fn new() -> HciOutCollector {
        HciOutCollector {
            data: [0u8; 256],
            index: 0,
            ready: false,
            kind: HciOutType::Unknown,
        }
    }

    fn is_ready(&self) -> bool {
        self.ready
    }

    fn push(&mut self, data: &[u8]) {
        self.data[self.index..(self.index + data.len())].copy_from_slice(data);
        self.index += data.len();

        if self.kind == HciOutType::Unknown {
            self.kind = match self.data[0] {
                1 => HciOutType::Command,
                2 => HciOutType::Acl,
                _ => HciOutType::Unknown,
            };
        }

        if !self.ready {
            if self.kind == HciOutType::Command && self.index >= 4 {
                if self.index == self.data[3] as usize + 4 {
                    self.ready = true;
                }
            } else if self.kind == HciOutType::Acl && self.index >= 5 {
                if self.index == (self.data[3] as usize) + ((self.data[4] as usize) << 8) + 5 {
                    self.ready = true;
                }
            }
        }
    }

    fn reset(&mut self) {
        self.index = 0;
        self.ready = false;
        self.kind = HciOutType::Unknown;
    }

    fn packet(&self) -> &[u8] {
        &self.data[0..(self.index as usize)]
    }
}
