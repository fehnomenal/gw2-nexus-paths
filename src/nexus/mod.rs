use std::{
    ffi::{c_short, c_void, CStr},
    mem::{transmute, MaybeUninit},
};

use egui::{Context, Visuals};
use windows::{core::Interface, Win32};

use crate::{
    render::{ui::manager::InputManager, RenderConfig, Renderer},
    state::{
        clear_global_state, get_mumble_link, get_mut_api, get_nexus_link, initialize_global_state,
        update_mumble_identity,
    },
};

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

const EV_MUMBLE_IDENTITY_UPDATED: &CStr = c"EV_MUMBLE_IDENTITY_UPDATED";
const EV_WINDOW_RESIZED: &CStr = c"EV_WINDOW_RESIZED";

const KB_TOGGLE_OPTIONS_ID: &CStr = c"KB_TOGGLE_OPTIONS";

const QA_SHORTCUT_ID: &CStr = c"QA_SHORTCUT";

const TEX_SHORTCUT_ID: &CStr = c"TEX_SHORTCUT_ICON";

unsafe extern "C" fn load(api: *mut api::AddonAPI) {
    if api.is_null() {
        // TODO: Can we do anything here?
        return;
    }

    let api = &*api;

    if let Some(swap_chain) =
        Win32::Graphics::Dxgi::IDXGISwapChain::from_raw_borrowed(&api.SwapChain)
    {
        initialize_global_state(*api);

        let nexus_link = get_nexus_link();

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

        let api = get_mut_api();

        api.register_render(render_cb);

        api.subscribe_event(EV_MUMBLE_IDENTITY_UPDATED, identity_updated_cb);
        api.subscribe_event(EV_WINDOW_RESIZED, window_resized_cb);

        api.register_wnd_proc(wnd_proc);

        // https://render.guildwars2.com/file/25B230711176AB5728E86F5FC5F0BFAE48B32F6E/97461.png
        api.load_texture_from_url(
            TEX_SHORTCUT_ID,
            c"https://render.guildwars2.com",
            c"/file/25B230711176AB5728E86F5FC5F0BFAE48B32F6E/97461.png",
            None,
        );

        api.register_key_binding(KB_TOGGLE_OPTIONS_ID, toggle_options, None);

        api.register_shortcut(
            QA_SHORTCUT_ID,
            TEX_SHORTCUT_ID,
            None,
            KB_TOGGLE_OPTIONS_ID,
            c"Markers and paths",
        );
    }
}

static mut RENDERER: MaybeUninit<Renderer<'static>> = MaybeUninit::uninit();
static mut RENDER_CONFIG: MaybeUninit<RenderConfig> = MaybeUninit::uninit();
static mut UI_INPUT_MANAGER: MaybeUninit<InputManager> = MaybeUninit::uninit();
static mut IS_UI_VISIBLE: bool = false;

unsafe extern "C" fn unload() {
    RENDER_CONFIG.assume_init_drop();
    RENDERER.assume_init_drop();
    UI_INPUT_MANAGER.assume_init_drop();

    clear_global_state();
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
    let identity = transmute::<*mut c_void, &api::Mumble_Identity>(identity);

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

    if IS_UI_VISIBLE {
        renderer.render_ui(UI_INPUT_MANAGER.assume_init_mut());
    }

    if !get_nexus_link().IsGameplay {
        return;
    }

    if get_mumble_link().Context.IsMapOpen() == 0 {
        renderer.render_world();
    }

    renderer.render_map();
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

unsafe extern "C" fn toggle_options(_identifier: *const i8, _is_pressed: bool) {
    IS_UI_VISIBLE = !IS_UI_VISIBLE;
}
