pub const LOG: bool = false;

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        if crate::log::LOG {
            write!(crate::get_serial(), $($arg)*).ok();
            write!(crate::get_serial(), "\r\n").ok();
        }
    };
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        write!(crate::get_serial(), $($arg)*).ok();
        write!(crate::get_serial(), "\r\n").ok();
    };
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        write!(crate::get_serial(), $($arg)*).ok();
    };
}
