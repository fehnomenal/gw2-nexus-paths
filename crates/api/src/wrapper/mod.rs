mod events;
mod key_binds;
mod render;
mod shortcuts;
mod wnd_proc;

use crate::AddonAPI;

trait Cleanup {
    unsafe fn cleanup(&mut self, api: &AddonAPI);
}

pub struct AddonApiWrapper {
    api: AddonAPI,
    cleanups: Vec<Box<dyn Cleanup>>,
}

impl AddonApiWrapper {
    pub fn wrap_api(api: AddonAPI) -> Self {
        Self {
            api,
            cleanups: vec![],
        }
    }
}

impl std::ops::Deref for AddonApiWrapper {
    type Target = AddonAPI;

    fn deref(&self) -> &Self::Target {
        &self.api
    }
}

impl Drop for AddonApiWrapper {
    fn drop(&mut self) {
        self.cleanups
            .iter_mut()
            .rev()
            .for_each(|c| unsafe { c.cleanup(&self.api) });
    }
}
