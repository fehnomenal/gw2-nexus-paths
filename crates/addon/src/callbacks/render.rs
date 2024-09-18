use paths_core::state::{get_mumble_link, get_nexus_link};

use crate::globals::{IS_UI_VISIBLE, RENDERER, UI_INPUT_MANAGER};

pub unsafe extern "C" fn render_cb() {
    let renderer = RENDERER.assume_init_mut();

    if IS_UI_VISIBLE {
        renderer.render_ui(UI_INPUT_MANAGER.assume_init_mut().get_events());
    }

    if !get_nexus_link().IsGameplay {
        return;
    }

    if get_mumble_link().Context.IsMapOpen() == 0 {
        renderer.render_world();
    }

    renderer.render_map();
}
