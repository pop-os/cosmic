use meta_sys::{
    MetaPlugin,
    meta_plugin_get_display,
};

use crate::meta::Display;

//TODO: OWNERSHIP!
pub struct Plugin(*mut MetaPlugin);

impl Plugin {
    pub unsafe fn as_ptr(&mut self) -> *mut MetaPlugin {
        self.0
    }

    pub unsafe fn from_ptr(ptr: *mut MetaPlugin) -> Option<Self> {
        if ! ptr.is_null() {
            Some(Self(ptr))
        } else {
            None
        }
    }

    pub fn get_display(&mut self) -> Option<Display> {
        unsafe { Display::from_ptr(meta_plugin_get_display(self.0)) }
    }
}
