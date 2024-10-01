mod addon_def;
mod callbacks;
mod constants;
mod input_manager;
mod panic;
mod state;

use callbacks::{identity_updated_cb, render_cb, toggle_ui_cb, window_resized_cb, wnd_proc_cb};
use constants::{
    EV_MUMBLE_IDENTITY_UPDATED, EV_WINDOW_RESIZED, KB_TOGGLE_UI_ID, QA_SHORTCUT_ID, TEX_SHORTCUT_ID,
};
use log::{set_logger, set_max_level, LevelFilter};
use panic::panic_hook;
use state::{clear_global_state, initialize_global_state};

pub unsafe extern "C" fn load(api: *mut api::AddonAPI) {
    if api.is_null() {
        // TODO: Can we do anything here?
        return;
    }

    let api = &*api;

    set_logger(api).unwrap();
    set_max_level(if cfg!(feature = "log_traces") {
        LevelFilter::Trace
    } else {
        LevelFilter::Debug
    });

    let api = initialize_global_state(api);

    std::panic::set_hook(Box::new(panic_hook));

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

pub unsafe extern "C" fn unload() {
    clear_global_state();
}
