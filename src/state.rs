use std::mem::MaybeUninit;

use crate::nexus::api;

static mut API: MaybeUninit<&api::AddonAPI> = MaybeUninit::uninit();
static mut NEXUS_LINK: MaybeUninit<&api::NexusLinkData> = MaybeUninit::uninit();

pub fn initialize_global_state(api: &'static api::AddonAPI) {
    unsafe {
        API.write(api);
        NEXUS_LINK.write(extract_nexus_link(api));
    }
}

pub fn get_api() -> &'static api::AddonAPI {
    unsafe { API.assume_init_ref() }
}

pub fn get_nexus_link() -> &'static api::NexusLinkData {
    unsafe { NEXUS_LINK.assume_init_ref() }
}

unsafe fn extract_nexus_link(api: &api::AddonAPI) -> &api::NexusLinkData {
    let dl = unsafe { api.DataLink.Get.unwrap()(c"DL_NEXUS_LINK".as_ptr()) };

    &*(dl as *mut api::NexusLinkData)
}
