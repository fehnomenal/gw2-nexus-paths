use std::ffi::CStr;

use crate::AddonAPI;

use super::{AddonApiWrapper, Cleanup};

pub type EventConsume = unsafe extern "C" fn(aEventArgs: *mut ::std::os::raw::c_void);

impl AddonApiWrapper {
    pub unsafe fn subscribe_event(&mut self, event_name: &'static CStr, callback: EventConsume) {
        self.cleanups
            .push(Box::new(EventWrapper::new(&self, event_name, callback)));
    }
}

struct EventWrapper(&'static CStr, EventConsume);

impl EventWrapper {
    unsafe fn new(api: &AddonAPI, event_name: &'static CStr, callback: EventConsume) -> Self {
        api.Events.Subscribe.expect("Cannot subscribe to event")(
            event_name.as_ptr(),
            Some(callback),
        );

        Self(event_name, callback)
    }
}

impl Cleanup for EventWrapper {
    unsafe fn cleanup(&mut self, api: &AddonAPI) {
        api.Events
            .Unsubscribe
            .expect("Cannot unsubscribe from event")(self.0.as_ptr(), Some(self.1));
    }
}
