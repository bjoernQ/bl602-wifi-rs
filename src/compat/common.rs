use embedded_time::duration::Milliseconds;

use crate::{binary::c_types::c_void, compat::get_time, log, print};
use core::fmt::Write;

use super::queue::SimpleQueue;

static mut MUTEXES: [Option<*mut u8>; 1] = [None];
pub static mut EMULATED_TIMER: [Option<EmulatedTimer>; 2] = [None; 2];

#[repr(C)]
#[derive(Debug)]
pub struct timespec {
    tv_sec: u32,
    tv_nsec: u32,
}

#[repr(C)]
pub struct itimerspec {
    it_value: timespec,    /* First time */
    it_interval: timespec, /* and thereafter */
}

pub struct StrBuf {
    buffer: [u8; 512],
    len: usize,
}

impl StrBuf {
    pub fn new() -> StrBuf {
        StrBuf {
            buffer: [0u8; 512],
            len: 0,
        }
    }

    pub unsafe fn from(c_str: *const u8) -> StrBuf {
        let mut res = StrBuf {
            buffer: [0u8; 512],
            len: 0,
        };

        let mut idx: usize = 0;
        while *(c_str.offset(idx as isize)) != 0 {
            res.buffer[idx] = *(c_str.offset(idx as isize));
            idx += 1;
        }

        res.len = idx;
        res
    }

    pub unsafe fn append_from(&mut self, c_str: *const u8) {
        let mut src_idx: usize = 0;
        let mut idx: usize = self.len;
        while *(c_str.offset(src_idx as isize)) != 0 {
            self.buffer[idx] = *(c_str.offset(src_idx as isize));
            idx += 1;
            src_idx += 1;
        }

        self.len = idx;
    }

    pub fn append(&mut self, s: &str) {
        let mut idx: usize = self.len;
        s.chars().for_each(|c| {
            self.buffer[idx] = c as u8;
            idx += 1;
        });
        self.len = idx;
    }

    pub fn append_char(&mut self, c: char) {
        let mut idx: usize = self.len;
        self.buffer[idx] = c as u8;
        idx += 1;
        self.len = idx;
    }

    pub unsafe fn as_str_ref(&self) -> &str {
        core::str::from_utf8_unchecked(&self.buffer[..self.len])
    }
}

impl Write for StrBuf {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        self.append(s);
        Ok(())
    }
}

#[no_mangle]
pub unsafe extern "C" fn syslog(_priority: u32, mut args: ...) {
    log!("syslog called");

    let fmt_str_ptr = args.arg::<*const u8>();

    let mut res_str = StrBuf::new();

    let strbuf = StrBuf::from(fmt_str_ptr);
    let s = strbuf.as_str_ref();

    let mut format_char = ' ';
    let mut is_long = false;
    let mut found = false;
    for c in s.chars().into_iter() {
        if found && format_char != ' ' {
            // have to format an arg
            match format_char {
                'd' => {
                    if is_long {
                        let v = args.arg::<i32>();
                        write!(res_str, "{}", v).ok();
                    } else {
                        let v = args.arg::<i32>();
                        write!(res_str, "{}", v).ok();
                    }
                }

                'u' => {
                    let v = args.arg::<u32>();
                    write!(res_str, "{}", v).ok();
                }

                'p' => {
                    let v = args.arg::<u32>();
                    write!(res_str, "0x{:x}", v).ok();
                }

                'X' => {
                    let v = args.arg::<u32>();
                    write!(res_str, "{:2x}", v).ok();
                }

                'x' => {
                    let v = args.arg::<u32>();
                    write!(res_str, "{:2x}", v).ok();
                }

                's' => {
                    let v = args.arg::<u32>() as *const u8;
                    let vbuf = StrBuf::from(v);
                    write!(res_str, "{}", vbuf.as_str_ref()).ok();
                }

                _ => {
                    write!(res_str, "<UNKNOWN{}>", format_char).ok();
                }
            }

            format_char = ' ';
            found = false;
            is_long = false;
        }

        if !found {
            if c == '%' {
                found = true;
            }

            if !found {
                res_str.append_char(c);
            }
        } else {
            if c.is_numeric() || c == '-' || c == 'l' {
                if c == 'l' {
                    is_long = true;
                }
                // ignore
            } else {
                // a format char
                format_char = c;
            }
        }
    }

    print!("{}", res_str.as_str_ref());
}

#[repr(C)]
pub struct sem {
    semcount: i16,
}

#[no_mangle]
pub unsafe extern "C" fn sem_init(sem: *mut sem, pshared: i32, value: u32) -> i32 {
    log!("sem_init called pshared={} value={}", pshared, value);

    (*sem).semcount = value as i16;

    0 // 0 = no error
}

#[no_mangle]
pub unsafe extern "C" fn sem_post(sem: *mut sem) {
    log!("sem_post called");

    (*sem).semcount += 1;
}

#[no_mangle]
pub unsafe extern "C" fn sem_wait(sem: *mut sem) {
    log!("sem_wait called");

    while (*sem).semcount == 0 {}

    (*sem).semcount -= 1;
}

#[no_mangle]
pub unsafe extern "C" fn sem_timedwait(sem: *mut sem, abstime: *mut timespec) -> i32 {
    log!("sem_timedwait called {:p} {:?}", sem, *abstime);

    while (*sem).semcount == 0 {
        // should be waiting'n breaking
    }

    (*sem).semcount -= 1;

    0
}

#[no_mangle]
pub unsafe extern "C" fn sem_destroy(sem: *mut u8) -> i32 {
    log!("sem_destroy called {:p}", sem);

    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_init(mutex: *mut u8, attr: *mut u8) -> i32 {
    log!("pthread_mutex_init called {:p} {:p}", mutex, attr);

    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_lock(mutex: *mut u8) -> i32 {
    log!("pthread_mutex_lock called {:p}", mutex);

    // TODO check if it's the mutex in question
    while riscv::interrupt::free(|_| MUTEXES[0].is_some()) {
        // wait...
    }
    riscv::interrupt::free(|_| MUTEXES[0] = Some(mutex));

    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_unlock(mutex: *mut u8) -> i32 {
    log!("pthread_mutex_unlock called {:p}", mutex);

    riscv::interrupt::free(|_| MUTEXES[0] = None);
    0
}

#[no_mangle]
pub unsafe extern "C" fn nanosleep() {
    unimplemented!("nanosleep");
}

#[no_mangle]
pub unsafe extern "C" fn usleep(usec: u32) -> i32 {
    log!("usleep called {}", usec);

    // not nearly accurate
    for _ in 0..usec * 10 {}

    0
}

#[no_mangle]
pub unsafe extern "C" fn sleep(sec: u32) -> i32 {
    log!("sleep called {}", sec);

    usleep(sec * 1000);

    0
}

#[no_mangle]
pub unsafe extern "C" fn clock_gettime(clk_id: u32, _ts: *mut timespec /* timespec */) -> i32 {
    log!("clock_gettime called id={}", clk_id);

    let t = get_time();
    (*_ts).tv_sec = t.0 / 1000;
    (*_ts).tv_nsec = 0;

    log!("clock_gettime called, secs = {}", (*_ts).tv_sec);

    0
}

#[repr(C)]
pub union sigval {
    sival_int: i32,           /* Integer value */
    sival_ptr: *const c_void, /* Pointer value */
}

#[repr(C)]
pub struct pthread_attr_s {
    priority: u8,     /* Priority of the pthread */
    policy: u8,       /* Pthread scheduler policy */
    inheritsched: u8, /* Inherit parent priority/policy? */
    detachstate: u8,  /* Initialize to the detach state */
    low_priority: u8, /* Low scheduling priority */
    max_repl: u8,     /* Maximum pending replenishments */

    stackaddr: *const c_void, /* Address of memory to be used as stack */
    stacksize: usize,         /* Size of the stack allocated for the pthread */
}

#[repr(C)]
pub struct sigevent {
    sigev_notify: u8, /* Notification method: SIGEV_SIGNAL, SIGEV_NONE, or SIGEV_THREAD */
    sigev_signo: u8,  /* Notification signal */
    sigev_value: sigval, /* Data passed with notification */

    sigev_notify_function: fn(), /* Notification function */
    sigev_notify_attributes: *const pthread_attr_s, /* Notification attributes (not used) */
}

#[derive(Debug, Clone, Copy)]
pub struct EmulatedTimer {
    pub notify_function: fn(),
    pub interval_secs: u32,
    pub next_notify: u32,
}

#[no_mangle]
pub unsafe extern "C" fn timer_create(clockid: u32, sevp: *mut sigevent, timerid: *mut u32) -> i32 {
    log!("timer_create called {} {:p} {:p}", clockid, sevp, timerid);
    log!("timer_create notify_fn={:p}", (*sevp).sigev_notify_function);

    let mut free_idx = 0;
    for &et in EMULATED_TIMER.iter() {
        if et.is_some() {
            free_idx += 1;
        } else {
            break;
        }
    }
    if free_idx == EMULATED_TIMER.len() {
        panic!("No more timers left");
    }

    *timerid = free_idx as u32;

    EMULATED_TIMER[free_idx] = Some(EmulatedTimer {
        notify_function: (*sevp).sigev_notify_function,
        interval_secs: 0,
        next_notify: 0,
    });

    0
}

#[no_mangle]
pub unsafe extern "C" fn timer_settime(
    timerid: u32,
    flags: u32,
    new_value: *const itimerspec,
    old_value: *const itimerspec,
) -> i32 {
    log!(
        "timer_settime called {} {} {:p} {:p}",
        timerid,
        flags,
        new_value,
        old_value
    );

    log!(
        "new_time sec/nsec {} {}",
        (*new_value).it_value.tv_sec,
        (*new_value).it_value.tv_nsec
    );
    log!(
        "new_time interval sec/nsec {} {}",
        (*new_value).it_interval.tv_sec,
        (*new_value).it_interval.tv_nsec
    );

    EMULATED_TIMER[timerid as usize] = match &EMULATED_TIMER[timerid as usize] {
        core::option::Option::Some(old) => Some(EmulatedTimer {
            notify_function: old.notify_function,
            interval_secs: (*new_value).it_interval.tv_sec,
            next_notify: (get_time() + Milliseconds::new((*new_value).it_interval.tv_sec * 1000)).0,
        }),
        core::option::Option::None => None,
    };

    0
}

#[no_mangle]
pub unsafe extern "C" fn timer_delete(timerid: u32) -> i32 {
    log!("timer_delete called {}", timerid);

    EMULATED_TIMER[timerid as usize] = None;

    0
}

#[no_mangle]
pub unsafe extern "C" fn file_mq_open() {
    // just an empty stub

    log!("file_mq_open called");
}

struct MqMessage {
    data: [u8; 256],
    len: usize,
}

static mut MESSAGES: SimpleQueue<MqMessage> = SimpleQueue::new();

#[no_mangle]
pub unsafe extern "C" fn file_mq_send(
    mq: *const u8,
    message: *const u8,
    msglen: u32,
    prio: u32,
) -> i32 {
    log!(
        "file_mq_send called mq={:p} msglen={} prio={}",
        mq,
        msglen,
        prio,
    );

    let mut data = [0u8; 256];
    for i in 0..msglen as usize {
        data[i] = *(message.offset(i as isize));
    }

    let msg = MqMessage {
        data,
        len: msglen as usize,
    };

    MESSAGES.enqueue(msg);

    0
}

#[no_mangle]
pub unsafe extern "C" fn file_mq_timedsend() {
    // just an empty stub

    log!("file_mq_timedsend called");
}

#[no_mangle]
pub unsafe extern "C" fn file_mq_timedreceive(
    mq: *const u8,
    msg: *mut u8,
    msglen: u32,
    prio: *mut u32,
    abstime: *const timespec,
) -> i32 {
    log!(
        "file_mq_timedreceive called {:p} {:p} {} {:p}=>{} {:?}",
        mq,
        msg,
        msglen,
        prio,
        *prio,
        *abstime,
    );

    let mut received_bytes: i32 = 0;

    let wait_end = Milliseconds::new((*abstime).tv_sec * 1000 + 1000);
    while MESSAGES.is_empty() {
        if get_time() > wait_end {
            break;
        }
    }

    match MESSAGES.dequeue() {
        core::option::Option::Some(message) => {
            for i in 0..message.len {
                *(msg.offset(i as isize)) = message.data[i];
            }

            log!("copied message with len {}", message.len);

            received_bytes = message.len as i32;
        }
        core::option::Option::None => {}
    };

    received_bytes
}

#[no_mangle]
pub unsafe extern "C" fn mq_open(mq_name: *const u8, oflags: i32, ...) -> i32 {
    let strbuf = StrBuf::from(mq_name);
    log!("mq_open called {} {}", strbuf.as_str_ref(), oflags);

    42
}

static FAKE_FILE: [u8; 0] = [0; 0];

#[no_mangle]
pub unsafe extern "C" fn fs_getfilep(fd: i32, filep: *mut *const u8) -> i32 {
    log!("fs_getfilep called fd:{}", fd);

    // probably not necessary - seems the code in the blob doesn't really care
    *filep = &FAKE_FILE as *const _ as *const u8;

    0
}

#[no_mangle]
pub unsafe extern "C" fn _assert(filename: *const u8, line: u32) {
    let strbuf = StrBuf::from(filename);
    log!("_assert called {}:{}", strbuf.as_str_ref(), line);
}

#[no_mangle]
pub unsafe extern "C" fn g_system_timer() {
    log!("g_system_timer called");
    unimplemented!("g_system_timer");
}

#[no_mangle]
pub unsafe extern "C" fn printf(s: *const u8, args: ...) {
    log!("printf called");

    syslog(0, s, args);
}

#[no_mangle]
pub unsafe extern "C" fn strlen(s: *const u8) -> i32 {
    log!("strlen called");

    let mut i = 0;
    while *(s.offset(i)) != 0 {
        i += 1;
    }

    i as i32
}

#[no_mangle]
pub unsafe extern "C" fn snprintf(dst: *mut u8, n: u32, format: *const u8, mut args: ...) {
    log!("snprintf called n={}", n);

    // almost same code as syslog ... dedup that

    let fmt_str_ptr = format;

    let mut res_str = StrBuf::new();

    let strbuf = StrBuf::from(fmt_str_ptr);
    let s = strbuf.as_str_ref();

    let mut format_char = ' ';
    let mut is_long = false;
    let mut found = false;
    for c in s.chars().into_iter() {
        if found && format_char != ' ' {
            // have to format an arg
            match format_char {
                'd' => {
                    if is_long {
                        let v = args.arg::<i32>();
                        write!(res_str, "{}", v).ok();
                    } else {
                        let v = args.arg::<i32>();
                        write!(res_str, "{}", v).ok();
                    }
                }

                'p' => {
                    let v = args.arg::<u32>();
                    write!(res_str, "0x{:x}", v).ok();
                }

                'X' => {
                    let v = args.arg::<u32>();
                    write!(res_str, "{:2x}", v).ok();
                }

                'x' => {
                    let v = args.arg::<u32>();
                    write!(res_str, "{:2x}", v).ok();
                }

                's' => {
                    let v = args.arg::<u32>() as *const u8;
                    let vbuf = StrBuf::from(v);
                    write!(res_str, "{}", vbuf.as_str_ref()).ok();
                }

                _ => {
                    write!(res_str, "<UNKNOWN{}>", format_char).ok();
                }
            }

            format_char = ' ';
            found = false;
            is_long = false;
        }

        if !found {
            if c == '%' {
                found = true;
            }

            if !found {
                res_str.append_char(c);
            }
        } else {
            if c.is_numeric() || c == '-' || c == 'l' {
                if c == 'l' {
                    is_long = true;
                }
                // ignore
            } else {
                // a format char
                format_char = c;
            }
        }
    }

    let mut idx = 0;
    res_str.as_str_ref().chars().for_each(|c| {
        *(dst.offset(idx)) = c as u8;
        idx += 1;
    });
    *(dst.offset(idx)) = 0;
}

#[no_mangle]
pub unsafe extern "C" fn puts(s: *const u8) {
    log!("puts called");

    let mut str_buf = StrBuf::new();
    let mut i = 0;
    while *(s.offset(i)) != 0 {
        str_buf.append_char(*(s.offset(i)) as char);
        i += 1;
    }

    print!("{}", str_buf.as_str_ref());
}

#[no_mangle]
pub unsafe extern "C" fn zalloc() {
    log!("zalloc called");

    unimplemented!("zalloc");
}

#[no_mangle]
pub unsafe extern "C" fn __errno() {
    log!("__errno called");

    unimplemented!("__errno");
}

#[no_mangle]
pub unsafe extern "C" fn __truncdfsf2(a: f64) -> f32 {
    log!("__truncdfsf2 called {}", a);

    // WORLD'S DUMBEST WAY TO CONVERT A DOUBLE TO FLOAT
    let mut str_buf = StrBuf::new();
    write!(str_buf, "{}", a).ok();
    let res = str_buf.as_str_ref().parse::<f32>().unwrap();
    res
}

#[no_mangle]
pub unsafe extern "C" fn strcmp(str1: *const u8, str2: *const u8) -> i32 {
    log!("strcmp called");

    let mut fmt_str_ptr = str1;
    while *fmt_str_ptr != 0 {
        fmt_str_ptr = fmt_str_ptr.offset(1);
    }

    let mut fmt_str_ptr = str2;
    while *fmt_str_ptr != 0 {
        fmt_str_ptr = fmt_str_ptr.offset(1);
    }

    let mut i = 0;
    let mut a = 0u8;
    let mut b = 0u8;

    while *(str1.offset(i)) == *(str2.offset(i)) {
        a = *(str1.offset(i));
        b = *(str2.offset(i));

        if a == 0 && b == 0 {
            return 0;
        }

        i += 1;
    }

    if a < b {
        -1
    } else {
        1
    }
}

#[no_mangle]
pub unsafe extern "C" fn strncpy(dest: *mut u8, src: *const u8, n: u32) -> *const u8 {
    log!("strncpy called");

    let mut dstidx = 0;
    for i in 0isize..n as isize {
        dstidx = i;
        *(dest.offset(i)) = *(src.offset(i));
        if *(src.offset(i)) == 0 {
            break;
        }
    }

    if dstidx < n as isize {
        *(dest.offset(dstidx)) = 0;
    }

    dest
}
