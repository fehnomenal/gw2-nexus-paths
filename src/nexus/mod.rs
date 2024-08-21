use std::{
    ffi::{c_short, c_void, CString},
    mem::{transmute, MaybeUninit},
};

use egui::{Context, Visuals};
use windows::{core::Interface, Win32};

use crate::{
    render::{ui::manager::InputManager, RenderConfig, Renderer},
    state::{
        clear_global_state, get_api, get_nexus_link, initialize_global_state,
        update_mumble_identity,
    },
};

pub mod api;
mod events;

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
        Win32::Graphics::Dxgi::IDXGISwapChain::from_raw_borrowed(&api.SwapChain)
    {
        let nexus_link = initialize_global_state(api);

        let egui_context = Context::default();
        egui_context.set_visuals(Visuals::light());

        UI_INPUT_MANAGER.write(InputManager::new(&egui_context));

        RENDER_CONFIG.write(RenderConfig::new(
            nexus_link.Width as f32,
            nexus_link.Height as f32,
        ));

        RENDERER.write(Renderer::new(
            RENDER_CONFIG.assume_init_ref(),
            swap_chain,
            &egui_context,
        ));

        api.register_render(render_cb);

        api.subscribe_event("EV_MUMBLE_IDENTITY_UPDATED", identity_updated_cb);
        api.subscribe_event("EV_WINDOW_RESIZED", window_resized_cb);

        api.register_wnd_proc(wnd_proc);
    }
}

static mut RENDERER: MaybeUninit<Renderer<'static>> = MaybeUninit::uninit();
static mut RENDER_CONFIG: MaybeUninit<RenderConfig> = MaybeUninit::uninit();
static mut UI_INPUT_MANAGER: MaybeUninit<InputManager> = MaybeUninit::uninit();

extern "C" fn unload() {
    unsafe {
        let api = get_api();

        api.unregister_wnd_proc(wnd_proc);

        api.unsubscribe_event("EV_WINDOW_RESIZED", window_resized_cb);
        api.unsubscribe_event("EV_MUMBLE_IDENTITY_UPDATED", identity_updated_cb);

        api.unregister_render(render_cb);

        RENDER_CONFIG.assume_init_drop();
        RENDERER.assume_init_drop();
        UI_INPUT_MANAGER.assume_init_drop();

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

unsafe extern "C" fn identity_updated_cb(identity: *mut c_void) {
    let identity = transmute::<*mut c_void, &api::mumble::Mumble_Identity>(identity);

    RENDER_CONFIG
        .assume_init_mut()
        .update_ui_size(identity.UISize);

    update_mumble_identity(identity);
}

unsafe extern "C" fn window_resized_cb(_payload: *mut c_void) {
    let nexus_link = get_nexus_link();

    RENDERER.assume_init_mut().rebuild_render_targets();

    RENDER_CONFIG
        .assume_init_mut()
        .update_screen_size(nexus_link.Width as f32, nexus_link.Height as f32);
}

unsafe extern "C" fn render_cb() {
    let renderer = RENDERER.assume_init_mut();

    renderer.render_world();
    renderer.render_map();
    renderer.render_ui(UI_INPUT_MANAGER.assume_init_mut());
}

unsafe extern "C" fn wnd_proc(
    _window: api::HWND,
    msg: api::UINT,
    w_param: api::WPARAM,
    l_param: api::LPARAM,
) -> u32 {
    let handled = UI_INPUT_MANAGER
        .assume_init_mut()
        .handle_wnd_proc(msg, w_param, l_param);

    if handled {
        0
    } else {
        1
    }
}
