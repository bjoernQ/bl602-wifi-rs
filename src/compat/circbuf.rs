use crate::{
    binary::c_types::c_void,
    compat::malloc::{free, malloc},
};

use super::queue::SimpleQueue;

#[derive(Debug)]
#[repr(C)]
pub struct CircBuf {
    base: *mut u8,  /* The pointer to buffer space */
    size: usize,    /* The size of buffer space */
    head: usize,    /* The head of buffer space */
    tail: usize,    /* The tail of buffer space */
    external: bool, /* The flag for external buffer */
}

/****************************************************************************
 * Name: circbuf_init
 *
 * Description:
 *   Initialize a circular buffer.
 *
 * Input Parameters:
 *   circ  - Address of the circular buffer to be used.
 *   base  - A pointer to circular buffer's internal buffer. It can be
 *           provided by caller because sometimes the creation of buffer
 *           is special or needs to preallocated, eg: DMA buffer.
 *           If NULL, a buffer of the given size will be allocated.
 *   bytes - The size of the internal buffer.
 *
 * Returned Value:
 *   Zero on success; A negated errno value is returned on any failure.
 *
 ****************************************************************************/
#[no_mangle]
pub unsafe extern "C" fn circbuf_init(circ: *mut CircBuf, base: *const u8, bytes: usize) -> i32 {
    (*circ).external = true;

    if base.is_null() {
        let buffer = malloc(bytes as u32);
        (*circ).base = buffer as *mut u8;
        (*circ).external = false;
    } else {
        (*circ).base = base as *mut _;
    }

    (*circ).size = bytes;
    (*circ).head = 0;
    (*circ).tail = 0;

    0
}

/****************************************************************************
 * Name: circbuf_uninit
 *
 * Description:
 *   Free the circular buffer.
 *
 * Input Parameters:
 *   circ  - Address of the circular buffer to be used.
 ****************************************************************************/
#[no_mangle]
pub unsafe extern "C" fn circbuf_uninit(circ: *const CircBuf) {
    if (*circ).external {
        free((*circ).base);
    }
}

/****************************************************************************
 * Name: circbuf_used
 *
 * Description:
 *   Return the used bytes of the circular buffer.
 *
 * Input Parameters:
 *   circ  - Address of the circular buffer to be used.
 ****************************************************************************/
#[no_mangle]
pub unsafe extern "C" fn circbuf_used(circ: *const CircBuf) -> usize {
    let mut used = (*circ).head as i32 - (*circ).tail as i32;
    if used < 0 {
        used = (*circ).size as i32 + used;
    }

    used as usize
}

/****************************************************************************
 * Name: circbuf_read
 *
 * Description:
 *   Get data form the circular buffer.
 *
 * Note :
 *   That with only one concurrent reader and one concurrent writer,
 *   you don't need extra locking to use these api.
 *
 * Input Parameters:
 *   circ  - Address of the circular buffer to be used.
 *   dst   - Address where to store the data.
 *   bytes - Number of bytes to get.
 *
 * Returned Value:
 *   The bytes of get data is returned if the read data is successful;
 *   A negated errno value is returned on any failure.
 ****************************************************************************/
#[no_mangle]
pub unsafe extern "C" fn circbuf_read(circ: *mut CircBuf, dst: *mut u8, bytes: usize) -> i32 {
    let count = usize::min(circbuf_used(circ), bytes) as isize;
    let mut t = (*circ).base.offset((*circ).tail as isize);
    for i in 0..count {
        dst.offset(i).write_volatile(t.read_volatile());

        t = t.offset(1);
        if t > (*circ).base.offset((*circ).size as isize) {
            t = (*circ).base;
        }
    }

    (*circ).tail = t as usize - (*circ).base as usize;

    count as i32
}

/****************************************************************************
 * Name: circbuf_write
 *
 * Description:
 *   Write data to the circular buffer.
 *
 * Note:
 *   That with only one concurrent reader and one concurrent writer,
 *   you don't need extra locking to use these api.
 *
 * Input Parameters:
 *   circ  - Address of the circular buffer to be used.
 *   src   - The data to be added.
 *   bytes - Number of bytes to be added.
 *
 * Returned Value:
 *   The bytes of get data is returned if the write data is successful;
 *   A negated errno value is returned on any failure.
 ****************************************************************************/
#[no_mangle]
pub unsafe extern "C" fn circbuf_write(circ: *mut CircBuf, src: *const u8, bytes: usize) -> i32 {
    let count = bytes as isize;
    let mut t = (*circ).base.offset((*circ).head as isize);
    for i in 0..count {
        let b = src.offset(i).read();
        t.write(b);

        t = t.offset(1);
        if t > (*circ).base.offset((*circ).size as isize) {
            t = (*circ).base;
        }
    }

    (*circ).head = t as usize - (*circ).base as usize;

    count as i32
}

/****************************************************************************
 * Name: uart_bth4_register
 *
 * Description:
 *   Register bluetooth H:4 UART driver.
 *
 ****************************************************************************/
#[no_mangle]
pub unsafe extern "C" fn uart_bth4_register(_path: *const u8, drv: *mut BtDriver) -> i32 {
    (*drv).receive = Some(bt_receive);
    BT_DRIVER = Some(drv);

    0
}

pub static mut BT_DRIVER: Option<*mut BtDriver> = None;

unsafe extern "C" fn bt_receive(_drv: *mut BtDriver, buf_type: u8, dst: *mut u8, len: usize) {
    let mut data = [0u8; 256];
    for i in 0..len {
        let b = dst.offset(i as isize).read();
        data[i + 1] = b;
    }

    let packet = ReceivedPacket {
        packet_type: buf_type,
        len: len as u8,
        data,
    };

    BT_RECEIVE_QUEUE.enqueue(packet);
}

#[derive(Debug)]
#[repr(C)]
pub struct BtDriver {
    head_reserve: usize,
    pub open: ::core::option::Option<unsafe extern "C" fn(drv: *mut BtDriver)>,

    pub send: ::core::option::Option<
        unsafe extern "C" fn(drv: *mut BtDriver, buf_type: u8, src: *const u8, len: usize),
    >,

    pub close: ::core::option::Option<unsafe extern "C" fn(drv: *mut BtDriver)>,

    pub receive: ::core::option::Option<
        unsafe extern "C" fn(drv: *mut BtDriver, buf_type: u8, dst: *mut u8, len: usize),
    >,

    pub private: *const c_void,
}

pub static mut BT_RECEIVE_QUEUE: SimpleQueue<ReceivedPacket> = SimpleQueue::new();

pub struct ReceivedPacket {
    pub packet_type: u8,
    pub len: u8,
    pub data: [u8; 256],
}

pub enum BtPacketType {
    BtCmd = 0,    /* HCI command */
    BtEvt = 1,    /* HCI event */
    BtAclOut = 2, /* Outgoing ACL data */
    BtAclIn = 3,  /* Incoming ACL data */
    BtIsoOut = 4, /* Outgoing ISO data */
    BtIsoIn = 5,  /* Incoming ISO data */
    BtDummy = 99, /* Only used for waking up kernel threads */
}
