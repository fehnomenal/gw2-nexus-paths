use std::ffi::c_short;

use crate::{load, unload};

#[no_mangle]
extern "C" fn GetAddonDef() -> *const api::AddonDefinition {
    Box::into_raw(Box::new(ADDON_DEFINITION))
}

pub const ADDON_DEFINITION: api::AddonDefinition = api::AddonDefinition {
    Signature: -252,
    APIVersion: 6,
    Name: c"Paths".as_ptr(),
    Version: api::AddonVersion {
        Major: parse_integer(env!("CARGO_PKG_VERSION_MAJOR")),
        Minor: parse_integer(env!("CARGO_PKG_VERSION_MINOR")),
        Build: parse_integer(env!("CARGO_PKG_VERSION_PATCH")),
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

const fn parse_integer(s: &str) -> c_short {
    let mut out: c_short = 0;
    let mut i: usize = 0;

    while i < s.len() {
        out *= 10;
        out += (s.as_bytes()[i] - b'0') as c_short;
        i += 1;
    }

    out
}
