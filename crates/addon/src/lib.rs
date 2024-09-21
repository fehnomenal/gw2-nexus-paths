use callbacks::{identity_updated_cb, render_cb, toggle_ui_cb, window_resized_cb, wnd_proc_cb};
use constants::{
    EV_MUMBLE_IDENTITY_UPDATED, EV_WINDOW_RESIZED, KB_TOGGLE_UI_ID, QA_SHORTCUT_ID, TEX_SHORTCUT_ID,
};
use egui::{Context, Visuals};
use globals::{RENDERER, RENDER_CONFIG, UI_INPUT_MANAGER};
use input_manager::InputManager;
use panic::panic_hook;
use paths_renderer::{RenderConfig, Renderer};
use state::{clear_global_state, get_mut_api, get_nexus_link, initialize_global_state};
use windows::{core::Interface, Win32::Graphics::Dxgi::IDXGISwapChain};

mod addon_def;
mod callbacks;
mod constants;
mod globals;
mod input_manager;
mod logger;
mod panic;
mod state;

pub unsafe extern "C" fn load(api: *mut api::AddonAPI) {
    if api.is_null() {
        // TODO: Can we do anything here?
        return;
    }

    let api = &*api;

    if let Some(swap_chain) = IDXGISwapChain::from_raw_borrowed(&api.SwapChain) {
        initialize_global_state(*api);

        std::panic::set_hook(Box::new(panic_hook));

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

        api.register_wnd_proc(wnd_proc_cb);

        // https://render.guildwars2.com/file/25B230711176AB5728E86F5FC5F0BFAE48B32F6E/97461.png
        api.load_texture_from_url(
            TEX_SHORTCUT_ID,
            c"https://render.guildwars2.com",
            c"/file/25B230711176AB5728E86F5FC5F0BFAE48B32F6E/97461.png",
            None,
        );

        api.register_key_binding(KB_TOGGLE_UI_ID, toggle_ui_cb, None);

        api.register_shortcut(
            QA_SHORTCUT_ID,
            TEX_SHORTCUT_ID,
            None,
            KB_TOGGLE_UI_ID,
            c"Markers and paths",
        );
    }
}

pub unsafe extern "C" fn unload() {
    RENDER_CONFIG.assume_init_drop();
    RENDERER.assume_init_drop();
    UI_INPUT_MANAGER.assume_init_drop();

    clear_global_state();
}
