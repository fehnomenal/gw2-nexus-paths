use std::ffi::{c_char, c_int, c_short, c_void};

#[repr(C)]
#[allow(non_snake_case)]
pub struct AddonAPI<'a> {
    SwapChain: &'a c_void,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct AddonDefinition {
    /* required */
    pub Signature: c_int,
    pub APIVersion: c_int,
    pub Name: *const c_char,
    pub Version: AddonVersion,
    pub Author: *const c_char,
    pub Description: *const c_char,
    pub Load: extern "C" fn(&AddonAPI),
    pub Unload: extern "C" fn(),
    pub Flags: EAddonFlags,

    /* update fallback */
    pub Provider: EUpdateProvider,
    pub UpdateLink: *const c_char,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct AddonVersion {
    pub Major: c_short,
    pub Minor: c_short,
    pub Build: c_short,
    pub Revision: c_short,
}

#[repr(C)]
#[allow(dead_code)]
pub enum EAddonFlags {
    None = 0,
    /// is hooking functions or doing anything else that's volatile and game build dependant
    IsVolatile = 1,
    /// prevents unloading at runtime, aka. will require a restart if updated, etc.
    DisableHotloading = 2,
    /// prevents loading the addon later than the initial character select
    OnlyLoadDuringGameLaunchSequence = 4,
}

#[repr(C)]
#[allow(dead_code)]
pub enum EUpdateProvider {
    /// Does not support auto updating
    None = 0,
    /// Provider is Raidcore (via API)
    Raidcore = 1,
    /// Provider is GitHub Releases
    GitHub = 2,
    /// Provider is direct file link
    Direct = 3,
    /// Provider is self check, addon has to request manually and version will not be verified
    _Self = 4,
}
