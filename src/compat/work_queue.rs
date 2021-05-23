use crate::log;

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

static mut WORKER: [Option<extern "C" fn()>; 4] = [None; 4]; // TODO should be a queue

#[no_mangle]
pub unsafe extern "C" fn work_queue(
    qid: i32,
    _work: *mut work_s,
    worker: extern "C" fn(),
    arg: *mut u8,
    delay: i32,
) -> i32 {
    log!("work_queue qid={} arg={:p} delay={}", qid, arg, delay);

    riscv::interrupt::free(|_| {
        let free_idx = WORKER
            .iter()
            .enumerate()
            .find(|v| v.1.is_none())
            .map(|v| v.0);

        match free_idx {
            Some(idx) => {
                WORKER[idx] = Some(worker);
            }
            None => {
                panic!("Already queued too many workers!");
            }
        }
    });

    0
}

pub fn do_work() {
    unsafe {
        let mut todo: [Option<extern "C" fn()>; 4] = [None; 4];

        riscv::interrupt::free(|_| {
            for i in 0..WORKER.len() {
                todo[i] = WORKER[i].take();
            }
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
