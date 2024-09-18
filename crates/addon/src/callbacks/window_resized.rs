use std::ffi::c_void;

use paths_core::state::get_nexus_link;

use crate::globals::{RENDERER, RENDER_CONFIG};

pub unsafe extern "C" fn window_resized_cb(_payload: *mut c_void) {
    let nexus_link = get_nexus_link();

    RENDERER.assume_init_mut().rebuild_render_targets();

    RENDER_CONFIG
        .assume_init_mut()
        .update_screen_size(nexus_link.Width as f32, nexus_link.Height as f32);
}
