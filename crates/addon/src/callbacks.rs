use std::ffi::c_void;

use crate::state::{
    handle_wnd_proc, render, toggle_ui_visible, update_mumble_identity, update_window_size,
};

pub unsafe extern "C" fn identity_updated_cb(identity: *mut c_void) {
    update_mumble_identity(std::mem::transmute(identity));
}

pub unsafe extern "C" fn render_cb() {
    render();
}

pub unsafe extern "C" fn toggle_ui_cb(_identifier: *const i8, _is_pressed: bool) {
    toggle_ui_visible();
}

pub unsafe extern "C" fn window_resized_cb(_payload: *mut c_void) {
    update_window_size();
}

pub unsafe extern "C" fn wnd_proc_cb(
    _window: api::HWND,
    msg: api::UINT,
    w_param: api::WPARAM,
    l_param: api::LPARAM,
) -> u32 {
    handle_wnd_proc(msg, w_param, l_param)
}
