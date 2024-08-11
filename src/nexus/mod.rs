use std::{
    ffi::{c_short, c_void},
    mem::MaybeUninit,
};

use api::ERenderType_ERenderType_Render;
use windows::{core::Interface, Win32};

use crate::{
    render::Renderer,
    state::{clear_global_state, get_api, get_nexus_link, initialize_global_state},
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

unsafe extern "C" fn load(api: *mut api::AddonAPI) {
    if api.is_null() {
        // TODO: Can we do anything here?
        return;
    }

    let api = &*api;

    if let Some(swap_chain) =
        Win32::Graphics::Dxgi::IDXGISwapChain4::from_raw_borrowed(&api.SwapChain)
    {
        initialize_global_state(api);

        RENDERER.write({
            let dev = swap_chain
                .GetDevice::<Win32::Graphics::Direct3D11::ID3D11Device>()
                .expect("Could not get d3d11 device");

            let link = get_nexus_link();
            let mut renderer = Renderer::new(dev, &swap_chain).expect("Could not create renderer");
            renderer.update_screen_size(link.Width as f32, link.Height as f32);

            renderer
        });

        api.Renderer.Register.unwrap()(ERenderType_ERenderType_Render, Some(render_cb));

        api.Events.Subscribe.unwrap()(c"EV_WINDOW_RESIZED".as_ptr(), Some(windows_resized_cb));
    }
}

static mut RENDERER: MaybeUninit<Renderer> = MaybeUninit::uninit();

extern "C" fn unload() {
    unsafe {
        let api = get_api();

        api.Events.Unsubscribe.unwrap()(c"EV_WINDOW_RESIZED".as_ptr(), Some(windows_resized_cb));

        api.Renderer.Deregister.unwrap()(Some(render_cb));

        RENDERER.assume_init_drop();

        clear_global_state();
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

unsafe extern "C" fn render_cb() {
    RENDERER.assume_init_mut().render();
}

unsafe extern "C" fn windows_resized_cb(_payload: *mut c_void) {
    let link = get_nexus_link();

    RENDERER
        .assume_init_mut()
        .update_screen_size(link.Width as f32, link.Height as f32);
}
