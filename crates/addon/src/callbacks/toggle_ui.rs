use crate::state::toggle_ui_visible;

pub unsafe extern "C" fn toggle_ui_cb(_identifier: *const i8, _is_pressed: bool) {
    toggle_ui_visible();
}
