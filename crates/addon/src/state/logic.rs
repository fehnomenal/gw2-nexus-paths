use std::{fs::read_to_string, thread};

use log_err::LogErrResult;
use paths_core::{
    loadable::BackgroundLoadable,
    markers::MarkerCategoryTree,
    settings::{apply_marker_category_settings, backup_marker_category_settings, read_settings},
};

use super::globals::{
    ACTIVE_MARKER_CATEGORIES, API, IS_UI_VISIBLE, MARKER_CATEGORY_TREE, MUMBLE_DATA,
    MUMBLE_IDENTITY, NEXUS_LINK_DATA, RENDERER, SETTINGS, SETTINGS_FILE_PATH, SETTINGS_SAVER,
    UI_INPUT_MANAGER,
};

pub unsafe fn handle_wnd_proc(msg: api::UINT, w_param: api::WPARAM, l_param: api::LPARAM) -> u32 {
    UI_INPUT_MANAGER
        .assume_init_mut()
        .handle_wnd_proc(msg, w_param, l_param)
        .then_some(0)
        .unwrap_or(1)
}

pub unsafe fn load_settings_in_background() {
    *MARKER_CATEGORY_TREE.assume_init_mut() = BackgroundLoadable::Loading;

    thread::Builder::new()
        .name("load_in_background".to_owned())
        .spawn(move || {
            let settings_json = read_to_string(SETTINGS_FILE_PATH.assume_init_ref())
                .log_expect("could not open settings file for reading");

            let settings = SETTINGS.write(read_settings(settings_json.as_bytes()));

            let mut tree = MarkerCategoryTree::from_all_packs_in_dir(
                &API.assume_init_ref().get_path_in_addon_directory("markers"),
            );

            apply_marker_category_settings(settings, &mut tree);

            let BackgroundLoadable::Loaded(ref tree) =
                MARKER_CATEGORY_TREE.write(BackgroundLoadable::Loaded(tree))
            else {
                return;
            };

            ACTIVE_MARKER_CATEGORIES
                .assume_init_mut()
                .read_from_tree(&tree);
        })
        .log_unwrap();
}

pub unsafe fn render() {
    let renderer = RENDERER.assume_init_mut();
    let mumble_data = MUMBLE_DATA.assume_init_ref();

    if IS_UI_VISIBLE {
        renderer.render_ui(
            UI_INPUT_MANAGER.assume_init_mut().get_events(),
            mumble_data,
            MARKER_CATEGORY_TREE.assume_init_ref(),
            || load_settings_in_background(),
            || {
                if let BackgroundLoadable::Loaded(tree) = MARKER_CATEGORY_TREE.assume_init_ref() {
                    backup_marker_category_settings(tree, SETTINGS.assume_init_mut());
                    // Trigger persisting the new settings.
                    SETTINGS_SAVER.assume_init_ref().put(());

                    // Update active categories.
                    ACTIVE_MARKER_CATEGORIES
                        .assume_init_mut()
                        .read_from_tree(tree);
                }
            },
        );
    }

    if NEXUS_LINK_DATA.assume_init_ref().IsGameplay {
        if mumble_data.Context.IsMapOpen() == 0 {
            renderer.render_world();
        }

        renderer.render_map(
            mumble_data,
            ACTIVE_MARKER_CATEGORIES.assume_init_ref(),
            SETTINGS.assume_init_ref(),
        );
    }
}

pub unsafe fn toggle_ui_visible() {
    IS_UI_VISIBLE = !IS_UI_VISIBLE;
}

pub unsafe fn update_mumble_identity(identity: &'static api::Mumble_Identity) {
    MUMBLE_IDENTITY = Some(identity);

    RENDERER
        .assume_init_ref()
        .config
        .lock()
        .log_unwrap()
        .update_ui_size(identity.UISize);

    ACTIVE_MARKER_CATEGORIES
        .assume_init_mut()
        .set_active_map(identity.MapID);
}

pub unsafe fn update_window_size() {
    let renderer = RENDERER.assume_init_mut();
    let nexus_data = NEXUS_LINK_DATA.assume_init_ref();

    renderer.rebuild_render_targets();

    renderer
        .config
        .lock()
        .log_unwrap()
        .update_screen_size(nexus_data.Width as f32, nexus_data.Height as f32);
}
