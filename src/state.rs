use std::{
    ffi::c_void,
    mem::{self, MaybeUninit},
};

use crate::nexus::api;

pub unsafe fn initialize_global_state(api: &'static api::AddonAPI) {
    STATE.write(State::from_api(api));

    api.Events.Subscribe.unwrap()(
        c"EV_MUMBLE_IDENTITY_UPDATED".as_ptr(),
        Some(identity_updated_cb),
    );
}

pub unsafe fn clear_global_state() {
    STATE.assume_init_ref().api.Events.Unsubscribe.unwrap()(
        c"EV_MUMBLE_IDENTITY_UPDATED".as_ptr(),
        Some(identity_updated_cb),
    );

    STATE.assume_init_drop();
}

pub unsafe fn get_api() -> &'static api::AddonAPI {
    STATE.assume_init_ref().api
}

pub unsafe fn get_mumble_identity() -> Option<&'static api::mumble::Mumble_Identity> {
    STATE.assume_init_ref().mumble_identity
}

pub unsafe fn get_mumble_link() -> &'static api::mumble::Mumble_Data {
    STATE.assume_init_ref().mumble_link
}

pub unsafe fn get_nexus_link() -> &'static api::NexusLinkData {
    STATE.assume_init_ref().nexus_link
}

static mut STATE: MaybeUninit<State> = MaybeUninit::uninit();

struct State {
    api: &'static api::AddonAPI,

    mumble_identity: Option<&'static api::mumble::Mumble_Identity>,
    mumble_link: &'static api::mumble::Mumble_Data,
    nexus_link: &'static api::NexusLinkData,
}

impl State {
    unsafe fn from_api(api: &'static api::AddonAPI) -> Self {
        let data_link_get = api.DataLink.Get.unwrap();

        let mumble_link =
            &*(data_link_get(c"DL_MUMBLE_LINK".as_ptr()) as *mut api::mumble::Mumble_Data);
        let nexus_link = &*(data_link_get(c"DL_NEXUS_LINK".as_ptr()) as *mut api::NexusLinkData);

        Self {
            api,
            mumble_identity: None,
            mumble_link,
            nexus_link,
        }
    }
}

unsafe extern "C" fn identity_updated_cb(identity: *mut c_void) {
    let identity = mem::transmute::<*mut c_void, &api::mumble::Mumble_Identity>(identity);

    STATE.assume_init_mut().mumble_identity = Some(identity);
}
