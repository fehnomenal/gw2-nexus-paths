use std::{backtrace::Backtrace, panic::PanicInfo, thread};

use log::error;

pub fn panic_hook(info: &PanicInfo) {
    // The current implementation always returns `Some`.
    let location = info.location().unwrap();

    let msg = match info.payload().downcast_ref::<&'static str>() {
        Some(s) => *s,
        None => match info.payload().downcast_ref::<String>() {
            Some(s) => &s[..],
            None => "Box<dyn Any>",
        },
    };

    let thread = thread::current();
    let name = thread.name().unwrap_or("<unnamed>");

    error!(
        "thread '{name}' panicked at {location}:\n{msg}\n{}",
        Backtrace::force_capture(),
    );
}
