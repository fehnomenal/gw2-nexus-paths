use std::ffi::CStr;

use log_err::LogErrOption;

use crate::AddonAPI;

use super::{AddonApiWrapper, Cleanup};

impl AddonApiWrapper {
    pub unsafe fn register_shortcut(
        &mut self,
        id: &'static CStr,
        texture_id: &CStr,
        hover_texture_id: Option<&CStr>,
        keybinding_id: &CStr,
        tooltip: &CStr,
    ) {
        self.cleanups.push(Box::new(ShortcutWrapper::new(
            &self,
            id,
            texture_id,
            hover_texture_id,
            keybinding_id,
            tooltip,
        )));
    }
}

struct ShortcutWrapper(&'static CStr);

impl ShortcutWrapper {
    unsafe fn new(
        api: &AddonAPI,
        id: &'static CStr,
        texture_id: &CStr,
        hover_texture_id: Option<&CStr>,
        keybinding_id: &CStr,
        tooltip: &CStr,
    ) -> Self {
        api.QuickAccess.Add.log_expect("cannot register shortcut")(
            id.as_ptr(),
            texture_id.as_ptr(),
            hover_texture_id.unwrap_or(texture_id).as_ptr(),
            keybinding_id.as_ptr(),
            tooltip.as_ptr(),
        );

        Self(id)
    }
}

impl Cleanup for ShortcutWrapper {
    unsafe fn cleanup(&mut self, api: &AddonAPI) {
        api.QuickAccess
            .Remove
            .log_expect("cannot unregister shortcut")(self.0.as_ptr());
    }
}
