use std::mem::MaybeUninit;

use crate::nexus::api;

pub unsafe fn initialize_global_state(api: &'static api::AddonAPI) {
    STATE.write(State::from_api(api));
}

pub unsafe fn clear_global_state() {
    STATE.assume_init_drop();
}

pub unsafe fn get_api() -> &'static api::AddonAPI {
    STATE.assume_init_ref().api
}

pub unsafe fn get_nexus_link() -> &'static api::NexusLinkData {
    STATE.assume_init_ref().nexus_link
}

static mut STATE: MaybeUninit<State> = MaybeUninit::uninit();

struct State {
    api: &'static api::AddonAPI,
    nexus_link: &'static api::NexusLinkData,
}

impl State {
    unsafe fn from_api(api: &'static api::AddonAPI) -> Self {
        let data_link_get = api.DataLink.Get.unwrap();

        let nexus_link = &*(data_link_get(c"DL_NEXUS_LINK".as_ptr()) as *mut api::NexusLinkData);

        Self { api, nexus_link }
    }
}
