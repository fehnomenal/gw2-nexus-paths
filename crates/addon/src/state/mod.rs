mod globals;
mod logic;

use std::{fs::File, rc::Rc, sync::Mutex, time::Duration};

use debounce::EventDebouncer;
use log_err::{LogErrOption, LogErrResult};
use paths_core::{
    markers::ActiveMarkerCategories,
    settings::{write_settings, Settings},
    ui::{prepare_egui_context, MainWindow, MarkerTreeWindow, UiState},
};
use windows::{core::Interface, Win32::Graphics::Dxgi::IDXGISwapChain};

use crate::{
    input_manager::InputManager,
    renderer::{RenderConfig, Renderer},
};

use self::globals::{
    ACTIVE_MARKER_CATEGORIES, API, MARKER_CATEGORY_TREE, MUMBLE_DATA, MUMBLE_IDENTITY,
    NEXUS_LINK_DATA, RENDERER, SETTINGS, SETTINGS_FILE_PATH, SETTINGS_SAVER, UI_INPUT_MANAGER,
    UI_STATE,
};
pub use self::logic::*;

pub unsafe fn init_globals(api: &'static api::AddonAPI) -> &mut api::AddonApiWrapper {
    let api_wrapper = API.write(api::AddonApiWrapper::wrap_api(api));

    let nexus_link_data = {
        let get_data_link = api
            .DataLink
            .Get
            .log_expect("could not get data link elements");

        let mumble_data = get_data_link(c"DL_MUMBLE_LINK".as_ptr());
        let nexus_link_data = get_data_link(c"DL_NEXUS_LINK".as_ptr());

        MUMBLE_DATA.write(std::mem::transmute(mumble_data));
        NEXUS_LINK_DATA.write(std::mem::transmute(nexus_link_data))
    };

    {
        let egui_context = prepare_egui_context(egui::Context::default());

        UI_STATE.write(UiState {
            actions: AddonUiActions,
            ui_was_displayed_once: false,
            main_window: MainWindow {
                actions: AddonUiActions,
                open: false,
            },
            marker_tree_window: MarkerTreeWindow {
                actions: AddonUiActions,
                open: false,
            },
        });

        RENDERER.write(Renderer::new(
            Rc::new(Mutex::new(RenderConfig::new(
                nexus_link_data.Width as f32,
                nexus_link_data.Height as f32,
            ))),
            IDXGISwapChain::from_raw_borrowed(&api.SwapChain)
                .log_expect("could not get swap chain"),
            egui_context.clone(),
        ));

        UI_INPUT_MANAGER.write(InputManager::new(egui_context));
    }

    ACTIVE_MARKER_CATEGORIES.write(ActiveMarkerCategories::new());

    {
        SETTINGS_FILE_PATH.write(api.get_path_in_addon_directory("settings.json"));

        SETTINGS.write(Settings::default());

        SETTINGS_SAVER.write(EventDebouncer::new(Duration::from_secs(1), |_| {
            let mut file = File::create(SETTINGS_FILE_PATH.assume_init_ref())
                .log_expect("could not open settings file for writing");

            write_settings(&mut file, SETTINGS.assume_init_ref());
        }));
    }

    api_wrapper
}

pub unsafe fn uninit_globals() {
    SETTINGS_SAVER.assume_init_drop();

    SETTINGS.assume_init_drop();

    SETTINGS_FILE_PATH.assume_init_drop();

    ACTIVE_MARKER_CATEGORIES.assume_init_drop();

    MARKER_CATEGORY_TREE.assume_init_drop();

    UI_INPUT_MANAGER.assume_init_drop();

    RENDERER.assume_init_drop();

    UI_STATE.assume_init_drop();

    NEXUS_LINK_DATA.assume_init_drop();

    MUMBLE_DATA.assume_init_drop();

    let _ = MUMBLE_IDENTITY.take();

    API.assume_init_drop();
}
