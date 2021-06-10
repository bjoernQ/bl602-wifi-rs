use core::fmt::Write;

pub const LOG: bool = false;

pub static mut WRITER: Option<fn() -> &'static mut dyn Write> = None;

pub fn set_writer(writer: fn() -> &'static mut dyn Write) {
    unsafe {
        WRITER.replace(writer);
    }
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        if $crate::log::LOG {
            #[allow(unused_unsafe)]
            unsafe {
                if let Some(writer) = $crate::log::WRITER {
                    let writer = writer();
                    write!(writer, $($arg)*).ok();
                    write!(writer, "\r\n").ok();
                }
            };
        }
    };
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        #[allow(unused_unsafe)]
        unsafe {
            if let Some(writer) = $crate::log::WRITER {
                let writer = writer();
                write!(writer, $($arg)*).ok();
                write!(writer, "\r\n").ok();
            }
        }
    };
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        #[allow(unused_unsafe)]
        unsafe {
            if let Some(writer) = $crate::log::WRITER {
                let writer = writer();
                write!(writer, $($arg)*).ok();
            }
        }
    };
}
