use std::ffi::c_void;

use crate::state::{get_renderer, update_mumble_identity};

pub unsafe extern "C" fn identity_updated_cb(identity: *mut c_void) {
    let identity = std::mem::transmute::<*mut c_void, &api::Mumble_Identity>(identity);

    get_renderer()
        .config
        .borrow_mut()
        .update_ui_size(identity.UISize);

    update_mumble_identity(identity);
}
