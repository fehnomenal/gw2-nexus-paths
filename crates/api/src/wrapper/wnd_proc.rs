use log_err::LogErrOption;

use crate::{AddonAPI, HWND, LPARAM, UINT, WPARAM};

use super::{AddonApiWrapper, Cleanup};

type WndProcCallback =
    unsafe extern "C" fn(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> UINT;

impl AddonApiWrapper {
    pub unsafe fn register_wnd_proc(&mut self, callback: WndProcCallback) {
        self.cleanups
            .push(Box::new(WndProcWrapper::new(&self, callback)));
    }
}

struct WndProcWrapper(WndProcCallback);

impl WndProcWrapper {
    unsafe fn new(api: &AddonAPI, callback: WndProcCallback) -> Self {
        api.WndProc
            .Register
            .log_expect("cannot register wnd proc handler")(Some(callback));

        Self(callback)
    }
}

impl Cleanup for WndProcWrapper {
    unsafe fn cleanup(&mut self, api: &AddonAPI) {
        api.WndProc
            .Deregister
            .log_expect("cannot unregister wnd proc handler")(Some(self.0));
    }
}
