use meta_sys::{
    MetaWorkspaceManager,
    meta_workspace_manager_get_active_workspace,
};

use crate::meta::Workspace;

//TODO: OWNERSHIP!
pub struct WorkspaceManager(*mut MetaWorkspaceManager);

impl WorkspaceManager {
    pub unsafe fn as_ptr(&mut self) -> *mut MetaWorkspaceManager {
        self.0
    }

    pub unsafe fn from_ptr(ptr: *mut MetaWorkspaceManager) -> Option<Self> {
        if ! ptr.is_null() {
            Some(Self(ptr))
        } else {
            None
        }
    }

    pub fn get_active_workspace(&mut self) -> Option<Workspace> {
        unsafe { Workspace::from_ptr(meta_workspace_manager_get_active_workspace(self.0)) }
    }
}
