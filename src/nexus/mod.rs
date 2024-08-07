use std::ffi::c_short;

use api::ERenderType_ERenderType_Render;

use crate::{
    logger::get_logger,
    state::{get_api, get_nexus_link, initialize_global_state},
};

pub(crate) mod api;

#[no_mangle]
extern "C" fn GetAddonDef() -> *const api::AddonDefinition {
    let def = api::AddonDefinition {
        Signature: -252,
        APIVersion: 6,
        Name: c"Paths".as_ptr(),
        Version: api::AddonVersion {
            Major: parse_version_part(env!("CARGO_PKG_VERSION_MAJOR")),
            Minor: parse_version_part(env!("CARGO_PKG_VERSION_MINOR")),
            Build: parse_version_part(env!("CARGO_PKG_VERSION_PATCH")),
            Revision: 0,
        },
        Author: c"fehnomenal".as_ptr(),
        Description: c"Displays paths".as_ptr(),
        Load: Some(load),
        Unload: Some(unload),
        Flags: api::EAddonFlags_EAddonFlags_None,
        Provider: api::EUpdateProvider_EUpdateProvider_GitHub,
        UpdateLink: c"https://github.com/fehnomenal/gw2-nexus-paths".as_ptr(),
    };

    Box::into_raw(Box::new(def))
}

extern "C" fn load(api: *mut api::AddonAPI) {
    if api.is_null() {
        // TODO: Can we do anything here?
        return;
    }

    let api = unsafe { &*api };
    initialize_global_state(api);

    let logger = get_logger();

    logger.info("Hello from the paths addon :-)");
    logger.critical(&format!("Und jetzt mit Werten: {}, {:?}", 123, 456));
    logger.debug("Und mit NUll Byte \0 secret message, harharhar");

    dbg!(api.SwapChain, api.SwapChain.is_null());

    if let Some(on) = api.Renderer.Register {
        unsafe { on(ERenderType_ERenderType_Render, Some(render_cb)) }
    }
}

extern "C" fn unload() {
    let logger = get_logger();

    logger.info("Bye bye");

    if let Some(off) = get_api().Renderer.Deregister {
        unsafe { off(Some(render_cb)) }
    }
}

const fn parse_version_part(s: &str) -> c_short {
    let mut out: c_short = 0;
    let mut i: usize = 0;
    while i < s.len() {
        out *= 10;
        out += (s.as_bytes()[i] - b'0') as c_short;
        i += 1;
    }
    out
}

static mut IS_MOVING: bool = false;

extern "C" fn render_cb() {
    let dl = get_nexus_link();

    if unsafe { IS_MOVING } != dl.IsMoving {
        unsafe { IS_MOVING = dl.IsMoving };

        let logger = get_logger();

        logger.info(&format!(
            "{} moving.",
            if dl.IsMoving { "Now" } else { "No longer" }
        ));
    }
}
