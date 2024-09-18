use crate::globals::IS_UI_VISIBLE;

pub unsafe extern "C" fn toggle_ui_cb(_identifier: *const i8, _is_pressed: bool) {
    IS_UI_VISIBLE = !IS_UI_VISIBLE;
}
