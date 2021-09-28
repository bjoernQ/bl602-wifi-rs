use crate::log;

use super::queue::SimpleQueue;

#[repr(C)]
pub struct dq_entry_s {
    flink: *mut u8,
    blink: *mut u8,
}

#[repr(C)]
pub struct work_s {
    dq: dq_entry_s,    /* Implements a doubly linked list */
    worker: *const u8, /* Work callback */
    arg: *const u8,    /* Callback argument */
    qtime: i64,        /* Time work queued */
    delay: i64,        /* Delay until work performed */
}

static mut WORKER_HIGH: SimpleQueue<extern "C" fn()> = SimpleQueue::new();
static mut WORKER_LOW: SimpleQueue<extern "C" fn()> = SimpleQueue::new();

#[no_mangle]
pub unsafe extern "C" fn work_queue(
    qid: i32,
    _work: *mut work_s,
    worker: extern "C" fn(),
    arg: *mut u8,
    delay: i32,
) -> i32 {
    log!("work_queue qid={} arg={:p} delay={}", qid, arg, delay);

    if qid == 0 {
        riscv::interrupt::free(|_| {
            WORKER_HIGH.enqueue(worker);
        });
    } else {
        riscv::interrupt::free(|_| {
            WORKER_LOW.enqueue(worker);
        });
    }

    0
}

pub fn do_work(qid: i32) {
    unsafe {
        let mut todo: [Option<extern "C" fn()>; 10] = [None; 10];

        riscv::interrupt::free(|_| {
            todo.iter_mut().for_each(|e| {
                let work = if qid == 0 {
                    WORKER_HIGH.dequeue()
                } else {
                    WORKER_LOW.dequeue()
                };
                match work {
                    Some(worker) => {
                        e.replace(worker);
                    }
                    None => {}
                }
            });
        });

        for worker in todo.iter() {
            match worker {
                core::option::Option::Some(f) => {
                    log!("before worker");

                    f();

                    log!("after worker");
                }
                core::option::Option::None => {}
            }
        }
    }
}
