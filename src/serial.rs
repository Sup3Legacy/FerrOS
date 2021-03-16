use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    SERIAL1
        .lock()
        .write_fmt(args)
        .expect("Printing to serial failed");
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
    $crate::serial::_print(format_args!($($arg)*));
    };
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! debugln {
    () => ($crate::debug!("Info\n"));
    ($fmt:expr) => ($crate::debug!(concat!("Info: ", $fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::debug!(
        concat!("Info: ", $fmt, "\n"), $($arg)*));
}

#[macro_export]
macro_rules! warningln {
    () => ($crate::debug!("\x1B[33mWarning \x1B[0m\n"));
    ($fmt:expr) => ($crate::debug!(concat!("\x1B[33mWarning: ", $fmt, "\x1B[0m\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::debug!(
        concat!("\x1B[33mWarning: ", $fmt, "\n"), $($arg)*));
}

#[macro_export]
macro_rules! errorln {
    () => ($crate::debug!("\x1B[91mERROR \x1B[0m\n"));
    ($fmt:expr) => ($crate::debug!(concat!("\x1B[91mERROR: ", $fmt, "\x1B[0m\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::debug!(
        concat!("\x1B[91mERROR: ", $fmt, "\n"), $($arg)*));
}

#[macro_export]
macro_rules! initdebugln {
    () => ($crate::debug!("\n ===== FerrOS debug interface =====\n"));
}
