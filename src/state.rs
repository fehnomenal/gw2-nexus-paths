use std::{
    ffi::c_void,
    mem::{self, MaybeUninit},
};

use crate::{nexus::api, render::RenderState};

pub unsafe fn initialize_global_state(api: &'static api::AddonAPI) {
    STATE.write(State::from_api(api));

    let subscribe_event = api.Events.Subscribe.unwrap();

    subscribe_event(
        c"EV_MUMBLE_IDENTITY_UPDATED".as_ptr(),
        Some(identity_updated_cb),
    );

    subscribe_event(c"EV_WINDOW_RESIZED".as_ptr(), Some(window_resized_cb));
}

pub unsafe fn clear_global_state() {
    let unsubscribe_event = STATE.assume_init_ref().api.Events.Unsubscribe.unwrap();

    unsubscribe_event(
        c"EV_MUMBLE_IDENTITY_UPDATED".as_ptr(),
        Some(identity_updated_cb),
    );

    unsubscribe_event(c"EV_WINDOW_RESIZED".as_ptr(), Some(window_resized_cb));

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

pub unsafe fn get_render_state() -> &'static RenderState {
    &STATE.assume_init_ref().render_state
}

static mut STATE: MaybeUninit<State> = MaybeUninit::uninit();

struct State {
    api: &'static api::AddonAPI,

    mumble_identity: Option<&'static api::mumble::Mumble_Identity>,
    mumble_link: &'static api::mumble::Mumble_Data,
    nexus_link: &'static api::NexusLinkData,

    render_state: RenderState,
}

impl State {
    unsafe fn from_api(api: &'static api::AddonAPI) -> Self {
        let data_link_get = api.DataLink.Get.unwrap();

        let mumble_link =
            &*(data_link_get(c"DL_MUMBLE_LINK".as_ptr()) as *mut api::mumble::Mumble_Data);
        let nexus_link = &*(data_link_get(c"DL_NEXUS_LINK".as_ptr()) as *mut api::NexusLinkData);

        let render_state = RenderState::new(nexus_link.Width as f32, nexus_link.Height as f32);

        Self {
            api,
            mumble_identity: None,
            mumble_link,
            nexus_link,
            render_state,
        }
    }
}

unsafe extern "C" fn identity_updated_cb(identity: *mut c_void) {
    let identity = mem::transmute::<*mut c_void, &api::mumble::Mumble_Identity>(identity);

    let state = STATE.assume_init_mut();

    state.mumble_identity = Some(identity);
    state.render_state.update_ui_size(identity.UISize);
}

unsafe extern "C" fn window_resized_cb(_payload: *mut c_void) {
    let state = STATE.assume_init_mut();

    state.render_state.update_screen_size(
        state.nexus_link.Width as f32,
        state.nexus_link.Height as f32,
    );
}
