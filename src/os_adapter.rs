use crate::{
    binary::wifi_mgmr::bl_ops_funcs_t,
    compat::{
        bl602::{irq_attach, up_enable_irq},
        common::{
            pthread_mutex_lock, pthread_mutex_unlock, sleep, usleep, vsnprintf, EmulatedTimer,
            StrBuf, EMULATED_TIMER,
        },
        get_time,
        malloc::{free, malloc},
        queue::SimpleQueue,
        work_queue::work_queue,
    },
    log, print, println,
    wifi::bl602_net_event,
};

#[no_mangle]
pub extern "C" fn bl_os_assert_func() {}

#[no_mangle]
static g_bl_ops_funcs: bl_ops_funcs_t = bl_ops_funcs_t {
    _version: 0x0001,
    _printf: Some(bl_os_printf),
    _init: Some(bl_os_api_init),
    _enter_critical: Some(bl_os_enter_critical),
    _exit_critical: Some(bl_os_exit_critical),
    _msleep: Some(bl_os_msleep),
    _sleep: Some(bl_os_sleep),
    _event_notify: Some(bl_os_event_notify),
    _lock_gaint: Some(bl_os_lock_gaint),
    _unlock_gaint: Some(bl_os_unlock_giant),
    _irq_attach: Some(bl_os_irq_attach),
    _irq_enable: Some(bl_os_irq_enable),
    _irq_disable: Some(bl_os_irq_disable),
    _workqueue_create: Some(bl_os_workqueue_create),
    _workqueue_submit_hp: Some(bl_os_workqueue_submit_hp),
    _workqueue_submit_lp: Some(bl_os_workqueue_submit_lp),
    _timer_create: Some(bl_os_timer_create),
    _timer_delete: Some(bl_os_timer_delete),
    _timer_start_once: Some(bl_os_timer_start_once),
    _timer_start_periodic: Some(bl_os_timer_start_periodic),
    _sem_create: Some(bl_os_sem_create),
    _sem_delete: Some(bl_os_sem_delete),
    _sem_take: Some(bl_os_sem_take),
    _sem_give: Some(bl_os_sem_give),
    _mutex_create: Some(bl_os_mutex_create),
    _mutex_delete: Some(bl_os_mutex_delete),
    _mutex_lock: Some(bl_os_mutex_lock),
    _mutex_unlock: Some(bl_os_mutex_unlock),
    _queue_create: Some(bl_os_queue_create),
    _queue_delete: Some(bl_os_queue_delete),
    _queue_send: Some(bl_os_queue_send),
    _queue_recv: Some(bl_os_queue_recv),
    _malloc: Some(bl_os_malloc),
    _free: Some(bl_os_free),
    _zalloc: Some(bl_os_zalloc),
    _get_time_ms: Some(bl_os_get_time_ms),
    _assert: Some(bl_os_assert),
    _event_group_create: Some(bl_os_event_group_create),
    _event_group_delete: Some(bl_os_event_group_delete),
    _event_group_send: Some(bl_os_event_group_send),
    _event_group_wait: Some(bl_os_event_group_wait),
    _event_register: Some(bl_os_event_register),
    _task_create: Some(bl_os_task_create),
    _task_delete: Some(bl_os_task_delete),
    _task_get_current_task: Some(bl_os_task_get_current_task),
    _task_notify_create: Some(bl_os_task_notify_create),
    _task_notify: Some(bl_os_task_notify),
    _task_wait: Some(bl_os_task_wait),
    _queue_send_wait: Some(bl_os_queue_send_wait),
    _get_tick: Some(bl_os_get_tick),
    _log_write: Some(bl_os_log_write),
};

unsafe extern "C" fn bl_os_printf(fmt: *const crate::binary::c_types::c_char, args: ...) {
    let mut buf = [0u8; 512];
    crate::compat::common::vsnprintf(&mut buf as *mut u8, 511, fmt as *const u8, args);
    let res_str = StrBuf::from(&buf as *const u8);
    print!("{}", res_str.as_str_ref());
}

/****************************************************************************
 * Name: bl_os_api_init
 *
 * Description:
 *
 * Input Parameters:
 *
 * Returned Value:
 *
 ****************************************************************************/
pub extern "C" fn bl_os_api_init() -> i32 {
    0
}

/****************************************************************************
 * Name: bl_os_enter_critical
 *
 * Description:
 *   Enter critical state
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   CPU PS value
 *
 ****************************************************************************/
pub unsafe extern "C" fn bl_os_enter_critical() -> u32 {
    log!("Unimplemented bl_os_enter_critical");
    crate::compat::bl602::up_irq_save();
    //riscv::interrupt::disable();
    1
}

/****************************************************************************
 * Name: bl_os_exit_critical
 *
 * Description:
 *   Exit from critical state
 *
 * Input Parameters:
 *   level - CPU PS value
 *
 * Returned Value:
 *   None
 *
 ****************************************************************************/
pub unsafe extern "C" fn bl_os_exit_critical(_level: u32) {
    log!("Unimplemented bl_os_exit_critical");
    crate::compat::bl602::up_irq_restore(1);
    //riscv::interrupt::enable();
}

/****************************************************************************
 * Name: bl_os_msleep
 *
 * Description:
 *
 * Input Parameters:
 *
 * Returned Value:
 *
 ****************************************************************************/
pub unsafe extern "C" fn bl_os_msleep(
    ms: crate::binary::c_types::c_long,
) -> crate::binary::c_types::c_int {
    log!("msleep");
    usleep(ms as u32);
    0
}

/****************************************************************************
 * Name: bl_os_sleep
 *
 * Description:
 *
 * Input Parameters:
 *
 * Returned Value:
 *
 ****************************************************************************/
pub unsafe extern "C" fn bl_os_sleep(
    seconds: crate::binary::c_types::c_uint,
) -> crate::binary::c_types::c_int {
    log!("sleep");
    sleep(seconds);
    0
}

/****************************************************************************
 * Name: bl_os_event_notify
 *
 * Description:
 *
 * Input Parameters:
 *
 * Returned Value:
 *
 ****************************************************************************/
unsafe extern "C" fn bl_os_event_notify(
    evt: crate::binary::c_types::c_int,
    val: crate::binary::c_types::c_int,
) -> crate::binary::c_types::c_int {
    log!("event_notify");
    bl602_net_event(evt, val as u32);
    return 0;
}

/****************************************************************************
 * Name: bl_os_lock_gaint
 *
 * Description:
 *
 * Input Parameters:
 *
 * Returned Value:
 *
 ****************************************************************************/
unsafe extern "C" fn bl_os_lock_gaint() {}

/****************************************************************************
 * Name: bl_os_unlock_giant
 *
 * Description:
 *
 * Input Parameters:
 *
 * Returned Value:
 *
 ****************************************************************************/
unsafe extern "C" fn bl_os_unlock_giant() {}

/****************************************************************************
 * Name: bl_os_irq_attach
 *
 * Description:
 *
 * Input Parameters:
 *
 * Returned Value:
 *
 ****************************************************************************/
unsafe extern "C" fn bl_os_irq_attach(
    n: i32,
    f: *mut crate::binary::c_types::c_void,
    arg: *mut crate::binary::c_types::c_void,
) {
    log!("irq attach {} {:p} {:p}", n, f, arg);
    let isr = core::mem::transmute(f);
    irq_attach(n, isr, arg as *mut _ as *const u8);
}

/****************************************************************************
 * Name: bl_os_irq_enable
 *
 * Description:
 *
 * Input Parameters:
 *
 * Returned Value:
 *
 ****************************************************************************/
unsafe extern "C" fn bl_os_irq_enable(n: i32) {
    log!("irq enable {}", n);
    up_enable_irq(n);
}

/****************************************************************************
 * Name: bl_os_irq_disable
 *
 * Description:
 *
 * Input Parameters:
 *
 * Returned Value:
 *
 ****************************************************************************/
unsafe extern "C" fn bl_os_irq_disable(_n: i32) {
    unimplemented!("irq_disable");
}

/****************************************************************************
 * Name: bl_os_workqueue_create
 *
 * Description:
 *
 * Input Parameters:
 *
 * Returned Value:
 *
 ****************************************************************************/

unsafe extern "C" fn bl_os_workqueue_create() -> *mut crate::binary::c_types::c_void {
    log!("workqueue_create");
    1 as *mut crate::binary::c_types::c_void
}

/****************************************************************************
 * Name: bl_os_workqueue_submit_hpwork
 *
 * Description:
 *
 * Input Parameters:
 *
 * Returned Value:
 *
 ****************************************************************************/
unsafe extern "C" fn bl_os_workqueue_submit_hp(
    work: *mut crate::binary::c_types::c_void,
    worker: *mut crate::binary::c_types::c_void,
    argv: *mut crate::binary::c_types::c_void,
    tick: crate::binary::c_types::c_long,
) -> crate::binary::c_types::c_int {
    log!(
        "workqueue_submit_hp {:p} {:p} {:p} {}",
        work,
        worker,
        argv,
        tick
    );
    let worker = core::mem::transmute(worker);
    work_queue(
        0,
        core::ptr::null_mut(),
        worker,
        core::ptr::null_mut(),
        tick,
    )
}

/****************************************************************************
 * Name: bl_os_workqueue_submit_lpwork
 *
 * Description:
 *
 * Input Parameters:
 *
 * Returned Value:
 *
 ****************************************************************************/
unsafe extern "C" fn bl_os_workqueue_submit_lp(
    work: *mut crate::binary::c_types::c_void,
    worker: *mut crate::binary::c_types::c_void,
    argv: *mut crate::binary::c_types::c_void,
    tick: crate::binary::c_types::c_long,
) -> crate::binary::c_types::c_int {
    log!(
        "workqueue_submit_lp {:p} {:p} {:p} {}",
        work,
        worker,
        argv,
        tick
    );
    let worker = core::mem::transmute(worker);
    work_queue(
        1,
        core::ptr::null_mut(),
        worker,
        core::ptr::null_mut(),
        tick,
    )
}

/****************************************************************************
 * Name: bl_os_timer_create
 *
 * Description:
 *
 * Input Parameters:
 *
 * Returned Value:
 *
 ****************************************************************************/
unsafe extern "C" fn bl_os_timer_create(
    func: *mut crate::binary::c_types::c_void,
    argv: *mut crate::binary::c_types::c_void,
) -> *mut crate::binary::c_types::c_void {
    log!("timer_create {:p} {:p}", func, argv);

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

    EMULATED_TIMER[free_idx] = Some(EmulatedTimer {
        notify_function: core::mem::transmute(func),
        interval_secs: 0,
        next_notify: 0,
    });

    (free_idx + 1) as *mut crate::binary::c_types::c_void
}

/****************************************************************************
 * Name: bl_os_timer_delete
 *
 * Description:
 *
 * Input Parameters:
 *
 * Returned Value:
 *
 ****************************************************************************/
unsafe extern "C" fn bl_os_timer_delete(
    _timerid: *mut crate::binary::c_types::c_void,
    _tick: u32,
) -> crate::binary::c_types::c_int {
    unimplemented!()
}

/****************************************************************************
 * Name: os_timer_start_once
 *
 * Description:
 *
 * Input Parameters:
 *
 * Returned Value:
 *
 ****************************************************************************/
unsafe extern "C" fn bl_os_timer_start_once(
    _timerid: *mut crate::binary::c_types::c_void,
    _t_sec: crate::binary::c_types::c_long,
    _t_nsec: crate::binary::c_types::c_long,
) -> crate::binary::c_types::c_int {
    unimplemented!()
}

/****************************************************************************
 * Name: os_timer_start_periodic
 *
 * Description:
 *
 * Input Parameters:
 *
 * Returned Value:
 *
 ****************************************************************************/
unsafe extern "C" fn bl_os_timer_start_periodic(
    timerid: *mut crate::binary::c_types::c_void,
    t_sec: crate::binary::c_types::c_long,
    t_nsec: crate::binary::c_types::c_long,
) -> crate::binary::c_types::c_int {
    log!("timer_start_periodic {:p} {} {}", timerid, t_sec, t_nsec);

    let mut timer = &mut EMULATED_TIMER[timerid as usize - 1].unwrap();
    timer.interval_secs = t_sec as u32;
    timer.next_notify = get_time().0 + t_sec as u32 * 1000;

    log!(
        "curr time = {} - next notify {}",
        get_time(),
        timer.next_notify
    );

    0
}

/****************************************************************************
 * Name: bl_os_sem_create
 *
 * Description:
 *   Create and initialize semaphore
 *
 * Input Parameters:
 *   max  - No mean
 *   init - semaphore initialization value
 *
 * Returned Value:
 *   Semaphore data pointer
 *
 ****************************************************************************/
unsafe extern "C" fn bl_os_sem_create(init: u32) -> *mut crate::binary::c_types::c_void {
    log!("create sem {}", init);

    let mut res = 0xffff;
    for (i, sem) in CURR_SEM.iter().enumerate() {
        if let None = *sem {
            res = i;
            break;
        }
    }

    log!("sem created res = {} (+1)", res);

    if res != 0xffff {
        CURR_SEM[res] = Some(init);
        (res + 1) as *mut crate::binary::c_types::c_void
    } else {
        core::ptr::null_mut()
    }
}

static mut CURR_SEM: [Option<u32>; 10] =
    [None, None, None, None, None, None, None, None, None, None];

/****************************************************************************
 * Name: bl_os_sem_delete
 *
 * Description:
 *   Delete semaphore
 *
 * Input Parameters:
 *   semphr - Semaphore data pointer
 *
 * Returned Value:
 *   None
 *
 ****************************************************************************/
unsafe extern "C" fn bl_os_sem_delete(semphr: *mut crate::binary::c_types::c_void) {
    log!("sem delete {:p}", semphr);
    CURR_SEM[semphr as usize - 1] = None;
}

/****************************************************************************
 * Name: bl_os_sem_take
 *
 * Description:
 *   Wait semaphore within a certain period of time
 *
 * Input Parameters:
 *   semphr - Semaphore data pointer
 *   ticks  - Wait system ticks
 *
 * Returned Value:
 *   True if success or false if fail
 *
 ****************************************************************************/
unsafe extern "C" fn bl_os_sem_take(semphr: *mut crate::binary::c_types::c_void, tick: u32) -> i32 {
    log!("sem_take {:p} {}", semphr, tick);

    let forever = if tick == 0 { true } else { false };
    let tick = if tick == 0 { 1 } else { tick };

    loop {
        for _ in 0..tick as usize {
            let res = riscv::interrupt::free(|_| {
                if let Some(cnt) = CURR_SEM[semphr as usize - 1] {
                    if cnt > 0 {
                        CURR_SEM[semphr as usize - 1] = Some(cnt - 1);
                        1
                    } else {
                        0
                    }
                } else {
                    0
                }
            });

            if res == 1 {
                return 1;
            }
        }

        if !forever {
            break;
        }
    }

    0
}

/****************************************************************************
 * Name: bl_os_sem_give
 *
 * Description:
 *   Post semaphore
 *
 * Input Parameters:
 *   semphr - Semaphore data pointer
 *
 * Returned Value:
 *   True if success or false if fail
 *
 ****************************************************************************/
unsafe extern "C" fn bl_os_sem_give(semphr: *mut crate::binary::c_types::c_void) -> i32 {
    log!("sem_give {:p}", semphr);

    let res = riscv::interrupt::free(|_| {
        if let Some(cnt) = CURR_SEM[semphr as usize - 1] {
            CURR_SEM[semphr as usize - 1] = Some(cnt + 1);
            1
        } else {
            0
        }
    });

    res
}

/****************************************************************************
 * Name: bl_os_mutex_create
 *
 * Description:
 *   Create mutex
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   Mutex data pointer
 *
 ****************************************************************************/
unsafe extern "C" fn bl_os_mutex_create() -> *mut crate::binary::c_types::c_void {
    log!("mutex create");
    1 as *mut crate::binary::c_types::c_void
}

/****************************************************************************
 * Name: bl_os_mutex_delete
 *
 * Description:
 *   Delete mutex
 *
 * Input Parameters:
 *   mutex_data - mutex data pointer
 *
 * Returned Value:
 *   None
 *
 ****************************************************************************/
unsafe extern "C" fn bl_os_mutex_delete(_mutex: *mut crate::binary::c_types::c_void) {
    unimplemented!("mutex delete")
}

/****************************************************************************
 * Name: bl_os_mutex_lock
 *
 * Description:
 *   Lock mutex
 *
 * Input Parameters:
 *   mutex_data - mutex data pointer
 *
 * Returned Value:
 *   True if success or false if fail
 *
 ****************************************************************************/
unsafe extern "C" fn bl_os_mutex_lock(mutex: *mut crate::binary::c_types::c_void) -> i32 {
    log!("mutex lock");
    pthread_mutex_lock(mutex as *mut u8);
    1
}

/****************************************************************************
 * Name: bl_os_mutex_unlock
 *
 * Description:
 *   Lock mutex
 *
 * Input Parameters:
 *   mutex_data - mutex data pointer
 *
 * Returned Value:
 *   True if success or false if fail
 *
 ****************************************************************************/
unsafe extern "C" fn bl_os_mutex_unlock(mutex: *mut crate::binary::c_types::c_void) -> i32 {
    log!("mutex unlock");
    pthread_mutex_unlock(mutex as *mut u8);
    1
}

/****************************************************************************
 * Name: bl_os_workqueue_create
 *
 * Description:
 *
 * Input Parameters:
 *
 * Returned Value:
 *
 ****************************************************************************/
unsafe extern "C" fn bl_os_queue_create(
    queue_len: u32,
    item_size: u32,
) -> *mut crate::binary::c_types::c_void {
    log!("queue_create len={} item_size={}", queue_len, item_size);

    let res = riscv::interrupt::free(|_| {
        let mut res = 0xffff;
        for (i, sem) in MESSAGE_QUEUES.iter().enumerate() {
            if let None = *sem {
                res = i;
                break;
            }
        }

        if res == 0xffff {
            panic!("No more messafe queues available");
        }

        MESSAGE_QUEUES[res] = Some(SimpleQueue::new());

        res
    });

    res as *mut crate::binary::c_types::c_void
}

struct MqMessage {
    data: [u8; 256],
    len: usize,
}

static mut MESSAGE_QUEUES: [Option<SimpleQueue<MqMessage>>; 2] = [None, None];

/****************************************************************************
 * Name: bl_os_mq_delete
 *
 * Description:
 *
 * Input Parameters:
 *
 * Returned Value:
 *
 ****************************************************************************/
unsafe extern "C" fn bl_os_queue_delete(_queue: *mut crate::binary::c_types::c_void) {
    unimplemented!("queue_delete")
}

/****************************************************************************
 * Name: bl_os_mq_send_generic
 *
 * Description:
 *   Generic send message to queue within a certain period of time
 *
 * Input Parameters:
 *   queue - Message queue data pointer
 *   item  - Message data pointer
 *   ticks - Wait ticks
 *   prio  - Message priority
 *
 * Returned Value:
 *   True if success or false if fail
 *
 ****************************************************************************/
unsafe extern "C" fn bl_os_queue_send(
    queue: *mut crate::binary::c_types::c_void,
    item: *mut crate::binary::c_types::c_void,
    len: u32,
) -> crate::binary::c_types::c_int {
    log!("queue_send {:p} {:p} {} {}", queue, item, len, 0,);

    let message = item as *const u8;
    let queue = queue as usize;
    let success = riscv::interrupt::free(|_| {
        if let Some(ref mut queue) = MESSAGE_QUEUES[queue] {
            let mut data = [0u8; 256];
            for i in 0..len as usize {
                data[i] = *(message.offset(i as isize));
            }

            let msg = MqMessage {
                data,
                len: len as usize,
            };

            queue.enqueue(msg)
        } else {
            false
        }
    });

    if success {
        1
    } else {
        0
    }
}

/****************************************************************************
 * Name: bl_os_mq_recv
 *
 * Description:
 *   Receive message from queue within a certain period of time
 *
 * Input Parameters:
 *   queue - Message queue data pointer
 *   item  - Message data pointer
 *   ticks - Wait ticks
 *
 * Returned Value:
 *   True if success or false if fail
 *
 ****************************************************************************/
unsafe extern "C" fn bl_os_queue_recv(
    queue: *mut crate::binary::c_types::c_void,
    item: *mut crate::binary::c_types::c_void,
    len: u32,
    tick: u32,
) -> crate::binary::c_types::c_int {
    log!("queue recv {:p} {:p} {} {}", queue, item, len, tick);

    // TODO implement waiting
    let queue = queue as usize;
    let res = riscv::interrupt::free(|_| {
        if let Some(ref mut queue) = MESSAGE_QUEUES[queue] {
            queue.dequeue()
        } else {
            None
        }
    });

    let mut received_bytes: i32 = 0;
    let msg = item as *mut u8;

    match res {
        core::option::Option::Some(message) => {
            for i in 0..message.len {
                *(msg.offset(i as isize)) = message.data[i];
            }

            log!("copied message with len {}", message.len);

            received_bytes = message.len as i32;
        }
        core::option::Option::None => {}
    };

    log!("queue recv - received bytes: {}", received_bytes);

    if received_bytes > 0 {
        1
    } else {
        0
    }
}

/****************************************************************************
 * Name: bl_os_malloc
 *
 * Description:
 *   Allocate a block of memory
 *
 * Input Parameters:
 *   size - memory size
 *
 * Returned Value:
 *   Memory pointer
 *
 ****************************************************************************/
unsafe extern "C" fn bl_os_malloc(
    size: crate::binary::c_types::c_uint,
) -> *mut crate::binary::c_types::c_void {
    log!("malloc {}", size);
    malloc(size) as *mut crate::binary::c_types::c_void
}

/****************************************************************************
 * Name: bl_os_free
 *
 * Description:
 *   Free a block of memory
 *
 * Input Parameters:
 *   ptr - memory block
 *
 * Returned Value:
 *   No
 *
 ****************************************************************************/
unsafe extern "C" fn bl_os_free(p: *mut crate::binary::c_types::c_void) {
    log!("free {:p}", p);
    free(p as *const u8);
}

/****************************************************************************
 * Name: bl_os_zalloc
 *
 * Description:
 *   Allocate a block of memory
 *
 * Input Parameters:
 *   size - memory size
 *
 * Returned Value:
 *   Memory pointer
 *
 ****************************************************************************/
pub unsafe extern "C" fn bl_os_zalloc(
    size: crate::binary::c_types::c_uint,
) -> *mut crate::binary::c_types::c_void {
    log!("zalloc {}", size);
    let res = malloc(size) as *mut crate::binary::c_types::c_void;
    for i in 0..size {
        (res as *mut u8).offset(i as isize).write_volatile(0);
    }

    res
}

/****************************************************************************
 * Name: bl_os_clock_gettime_ms
 *
 * Description:
 *
 * Input Parameters:
 *
 * Returned Value:
 *
 ****************************************************************************/
unsafe extern "C" fn bl_os_get_time_ms() -> u64 {
    get_time().0 as u64
}

unsafe extern "C" fn bl_os_assert(
    _file: *const crate::binary::c_types::c_char,
    _line: crate::binary::c_types::c_int,
    _func: *const crate::binary::c_types::c_char,
    _expr: *const crate::binary::c_types::c_char,
) {
    unimplemented!()
}

unsafe extern "C" fn bl_os_event_group_create() -> *mut crate::binary::c_types::c_void {
    unimplemented!()
}

unsafe extern "C" fn bl_os_event_group_delete(_event: *mut crate::binary::c_types::c_void) {
    unimplemented!()
}

unsafe extern "C" fn bl_os_event_group_send(
    _event: *mut crate::binary::c_types::c_void,
    _bits: u32,
) -> u32 {
    unimplemented!()
}

unsafe extern "C" fn bl_os_event_group_wait(
    _event: *mut crate::binary::c_types::c_void,
    _bits_to_wait_for: u32,
    _clear_on_exit: crate::binary::c_types::c_int,
    _wait_for_all_bits: crate::binary::c_types::c_int,
    _block_time_tick: u32,
) -> u32 {
    unimplemented!()
}

unsafe extern "C" fn bl_os_event_register(
    _type_: crate::binary::c_types::c_int,
    _cb: *mut crate::binary::c_types::c_void,
    _arg: *mut crate::binary::c_types::c_void,
) -> crate::binary::c_types::c_int {
    unimplemented!()
}

unsafe extern "C" fn bl_os_task_create(
    _name: *const crate::binary::c_types::c_char,
    _entry: *mut crate::binary::c_types::c_void,
    _stack_depth: u32,
    _param: *mut crate::binary::c_types::c_void,
    _prio: u32,
    _task_handle: *mut crate::binary::c_types::c_void,
) -> crate::binary::c_types::c_int {
    unimplemented!()
}

unsafe extern "C" fn bl_os_task_delete(_task_handle: *mut crate::binary::c_types::c_void) {
    unimplemented!()
}

unsafe extern "C" fn bl_os_task_get_current_task() -> *mut crate::binary::c_types::c_void {
    unimplemented!()
}

unsafe extern "C" fn bl_os_task_notify_create() -> *mut crate::binary::c_types::c_void {
    unimplemented!()
}

unsafe extern "C" fn bl_os_task_notify(_task_handle: *mut crate::binary::c_types::c_void) {
    unimplemented!()
}

unsafe extern "C" fn bl_os_task_wait(
    _task_handle: *mut crate::binary::c_types::c_void,
    _tick: u32,
) {
    unimplemented!()
}

unsafe extern "C" fn bl_os_queue_send_wait(
    _queue: *mut crate::binary::c_types::c_void,
    _item: *mut crate::binary::c_types::c_void,
    _len: u32,
    _ticks: u32,
    _prio: crate::binary::c_types::c_int,
) -> crate::binary::c_types::c_int {
    unimplemented!()
}

unsafe extern "C" fn bl_os_get_tick() -> u32 {
    get_time().0
}

unsafe extern "C" fn bl_os_log_write(
    level: u32,
    tag: *const crate::binary::c_types::c_char,
    file: *const crate::binary::c_types::c_char,
    line: crate::binary::c_types::c_int,
    format: *const crate::binary::c_types::c_char,
    args: ...
) {
    let tag = if tag.is_null() {
        StrBuf::new()
    } else {
        StrBuf::from(tag)
    };
    let file = StrBuf::from(file);

    let mut buf = [0u8; 512];
    vsnprintf(&mut buf as *mut u8, 511, format, args);
    let res_str = StrBuf::from(&buf as *const u8);
    print!("{}", res_str.as_str_ref());

    println!(
        "{} {} {}:{} {}",
        level,
        tag.as_str_ref(),
        file.as_str_ref(),
        line,
        res_str.as_str_ref(),
    );
}
