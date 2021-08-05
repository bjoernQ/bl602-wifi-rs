use core::mem::MaybeUninit;

use crate::{compat::common::StrBuf, log};

pub mod controller;

static mut HCI_PIPE: MaybeUninit<HciPipe> = MaybeUninit::uninit();
static mut BLE_INITIALIZED: bool = false;

extern "C" {
    fn bl602_hci_uart_init(uartid: u8);
    fn ble_hci_do_rx() -> i32;
}

// BL602 NuttX BLE works like this:
// `initialize` will create a file /dev/ptmx and renames /dev/pts to /dev/ble
// so the communication is via PTMX (see https://linux.die.net/man/4/ptmx)
// We just simulate that here

pub struct File {
    _f_oflags: i32,      /* Open mode flags */
    _f_pos: i64,         /* File position */
    _f_inode: *const u8, /* Driver or file system interface */
    _f_priv: *const u8,  /* Per file driver private data */
}

#[no_mangle]
pub unsafe extern "C" fn file_open(
    _filep: &File,
    path: *const u8,
    _oflags: i32,
    _args: ...
) -> i32 {
    let path = StrBuf::from(path);
    log!("file_open {}", path.as_str_ref());
    0
}

#[no_mangle]
pub unsafe extern "C" fn file_ioctl(_filep: &File, _req: i32, _args: ...) -> i32 {
    log!("file_ioctl");
    0
}

#[no_mangle]
pub unsafe extern "C" fn nxsched_get_streams() {
    panic!("nxsched_get_streams is not implemented");
}

#[no_mangle]
pub unsafe extern "C" fn fprintf() {
    panic!("fprintf is not implemented");
}

#[no_mangle]
pub unsafe extern "C" fn close() {
    panic!("close is not implemented");
}

#[no_mangle]
pub unsafe extern "C" fn rename(oldpath: *const u8, newpath: *const u8) -> i32 {
    let oldpath = StrBuf::from(oldpath);
    let newpath = StrBuf::from(newpath);

    log!("rename {} {}", oldpath.as_str_ref(), newpath.as_str_ref());

    0
}

#[no_mangle]
pub unsafe extern "C" fn file_write(_filep: &File, buf: *mut u8, nbytes: i32) -> i16 {
    log!("file_write {:p} {}", buf, nbytes);

    for i in 0..nbytes {
        log!("{:x}", (*(buf.offset(i as isize))));
    }

    for i in 0..nbytes {
        (*HCI_PIPE.as_mut_ptr()).controller_write(*(buf.offset(i as isize)));
    }

    nbytes as i16
}

// ssize_t file_read(FAR struct file *filep, FAR void *buf, size_t nbytes)
#[no_mangle]
pub unsafe extern "C" fn file_read(_filep: &File, buf: *mut u8, nbytes: i32) -> i16 {
    let mut read = 0;
    for i in 0..nbytes {
        let r = (*HCI_PIPE.as_mut_ptr()).controller_read();

        match r {
            Some(b) => {
                log!("got byte {:x}", b);
                *(buf.offset(i as isize)) = b;
                read += 1;
            }
            None => {
                break;
            }
        }
    }

    if read > 0 {
        log!("file_read read {} bytes", read);
    }

    read as i16
}

pub fn ble_init() {
    unsafe {
        *(HCI_PIPE.as_mut_ptr()) = HciPipe::new();

        bl602_hci_uart_init(0);

        riscv::interrupt::free(|_| {
            BLE_INITIALIZED = true;
        });
    }
}

pub fn send_hci(data: &[u8]) {
    for v in data {
        unsafe {
            (*HCI_PIPE.as_mut_ptr()).host_write(*v);
        }
    }
}

pub fn read_hci(data: &mut [u8]) -> usize {
    let mut count = 0;
    for v in data {
        let r = unsafe { (*HCI_PIPE.as_mut_ptr()).host_read() };
        match r {
            Some(b) => {
                *v = b;
                count += 1;
            }
            None => {
                return count;
            }
        }
    }

    count
}

pub extern "C" fn ble_worker() {
    unsafe {
        while !riscv::interrupt::free(|_| BLE_INITIALIZED) {}

        loop {
            ble_hci_do_rx();
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
