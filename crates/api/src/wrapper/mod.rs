mod events;
mod key_binds;
mod render;
mod shortcuts;
mod wnd_proc;

use std::ops::Deref;

use crate::AddonAPI;

trait Cleanup {
    unsafe fn cleanup(&mut self, api: &AddonAPI);
}

pub struct AddonApiWrapper<'a> {
    api: &'a AddonAPI,
    cleanups: Vec<Box<dyn Cleanup>>,
}

impl<'a> AddonApiWrapper<'a> {
    pub fn wrap_api(api: &'a AddonAPI) -> Self {
        Self {
            api,
            cleanups: vec![],
        }
    }
}

impl Deref for AddonApiWrapper<'_> {
    type Target = AddonAPI;

    fn deref(&self) -> &Self::Target {
        &self.api
    }
}

impl Drop for AddonApiWrapper<'_> {
    fn drop(&mut self) {
        self.cleanups
            .iter_mut()
            .rev()
            .for_each(|c| unsafe { c.cleanup(&self.api) });
    }
}
