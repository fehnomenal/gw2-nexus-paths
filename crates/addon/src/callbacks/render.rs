use paths_core::{loadable::BackgroundLoadable, settings::backup_marker_category_settings};

use crate::{
    globals::{IS_UI_VISIBLE, RENDERER, UI_INPUT_MANAGER},
    state::{
        get_marker_category_tree, get_mumble_data, get_nexus_link,
        load_marker_category_tree_in_background, update_settings,
    },
};

pub unsafe extern "C" fn render_cb() {
    let renderer = RENDERER.assume_init_mut();
    let mumble_data = get_mumble_data();

    if IS_UI_VISIBLE {
        let marker_category_tree = get_marker_category_tree();

        renderer.render_ui(
            UI_INPUT_MANAGER.assume_init_mut().get_events(),
            mumble_data,
            marker_category_tree,
            || load_marker_category_tree_in_background(),
            || {
                update_settings(|settings| {
                    if let BackgroundLoadable::Loaded(tree) = marker_category_tree {
                        backup_marker_category_settings(tree, settings);
                    }
                });
            },
        );
    }

    if !get_nexus_link().IsGameplay {
        return;
    }

    if mumble_data.Context.IsMapOpen() == 0 {
        renderer.render_world();
    }

    renderer.render_map(mumble_data);
}
