use std::ffi::CString;

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
    pub fn subscribe_event(&self, event_name: &str, callback: EventConsume) {
        unsafe {
            let cstr = CString::new(event_name).unwrap();

            self.Events.Subscribe.unwrap()(cstr.as_ptr(), Some(callback));
        }
    }

    pub fn unsubscribe_event(&self, event_name: &str, callback: EventConsume) {
        unsafe {
            let cstr = CString::new(event_name).unwrap();

            self.Events.Unsubscribe.unwrap()(cstr.as_ptr(), Some(callback));
        }
    }

    pub fn register_render(&self, render_callback: GuiRender) {
        unsafe {
            self.Renderer.Register.unwrap()(
                api::ERenderType_ERenderType_Render,
                Some(render_callback),
            );
        }
    }

    pub fn register_pre_render(&self, render_callback: GuiRender) {
        unsafe {
            self.Renderer.Register.unwrap()(
                api::ERenderType_ERenderType_PreRender,
                Some(render_callback),
            );
        }
    }

    pub fn register_options_render(&self, render_callback: GuiRender) {
        unsafe {
            self.Renderer.Register.unwrap()(
                api::ERenderType_ERenderType_OptionsRender,
                Some(render_callback),
            );
        }
    }

    pub fn register_post_render(&self, render_callback: GuiRender) {
        unsafe {
            self.Renderer.Register.unwrap()(
                api::ERenderType_ERenderType_PostRender,
                Some(render_callback),
            );
        }
    }

    pub fn unregister_render(&self, render_callback: GuiRender) {
        unsafe {
            self.Renderer.Deregister.unwrap()(Some(render_callback));
        }
    }

    pub fn register_wnd_proc(&self, callback: WndProcCallback) {
        unsafe {
            self.WndProc.Register.unwrap()(Some(callback));
        }
    }
    pub fn unregister_wnd_proc(&self, callback: WndProcCallback) {
        unsafe {
            self.WndProc.Deregister.unwrap()(Some(callback));
        }
    }
}
