pub use api::{get_api, set_global_api};

mod api {
    use std::{mem::MaybeUninit, sync::Once};

    use crate::nexus::api;

    static mut API: MaybeUninit<api::AddonAPI> = MaybeUninit::uninit();

    pub fn set_global_api(api: api::AddonAPI) {
        static ONCE: Once = Once::new();

        ONCE.call_once(|| {
            unsafe { API.write(api) };
        });
    }

    pub fn get_api() -> &'static api::AddonAPI {
        unsafe { API.assume_init_ref() }
    }
}
