use std::ffi::c_void;

use paths_core::state::update_mumble_identity;

use crate::globals::RENDER_CONFIG;

pub unsafe extern "C" fn identity_updated_cb(identity: *mut c_void) {
    let identity = std::mem::transmute::<*mut c_void, &api::Mumble_Identity>(identity);

    RENDER_CONFIG
        .assume_init_mut()
        .update_ui_size(identity.UISize);

    update_mumble_identity(identity);
}
