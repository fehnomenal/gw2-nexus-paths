#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use std::{
    ffi::{CStr, CString},
    path::PathBuf,
};

include!("./bindings-nexus-api.rs");

pub mod mumble {
    include!("./bindings-mumble-api.rs");
}

type EventConsume = unsafe extern "C" fn(aEventArgs: *mut ::std::os::raw::c_void);

type GuiRender = unsafe extern "C" fn();

type WndProcCallback =
    unsafe extern "C" fn(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> UINT;

type KeybindsProcess =
    unsafe extern "C" fn(aIdentifier: *const ::std::os::raw::c_char, aIsRelease: bool);

type TexturesReceiveCallback =
    unsafe extern "C" fn(aIdentifier: *const ::std::os::raw::c_char, aTexture: *mut Texture);

impl AddonAPI {
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
                ERenderType_ERenderType_Render,
                Some(render_callback),
            );
        }
    }

    pub fn register_pre_render(&self, render_callback: GuiRender) {
        unsafe {
            self.Renderer
                .Register
                .expect("Could not register pre render callback")(
                ERenderType_ERenderType_PreRender,
                Some(render_callback),
            );
        }
    }

    pub fn register_options_render(&self, render_callback: GuiRender) {
        unsafe {
            self.Renderer
                .Register
                .expect("Could not register options render callback")(
                ERenderType_ERenderType_OptionsRender,
                Some(render_callback),
            );
        }
    }

    pub fn register_post_render(&self, render_callback: GuiRender) {
        unsafe {
            self.Renderer
                .Register
                .expect("Could not register post render callback")(
                ERenderType_ERenderType_PostRender,
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

    pub fn register_key_binding(
        &self,
        id: &CStr,
        callback: KeybindsProcess,
        binding: Option<&CStr>,
    ) {
        unsafe {
            self.InputBinds
                .RegisterWithString
                .expect("Could not register key binding")(
                id.as_ptr(),
                Some(callback),
                binding.unwrap_or(c"(none)").as_ptr(),
            );
        }
    }

    pub fn unregister_key_binding(&self, id: &CStr) {
        unsafe {
            self.InputBinds
                .Deregister
                .expect("Could not unregister key binding")(id.as_ptr());
        }
    }

    pub fn register_shortcut(
        &self,
        id: &CStr,
        textureId: &CStr,
        hoverTextureId: Option<&CStr>,
        keybindingId: &CStr,
        tooltip: &CStr,
    ) {
        unsafe {
            self.QuickAccess.Add.expect("Could not register shortcut")(
                id.as_ptr(),
                textureId.as_ptr(),
                hoverTextureId.unwrap_or(textureId).as_ptr(),
                keybindingId.as_ptr(),
                tooltip.as_ptr(),
            );
        }
    }

    pub fn unregister_shortcut(&self, id: &CStr) {
        unsafe {
            self.QuickAccess
                .Remove
                .expect("Could not unregister shortcut")(id.as_ptr());
        }
    }

    pub fn load_texture_from_url(
        &self,
        id: &CStr,
        origin: &CStr,
        path: &CStr,
        callback: Option<TexturesReceiveCallback>,
    ) {
        unsafe {
            self.Textures
                .LoadFromURL
                .expect("Could not load texture from url")(
                id.as_ptr(),
                origin.as_ptr(),
                path.as_ptr(),
                callback,
            );
        }
    }

    pub fn getAddonDirectory(&self, path: &str) -> PathBuf {
        unsafe {
            let path =
                CString::new(format!("paths/{}", path)).expect("Could not get addon directory");

            let dir = self.Paths.GetAddonDirectory.unwrap()(path.as_ptr());

            PathBuf::from(
                CString::from_raw(dir.cast_mut())
                    .to_str()
                    .expect("Could not get addon directory"),
            )
        }
    }
}
