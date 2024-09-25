use paths_core::{loadable::BackgroundLoadable, settings::backup_marker_category_settings};

use crate::state::{
    get_active_marker_categories, get_input_manager, get_marker_category_tree, get_mumble_data,
    get_nexus_link, get_renderer, is_ui_visible, load_marker_category_tree_in_background,
    update_settings,
};

pub unsafe extern "C" fn render_cb() {
    let renderer = get_renderer();
    let mumble_data = get_mumble_data();

    if is_ui_visible() {
        let marker_category_tree = get_marker_category_tree();

        renderer.render_ui(
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
        renderer.render_world();
    }

    renderer.render_map(mumble_data);
}
