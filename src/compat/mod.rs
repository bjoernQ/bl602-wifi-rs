use embedded_time::duration::Milliseconds;

pub mod bl602;
pub mod common;
pub mod malloc;
pub mod queue;
pub mod work_queue;

static mut TIME_SOURCE: Option<fn() -> Milliseconds> = None;

pub fn set_time_source(time_source: fn() -> Milliseconds) {
    unsafe {
        TIME_SOURCE = Some(time_source);
    }
}

pub fn get_time() -> Milliseconds {
    unsafe {
        match TIME_SOURCE {
            Some(time_source) => time_source(),
            None => panic!("TIME_SOURCE is none"),
        }
    }
}
