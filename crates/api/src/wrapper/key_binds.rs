use std::ffi::CStr;

use log_err::LogErrOption;

use crate::AddonAPI;

use super::{AddonApiWrapper, Cleanup};

type KeybindsProcess =
    unsafe extern "C" fn(aIdentifier: *const ::std::os::raw::c_char, aIsRelease: bool);

impl AddonApiWrapper<'_> {
    pub unsafe fn register_key_binding(
        &mut self,
        id: &'static CStr,
        callback: KeybindsProcess,
        binding: Option<&CStr>,
    ) {
        self.cleanups
            .push(Box::new(KeyBindWrapper::new(&self, id, callback, binding)));
    }
}

struct KeyBindWrapper(&'static CStr);

impl KeyBindWrapper {
    unsafe fn new(
        api: &AddonAPI,
        id: &'static CStr,
        callback: KeybindsProcess,
        binding: Option<&CStr>,
    ) -> Self {
        api.InputBinds
            .RegisterWithString
            .log_expect("cannot register key binding")(
            id.as_ptr(),
            Some(callback),
            binding.unwrap_or(c"(none)").as_ptr(),
        );

        Self(id)
    }
}

impl Cleanup for KeyBindWrapper {
    unsafe fn cleanup(&mut self, api: &AddonAPI) {
        api.InputBinds
            .Deregister
            .log_expect("cannot unregister key binding")(self.0.as_ptr());
    }
}
