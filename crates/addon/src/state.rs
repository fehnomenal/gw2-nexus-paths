use std::{
    fs::{read_to_string, File},
    mem::MaybeUninit,
    path::PathBuf,
    rc::Rc,
    sync::Mutex,
    thread,
    time::Duration,
};

use debounce::EventDebouncer;
use egui::{Context, Visuals};
use log_err::{LogErrOption, LogErrResult};
use paths_core::{
    loadable::BackgroundLoadable,
    markers::{ActiveMarkerCategories, MarkerCategoryTree},
    settings::{apply_marker_category_settings, read_settings, write_settings, Settings},
};
use windows::{core::Interface, Win32::Graphics::Dxgi::IDXGISwapChain};

use crate::{
    input_manager::InputManager,
    renderer::{RenderConfig, Renderer},
};

pub unsafe fn initialize_global_state(api: &'static api::AddonAPI) -> &mut api::AddonApiWrapper {
    let api = &mut STATE.write(State::from_api(api)).api;

    load_marker_category_tree_in_background();
    load_settings_in_background();

    api
}

pub unsafe fn update_mumble_identity(identity: &'static api::Mumble_Identity) {
    let state = STATE.assume_init_mut();

    state.mumble_identity = Some(identity);
    state
        .active_marker_categories
        .set_active_map(identity.MapID);
}

pub unsafe fn clear_global_state() {
    STATE.assume_init_drop();
}

pub unsafe fn get_api() -> &'static api::AddonApiWrapper {
    &STATE.assume_init_ref().api
}

#[allow(dead_code)]
pub unsafe fn get_mumble_identity() -> Option<&'static api::Mumble_Identity> {
    STATE.assume_init_ref().mumble_identity
}

pub unsafe fn get_mumble_data() -> &'static api::Mumble_Data {
    STATE.assume_init_ref().mumble
}

pub unsafe fn get_nexus_link() -> &'static api::NexusLinkData {
    STATE.assume_init_ref().nexus_link
}

pub unsafe fn get_renderer() -> &'static mut Renderer<'static> {
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

pub unsafe fn load_marker_category_tree_in_background() -> thread::JoinHandle<()> {
    STATE.assume_init_mut().marker_category_tree = BackgroundLoadable::Loading;

    thread::Builder::new()
        .name("load_markers".to_owned())
        .spawn(|| {
            let api = get_api();

            let marker_dir = api.get_path_in_addon_directory("markers");
            let mut tree = MarkerCategoryTree::from_all_packs_in_dir(&marker_dir);

            let state = STATE.assume_init_mut();

            apply_marker_category_settings(&state.settings.settings, &mut tree);

            state.marker_category_tree = BackgroundLoadable::Loaded(tree);

            // This needs to be a call to get the static lifetime...
            if let BackgroundLoadable::Loaded(tree) = get_marker_category_tree() {
                state.active_marker_categories.read_from_tree(&tree);
            }
        })
        .log_unwrap()
}

pub unsafe fn get_marker_category_tree() -> &'static BackgroundLoadable<MarkerCategoryTree> {
    &STATE.assume_init_ref().marker_category_tree
}

pub unsafe fn get_active_marker_categories() -> &'static mut ActiveMarkerCategories<'static> {
    &mut STATE.assume_init_mut().active_marker_categories
}

pub unsafe fn load_settings_in_background() -> thread::JoinHandle<()> {
    let path = STATE.assume_init_ref().settings.file_path.as_path();

    thread::Builder::new()
        .name("load_settings".to_owned())
        .spawn(move || {
            let settings_json =
                read_to_string(path).log_expect("could not open settings file for reading");

            let settings = read_settings(settings_json.as_bytes());

            unsafe {
                let state = STATE.assume_init_mut();

                if let BackgroundLoadable::Loaded(tree) = &mut state.marker_category_tree {
                    apply_marker_category_settings(&settings, tree);
                }

                state.settings.settings = settings;

                // This needs to be a call to get the static lifetime...
                if let BackgroundLoadable::Loaded(tree) = get_marker_category_tree() {
                    state.active_marker_categories.read_from_tree(&tree);
                }
            }
        })
        .log_unwrap()
}

pub unsafe fn get_settings() -> &'static Settings {
    &STATE.assume_init_ref().settings.settings
}

pub unsafe fn update_settings<F: FnMut(&mut Settings)>(mut update: F) {
    let holder = &mut STATE.assume_init_mut().settings;

    update(&mut holder.settings);

    holder.request_save();
}

static mut STATE: MaybeUninit<State> = MaybeUninit::uninit();

struct State<'a> {
    api: api::AddonApiWrapper,

    mumble_identity: Option<&'a api::Mumble_Identity>,
    mumble: &'a api::Mumble_Data,
    nexus_link: &'a api::NexusLinkData,

    renderer: Renderer<'a>,
    ui_input_manager: InputManager,
    is_ui_visible: bool,

    marker_category_tree: BackgroundLoadable<MarkerCategoryTree>,
    active_marker_categories: ActiveMarkerCategories<'a>,
    settings: SettingsHolder,
}

impl<'a> State<'a> {
    unsafe fn from_api(api: &'static api::AddonAPI) -> Self {
        let data_link_get = api
            .DataLink
            .Get
            .log_expect("could not get data link elements");

        let mumble = &*(data_link_get(c"DL_MUMBLE_LINK".as_ptr()) as *mut api::Mumble_Data);
        let nexus_link = &*(data_link_get(c"DL_NEXUS_LINK".as_ptr()) as *mut api::NexusLinkData);

        let egui_context = Context::default();
        egui_context.set_visuals(Visuals::light());

        let render_config = Rc::new(Mutex::new(RenderConfig::new(
            nexus_link.Width as f32,
            nexus_link.Height as f32,
        )));

        let renderer = Renderer::new(
            render_config,
            IDXGISwapChain::from_raw_borrowed(&api.SwapChain)
                .log_expect("could not get swap chain"),
            egui_context.clone(),
        );
        let input_manager = InputManager::new(egui_context.clone());

        Self {
            api: api::AddonApiWrapper::wrap_api(*api),

            mumble_identity: None,
            mumble,
            nexus_link,

            renderer,
            ui_input_manager: input_manager,
            is_ui_visible: false,

            marker_category_tree: BackgroundLoadable::Loading,
            active_marker_categories: ActiveMarkerCategories::new(),
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
            File::create(&self.file_path).log_expect("could not open settings file for writing");
        write_settings(&mut file, &self.settings);
    }
}
