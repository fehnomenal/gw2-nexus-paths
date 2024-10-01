use std::ffi::CString;

use log::{Level, Log, Metadata, Record};

use crate::{
    AddonAPI, ELogLevel_ELogLevel_CRITICAL, ELogLevel_ELogLevel_DEBUG, ELogLevel_ELogLevel_INFO,
    ELogLevel_ELogLevel_TRACE, ELogLevel_ELogLevel_WARNING,
};

unsafe impl Send for AddonAPI {}
unsafe impl Sync for AddonAPI {}

impl Log for AddonAPI {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        if let Some(log) = self.Log {
            const CHANNEL: *const i8 = c"Paths".as_ptr();

            let level = match record.level() {
                Level::Error => ELogLevel_ELogLevel_CRITICAL,
                Level::Warn => ELogLevel_ELogLevel_WARNING,
                Level::Info => ELogLevel_ELogLevel_INFO,
                Level::Debug => ELogLevel_ELogLevel_DEBUG,
                Level::Trace => ELogLevel_ELogLevel_TRACE,
            };

            #[cfg(feature = "log_location")]
            let msg = {
                let file = record.file().or(record.file_static());
                let module_path = record.module_path().or(record.module_path_static());

                let location = match (file, module_path, record.line()) {
                    (Some(file), _, Some(line)) => Some(format!("{file}:{line}")),
                    (None, Some(module_path), _) => Some(module_path.to_owned()),
                    _ => None,
                };

                if let Some(location) = location {
                    format!("{}\n at {location}", record.args())
                } else {
                    format!("{}\n (unknown location)", record.args())
                }
            };

            #[cfg(not(feature = "log_location"))]
            let msg = format!("{}", record.args());

            match CString::new(msg) {
                Ok(msg) => unsafe {
                    log(level, CHANNEL, msg.as_ptr());
                },

                Err(err) => unsafe {
                    let orig_msg = format!("{}", record.args());

                    let msg = format!("The next message was truncated at pos {} because of a NULL byte. Original length: {}", err.nul_position(), orig_msg.len());
                    let msg = CString::new(msg).unwrap();
                    log(ELogLevel_ELogLevel_WARNING, CHANNEL, msg.as_ptr());

                    let safe = CString::new(&orig_msg[0..err.nul_position()]).unwrap();
                    log(level, CHANNEL, safe.as_ptr())
                },
            }
        }
    }

    fn flush(&self) {}
}
