use log_err::LogErrOption;

use crate::{AddonAPI, ERenderType_ERenderType_Render};

use super::{AddonApiWrapper, Cleanup};

pub type GuiRender = unsafe extern "C" fn();

impl AddonApiWrapper {
    pub unsafe fn register_render(&mut self, render_callback: GuiRender) {
        self.cleanups
            .push(Box::new(RenderWrapper::new(&self.api, render_callback)));
    }
}

struct RenderWrapper(GuiRender);

impl RenderWrapper {
    unsafe fn new(api: &AddonAPI, render_callback: GuiRender) -> Self {
        api.Renderer
            .Register
            .log_expect("cannot register render callback")(
            ERenderType_ERenderType_Render,
            Some(render_callback),
        );

        Self(render_callback)
    }
}

impl Cleanup for RenderWrapper {
    unsafe fn cleanup(&mut self, api: &AddonAPI) {
        api.Renderer
            .Deregister
            .log_expect("cannot unregister render callback")(Some(self.0));
    }
}
