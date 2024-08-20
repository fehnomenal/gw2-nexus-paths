use std::ffi::{c_char, CString};

use crate::{
    nexus::api::{
        ELogLevel, ELogLevel_ELogLevel_CRITICAL, ELogLevel_ELogLevel_DEBUG,
        ELogLevel_ELogLevel_INFO, ELogLevel_ELogLevel_TRACE, ELogLevel_ELogLevel_WARNING,
        LOGGER_LOG2,
    },
    state::get_api,
};

#[allow(dead_code)]
pub fn get_logger() -> Logger {
    create_logger(unsafe { get_api().Log })
}

pub type Logger = Box<dyn LoggerTrait>;

#[allow(dead_code)]
pub trait LoggerTrait {
    fn critical(&self, msg: &str);
    fn warning(&self, msg: &str);
    fn info(&self, msg: &str);
    fn debug(&self, msg: &str);
    fn trace(&self, msg: &str);
}

fn create_logger(base: LOGGER_LOG2) -> Logger {
    if let Some(log) = base {
        Box::new(LoggerImpl(log))
    } else {
        Box::new(NoopLogger)
    }
}

struct NoopLogger;
impl LoggerTrait for NoopLogger {
    fn critical(&self, _msg: &str) {}
    fn warning(&self, _msg: &str) {}
    fn info(&self, _msg: &str) {}
    fn debug(&self, _msg: &str) {}
    fn trace(&self, _msg: &str) {}
}

struct LoggerImpl(
    // We need to copy the unwrapped type here...
    unsafe extern "C" fn(aLogLevel: ELogLevel, aChannel: *const c_char, aStr: *const c_char),
);

impl LoggerTrait for LoggerImpl {
    fn critical(&self, msg: &str) {
        self.log(ELogLevel_ELogLevel_CRITICAL, msg)
    }
    fn warning(&self, msg: &str) {
        self.log(ELogLevel_ELogLevel_WARNING, msg)
    }
    fn info(&self, msg: &str) {
        self.log(ELogLevel_ELogLevel_INFO, msg)
    }
    fn debug(&self, msg: &str) {
        self.log(ELogLevel_ELogLevel_DEBUG, msg)
    }
    fn trace(&self, msg: &str) {
        self.log(ELogLevel_ELogLevel_TRACE, msg)
    }
}

impl LoggerImpl {
    #[allow(dead_code)]
    fn log(&self, level: ELogLevel, msg: &str) {
        let c_msg = CString::new(msg);

        match c_msg {
            Ok(msg) => {
                unsafe { self.0(level, c"Paths".as_ptr(), msg.as_ptr()) };
            }
            Err(err) => {
                let safe = &msg[0..err.nul_position()];

                let msg = format!("The next message was truncated at pos {} because of a NULL byte. Original length: {}", err.nul_position(), msg.len());
                self.log(ELogLevel_ELogLevel_WARNING, &msg);

                self.log(level, safe);
            }
        }
    }
}
