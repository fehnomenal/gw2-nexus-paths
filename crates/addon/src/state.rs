use std::{
    cell::RefCell,
    fs::{read_to_string, File},
    mem::MaybeUninit,
    path::PathBuf,
    rc::Rc,
    thread,
    time::Duration,
};

use debounce::EventDebouncer;
use egui::{Context, Visuals};
use paths_core::{loadable::BackgroundLoadable, settings::apply_marker_category_settings};
use paths_data::{
    markers::MarkerCategoryTree,
    settings::{read_settings, write_settings},
};
use paths_renderer::{RenderConfig, Renderer};
use paths_types::settings::Settings;
use windows::{core::Interface, Win32::Graphics::Dxgi::IDXGISwapChain};

use crate::{
    input_manager::InputManager,
    logger::{create_logger, Logger},
};

pub unsafe fn initialize_global_state(api: &'static api::AddonAPI) -> &mut api::AddonApiWrapper {
    &mut STATE.write(State::from_api(api)).api
}

pub unsafe fn update_mumble_identity(identity: &'static api::Mumble_Identity) {
    let state = STATE.assume_init_mut();

    state.mumble_identity = Some(identity);
}

pub unsafe fn clear_global_state() {
    STATE.assume_init_drop();
}

pub unsafe fn get_api() -> &'static api::AddonApiWrapper {
    &STATE.assume_init_ref().api
}

pub unsafe fn get_logger() -> &'static Logger {
    &STATE.assume_init_ref().logger
}

pub unsafe fn get_mumble_identity() -> Option<&'static api::Mumble_Identity> {
    STATE.assume_init_ref().mumble_identity
}

pub unsafe fn get_mumble_data() -> &'static api::Mumble_Data {
    STATE.assume_init_ref().mumble
}

pub unsafe fn get_nexus_link() -> &'static api::NexusLinkData {
    STATE.assume_init_ref().nexus_link
}

pub unsafe fn get_renderer() -> &'static mut Renderer {
    &mut STATE.assume_init_mut().renderer
}

pub unsafe fn get_input_manager() -> &'static mut InputManager {
    &mut STATE.assume_init_mut().ui_input_manager
}

pub unsafe fn is_ui_visible() -> bool {
    STATE.assume_init_ref().is_ui_visible
}

pub unsafe fn toggle_ui_visible() {
    let state = STATE.assume_init_mut();

    state.is_ui_visible = !state.is_ui_visible;
}

pub unsafe fn load_marker_category_tree_in_background() {
    thread::Builder::new()
        .name("load_markers".to_owned())
        .spawn(|| {
            let api = get_api();

            let marker_dir = api.get_path_in_addon_directory("markers");
            let mut tree = MarkerCategoryTree::from_all_packs_in_dir(&marker_dir);

            apply_marker_category_settings(&STATE.assume_init_ref().settings.settings, &mut tree);

            STATE.assume_init_mut().marker_category_tree = BackgroundLoadable::Loaded(tree);
        })
        .unwrap();

    STATE.assume_init_mut().marker_category_tree = BackgroundLoadable::Loading;
}

pub unsafe fn get_marker_category_tree(
) -> &'static BackgroundLoadable<MarkerCategoryTree<egui::Rgba>> {
    &STATE.assume_init_ref().marker_category_tree
}

pub unsafe fn update_settings<F: FnMut(&mut Settings)>(mut update: F) {
    let holder = &mut STATE.assume_init_mut().settings;

    update(&mut holder.settings);

    holder.request_save();
}

static mut STATE: MaybeUninit<State<egui::Rgba>> = MaybeUninit::uninit();

struct State<'a, C> {
    api: api::AddonApiWrapper,
    logger: Logger,

    mumble_identity: Option<&'a api::Mumble_Identity>,
    mumble: &'a api::Mumble_Data,
    nexus_link: &'a api::NexusLinkData,

    renderer: Renderer,
    ui_input_manager: InputManager,
    is_ui_visible: bool,

    marker_category_tree: BackgroundLoadable<MarkerCategoryTree<C>>,
    settings: SettingsHolder,
}

impl<C> State<'static, C> {
    unsafe fn from_api(api: &'static api::AddonAPI) -> Self {
        let data_link_get = api.DataLink.Get.expect("Could not get data link elements");

        let mumble = &*(data_link_get(c"DL_MUMBLE_LINK".as_ptr()) as *mut api::Mumble_Data);
        let nexus_link = &*(data_link_get(c"DL_NEXUS_LINK".as_ptr()) as *mut api::NexusLinkData);

        let egui_context = Context::default();
        egui_context.set_visuals(Visuals::light());

        let render_config = Rc::new(RefCell::new(RenderConfig::new(
            nexus_link.Width as f32,
            nexus_link.Height as f32,
        )));

        let renderer = Renderer::new(
            render_config,
            IDXGISwapChain::from_raw_borrowed(&api.SwapChain).expect("Could not get swap chain"),
            egui_context.clone(),
        );
        let input_manager = InputManager::new(egui_context.clone());

        load_marker_category_tree_in_background();

        Self {
            api: api::AddonApiWrapper::wrap_api(*api),
            logger: create_logger(api.Log),

            mumble_identity: None,
            mumble,
            nexus_link,

            renderer,
            ui_input_manager: input_manager,
            is_ui_visible: false,

            marker_category_tree: BackgroundLoadable::Loading,
            settings: SettingsHolder::from_api(&api),
        }
    }
}

struct SettingsHolder {
    settings: Settings,
    file_path: PathBuf,
    save_debouncer: EventDebouncer<()>,
}

impl SettingsHolder {
    fn from_api(api: &api::AddonAPI) -> Self {
        let file_path = api.get_path_in_addon_directory("settings.json");
        let cloned = file_path.clone();

        thread::Builder::new()
            .name("load_settings".to_owned())
            .spawn(|| {
                let settings_json =
                    read_to_string(cloned).expect("Could not open settings file for reading");

                let settings = read_settings(settings_json.as_bytes());

                unsafe {
                    let state = STATE.assume_init_mut();

                    if let BackgroundLoadable::Loaded(tree) = &mut state.marker_category_tree {
                        apply_marker_category_settings(&settings, tree);
                    }

                    state.settings.settings = settings;
                }
            })
            .unwrap();

        Self {
            settings: Settings::default(),
            file_path,
            save_debouncer: EventDebouncer::new(Duration::from_secs(1), |_| {
                let holder = unsafe { &STATE.assume_init_ref().settings };
                holder.write_to_file();
            }),
        }
    }

    fn request_save(&self) {
        self.save_debouncer.put(());
    }

    fn write_to_file(&self) {
        let mut file =
            File::create(&self.file_path).expect("Could not open settings file for writing");
        write_settings(&mut file, &self.settings);
    }
}
