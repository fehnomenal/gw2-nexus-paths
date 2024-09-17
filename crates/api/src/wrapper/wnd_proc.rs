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
            .expect("Cannot register wnd proc handler")(Some(callback));

        Self(callback)
    }
}

impl Cleanup for WndProcWrapper {
    unsafe fn cleanup(&mut self, api: &AddonAPI) {
        api.WndProc
            .Deregister
            .expect("Cannot unregister wnd proc handler")(Some(self.0));
    }
}
