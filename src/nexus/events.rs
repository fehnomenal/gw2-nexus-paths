use std::ffi::CString;

use super::api::{
    self, ERenderType, ERenderType_ERenderType_OptionsRender, ERenderType_ERenderType_PostRender,
    ERenderType_ERenderType_PreRender, ERenderType_ERenderType_Render,
};

type EventsSubscribe = unsafe extern "C" fn(
    aIdentifier: *const ::std::os::raw::c_char,
    aConsumeEventCallback: Option<EventConsume>,
);
type EventConsume = unsafe extern "C" fn(aEventArgs: *mut ::std::os::raw::c_void);

type RegisterRender =
    unsafe extern "C" fn(aRenderType: ERenderType, aRenderCallback: Option<GuiRender>);
type UnregisterRender = unsafe extern "C" fn(aRenderCallback: Option<GuiRender>);
type GuiRender = unsafe extern "C" fn();

pub struct NexusEventListeners {
    subscribe_event: EventsSubscribe,
    unsubscribe_event: EventsSubscribe,
    register_render: RegisterRender,
    unregister_render: UnregisterRender,
}

impl NexusEventListeners {
    pub fn for_api(api: &api::AddonAPI) -> Self {
        let subscribe_event = api.Events.Subscribe.expect("Cannot subscribe to events");
        let unsubscribe_event = api
            .Events
            .Unsubscribe
            .expect("Cannot unsubscribe from events");

        let register_render = api
            .Renderer
            .Register
            .expect("Cannot register render callback");
        let unregister_render = api
            .Renderer
            .Deregister
            .expect("Cannot unregister render callback");

        Self {
            subscribe_event,
            unsubscribe_event,
            register_render,
            unregister_render,
        }
    }

    pub fn subscribe_event(&self, event_name: &str, callback: EventConsume) {
        unsafe {
            let cstr = CString::new(event_name).unwrap();
            (self.subscribe_event)(cstr.as_ptr(), Some(callback));
        }
    }

    pub fn unsubscribe_event(&self, event_name: &str, callback: EventConsume) {
        unsafe {
            let cstr = CString::new(event_name).unwrap();
            (self.unsubscribe_event)(cstr.as_ptr(), Some(callback));
        }
    }

    pub fn register_render(&self, render_callback: GuiRender) {
        unsafe {
            (self.register_render)(ERenderType_ERenderType_Render, Some(render_callback));
        }
    }

    pub fn register_pre_render(&self, render_callback: GuiRender) {
        unsafe {
            (self.register_render)(ERenderType_ERenderType_PreRender, Some(render_callback));
        }
    }

    pub fn register_options_render(&self, render_callback: GuiRender) {
        unsafe {
            (self.register_render)(ERenderType_ERenderType_OptionsRender, Some(render_callback));
        }
    }

    pub fn register_post_render(&self, render_callback: GuiRender) {
        unsafe {
            (self.register_render)(ERenderType_ERenderType_PostRender, Some(render_callback));
        }
    }

    pub fn unregister_render(&self, render_callback: GuiRender) {
        unsafe {
            (self.unregister_render)(Some(render_callback));
        }
    }
}
