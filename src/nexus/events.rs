use std::ffi::CStr;

use super::api;

type EventConsume = unsafe extern "C" fn(aEventArgs: *mut ::std::os::raw::c_void);

type GuiRender = unsafe extern "C" fn();

type WndProcCallback = unsafe extern "C" fn(
    hWnd: api::HWND,
    uMsg: api::UINT,
    wParam: api::WPARAM,
    lParam: api::LPARAM,
) -> api::UINT;

impl api::AddonAPI {
    pub fn subscribe_event(&self, event_name: &CStr, callback: EventConsume) {
        unsafe {
            self.Events.Subscribe.expect("Could not subscribe to event")(
                event_name.as_ptr(),
                Some(callback),
            );
        }
    }

    pub fn unsubscribe_event(&self, event_name: &CStr, callback: EventConsume) {
        unsafe {
            self.Events
                .Unsubscribe
                .expect("Could not unsubscribe from event")(
                event_name.as_ptr(), Some(callback)
            );
        }
    }

    pub fn register_render(&self, render_callback: GuiRender) {
        unsafe {
            self.Renderer
                .Register
                .expect("Could not register render callback")(
                api::ERenderType_ERenderType_Render,
                Some(render_callback),
            );
        }
    }

    pub fn register_pre_render(&self, render_callback: GuiRender) {
        unsafe {
            self.Renderer
                .Register
                .expect("Could not register pre render callback")(
                api::ERenderType_ERenderType_PreRender,
                Some(render_callback),
            );
        }
    }

    pub fn register_options_render(&self, render_callback: GuiRender) {
        unsafe {
            self.Renderer
                .Register
                .expect("Could not register options render callback")(
                api::ERenderType_ERenderType_OptionsRender,
                Some(render_callback),
            );
        }
    }

    pub fn register_post_render(&self, render_callback: GuiRender) {
        unsafe {
            self.Renderer
                .Register
                .expect("Could not register post render callback")(
                api::ERenderType_ERenderType_PostRender,
                Some(render_callback),
            );
        }
    }

    pub fn unregister_render(&self, render_callback: GuiRender) {
        unsafe {
            self.Renderer
                .Deregister
                .expect("Could not unregister render callback")(Some(render_callback));
        }
    }

    pub fn register_wnd_proc(&self, callback: WndProcCallback) {
        unsafe {
            self.WndProc
                .Register
                .expect("Could not register wnd proc handler")(Some(callback));
        }
    }
    pub fn unregister_wnd_proc(&self, callback: WndProcCallback) {
        unsafe {
            self.WndProc
                .Deregister
                .expect("Could not unregister wnd proc handler")(Some(callback));
        }
    }
}
