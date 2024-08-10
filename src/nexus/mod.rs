use std::ffi::c_short;

use api::ERenderType_ERenderType_Render;
use windows::{core::Interface, Win32};

use crate::{
    render::Renderer,
    state::{get_api, initialize_global_state},
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

    if let Some(on) = api.Renderer.Register {
        unsafe { on(ERenderType_ERenderType_Render, Some(render_cb)) }
    }

    unsafe {
        if let Some(swap_chain) =
            Win32::Graphics::Dxgi::IDXGISwapChain4::from_raw_borrowed(&api.SwapChain)
        {
            let dev = swap_chain
                .GetDevice::<Win32::Graphics::Direct3D11::ID3D11Device>()
                .expect("Could not get d3d11 device");

            RENDERER = Some(Renderer::new(dev, &swap_chain).expect("Could not create renderer"));
        }
    }
}

static mut RENDERER: Option<Renderer> = None;

extern "C" fn unload() {
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

extern "C" fn render_cb() {
    if let Some(renderer) = unsafe { RENDERER.as_mut() } {
        unsafe { renderer.render() };
    }
}
