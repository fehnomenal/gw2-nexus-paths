mod bindings;
mod wrapper;

use std::{
    ffi::{CStr, CString},
    path::PathBuf,
};

pub use bindings::*;
use log_err::{LogErrOption, LogErrResult};
pub use wrapper::AddonApiWrapper;

type TexturesReceiveCallback =
    unsafe extern "C" fn(aIdentifier: *const ::std::os::raw::c_char, aTexture: *mut Texture);

impl AddonAPI {
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
                .log_expect("could not load texture from url")(
                id.as_ptr(),
                origin.as_ptr(),
                path.as_ptr(),
                callback,
            );
        }
    }

    pub fn get_path_in_addon_directory(&self, path: &str) -> PathBuf {
        unsafe {
            let path =
                CString::new(format!("paths/{}", path)).log_expect("could not get addon directory");

            let dir = self.Paths.GetAddonDirectory.log_unwrap()(path.as_ptr());

            PathBuf::from(
                CString::from_raw(dir.cast_mut())
                    .to_str()
                    .log_expect("could not get addon directory"),
            )
        }
    }
}
