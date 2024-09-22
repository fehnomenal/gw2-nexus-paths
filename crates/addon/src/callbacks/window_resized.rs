use std::ffi::c_void;

use crate::state::{get_nexus_link, get_renderer};

pub unsafe extern "C" fn window_resized_cb(_payload: *mut c_void) {
    let nexus_link = get_nexus_link();

    let renderer = get_renderer();

    renderer.rebuild_render_targets();

    renderer
        .config
        .borrow_mut()
        .update_screen_size(nexus_link.Width as f32, nexus_link.Height as f32);
}
