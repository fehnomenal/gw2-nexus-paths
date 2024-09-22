use crate::state::get_input_manager;

pub unsafe extern "C" fn wnd_proc_cb(
    _window: api::HWND,
    msg: api::UINT,
    w_param: api::WPARAM,
    l_param: api::LPARAM,
) -> u32 {
    let handled = get_input_manager().handle_wnd_proc(msg, w_param, l_param);

    if handled {
        0
    } else {
        1
    }
}
