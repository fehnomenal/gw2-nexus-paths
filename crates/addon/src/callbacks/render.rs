use paths_core::{loadable::BackgroundLoadable, settings::backup_marker_category_settings};

use crate::state::{
    get_active_marker_categories, get_input_manager, get_marker_category_tree, get_mumble_data,
    get_nexus_link, get_renderer, get_settings, load_marker_category_tree_in_background, ui_state,
    update_settings,
};

pub unsafe extern "C" fn render_cb() {
    let renderer = get_renderer();
    let mumble_data = get_mumble_data();
    let ui_state = ui_state();

    // This is a stupid hack. It seems that some objects of the directx11 renderer are not initialized on the
    // start of rendering but only later on. With this condition the first render is deferred until the user
    // clicks the menu button. Apparently, this is enough to allow the initialization. Probably, this is
    // related to the bug that kills the UI on resizing the game window.
    if ui_state.ui_was_displayed_once {
        let marker_category_tree = get_marker_category_tree();

        renderer.render_ui(
            ui_state,
            get_input_manager().get_events(),
            mumble_data,
            marker_category_tree,
            || load_marker_category_tree_in_background(),
            || {
                update_settings(|settings| {
                    if let BackgroundLoadable::Loaded(tree) = marker_category_tree {
                        backup_marker_category_settings(tree, settings);
                    }
                });

                if let BackgroundLoadable::Loaded(tree) = marker_category_tree {
                    let active_marker_categories = get_active_marker_categories();
                    active_marker_categories.read_from_tree(tree);
                }
            },
        );
    }

    if !get_nexus_link().IsGameplay {
        return;
    }

    if mumble_data.Context.IsMapOpen() == 0 {
        // renderer.render_world();
    }

    let active_marker_categories = get_active_marker_categories();
    let settings = get_settings();

    renderer.render_map(mumble_data, active_marker_categories, &settings);
}
