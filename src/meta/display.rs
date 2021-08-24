use meta_sys::{
    MetaDisplay,
    meta_display_get_current_time,
    meta_display_get_workspace_manager,
};

use crate::meta::WorkspaceManager;

//TODO: OWNERSHIP!
pub struct Display(*mut MetaDisplay);

impl Display {
    pub unsafe fn as_ptr(&mut self) -> *mut MetaDisplay {
        self.0
    }

    pub unsafe fn from_ptr(ptr: *mut MetaDisplay) -> Option<Self> {
        if ! ptr.is_null() {
            Some(Self(ptr))
        } else {
            None
        }
    }

    pub fn get_workspace_manager(&mut self) -> Option<WorkspaceManager> {
        unsafe { WorkspaceManager::from_ptr(meta_display_get_workspace_manager(self.0)) }
    }

    pub fn get_current_time(&mut self) -> u32 {
        unsafe { meta_display_get_current_time(self.0) }
    }
}
