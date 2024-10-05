use crate::state::ui_state;

pub unsafe extern "C" fn toggle_ui_cb(_identifier: *const i8, _is_pressed: bool) {
    let ui_state = ui_state();

    // The UI is not displayed initially, so the first click will definitely display it.
    ui_state.ui_was_displayed_once = true;

    ui_state.main_window_open = !ui_state.main_window_open
}
