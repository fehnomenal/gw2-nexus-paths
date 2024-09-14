use std::{mem::MaybeUninit, thread};

use crate::data::{load_all_marker_packs, MarkerCategoryTree};

pub unsafe fn initialize_global_state(api: &'static api::AddonAPI) -> &api::NexusLinkData {
    let state = STATE.write(State::from_api(api));

    state.nexus_link
}

pub unsafe fn update_mumble_identity(identity: &'static api::Mumble_Identity) {
    let state = STATE.assume_init_mut();

    state.mumble_identity = Some(identity);
}

pub unsafe fn clear_global_state() {
    STATE.assume_init_drop();
}

pub unsafe fn get_api() -> &'static api::AddonAPI {
    STATE.assume_init_ref().api
}

pub unsafe fn get_mumble_identity() -> Option<&'static api::Mumble_Identity> {
    STATE.assume_init_ref().mumble_identity
}

pub unsafe fn get_mumble_link() -> &'static api::Mumble_Data {
    STATE.assume_init_ref().mumble_link
}

pub unsafe fn get_nexus_link() -> &'static api::NexusLinkData {
    STATE.assume_init_ref().nexus_link
}

pub unsafe fn load_marker_category_tree_in_background() {
    thread::spawn(|| {
        let api = get_api();

        let marker_dir = api.get_path_in_addon_directory("markers");
        let tree = load_all_marker_packs(&marker_dir);

        STATE.assume_init_mut().marker_category_tree = BackgroundLoadable::Loaded(tree);
    });

    STATE.assume_init_mut().marker_category_tree = BackgroundLoadable::Loading;
}

pub unsafe fn get_marker_category_tree() -> &'static BackgroundLoadable<MarkerCategoryTree> {
    &STATE.assume_init_ref().marker_category_tree
}

static mut STATE: MaybeUninit<State> = MaybeUninit::uninit();

struct State {
    api: &'static api::AddonAPI,

    mumble_identity: Option<&'static api::Mumble_Identity>,
    mumble_link: &'static api::Mumble_Data,
    nexus_link: &'static api::NexusLinkData,

    marker_category_tree: BackgroundLoadable<MarkerCategoryTree>,
}

impl State {
    unsafe fn from_api(api: &'static api::AddonAPI) -> Self {
        let data_link_get = api.DataLink.Get.expect("Could not get data link elements");

        let mumble_link = &*(data_link_get(c"DL_MUMBLE_LINK".as_ptr()) as *mut api::Mumble_Data);
        let nexus_link = &*(data_link_get(c"DL_NEXUS_LINK".as_ptr()) as *mut api::NexusLinkData);

        load_marker_category_tree_in_background();

        Self {
            api,
            mumble_identity: None,
            mumble_link,
            nexus_link,

            marker_category_tree: BackgroundLoadable::Loading,
        }
    }
}

pub enum BackgroundLoadable<T> {
    Loading,
    Loaded(T),
}
