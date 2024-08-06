use std::ffi::c_short;

use api::AddonAPI;

mod api;

#[no_mangle]
extern "C" fn GetAddonDef() -> api::AddonDefinition {
    api::AddonDefinition {
        Signature: -252,
        APIVersion: 0,
        Name: c"Paths".as_ptr(),
        Version: api::AddonVersion {
            Major: parse_version_part(env!("CARGO_PKG_VERSION_MAJOR")),
            Minor: parse_version_part(env!("CARGO_PKG_VERSION_MINOR")),
            Build: parse_version_part(env!("CARGO_PKG_VERSION_PATCH")),
            Revision: 0,
        },
        Author: c"fehnomenal".as_ptr(),
        Description: c"Displays paths".as_ptr(),
        Load: load,
        Unload: unload,
        Flags: api::EAddonFlags::None,
        Provider: api::EUpdateProvider::GitHub,
        UpdateLink: c"https://github.com/fehnomenal/gw2-nexus-paths".as_ptr(),
    }
}

extern "C" fn load(_api: &AddonAPI) {}

extern "C" fn unload() {}

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
