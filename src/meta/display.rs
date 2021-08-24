use glib_sys::{
    GFALSE,
    GTRUE,
};
use meta_sys::{
    MetaDisplay,
    MetaRectangle,
    MetaTabList,
    meta_display_get_current_time,
    meta_display_get_monitor_geometry,
    meta_display_get_n_monitors,
    meta_display_get_tab_current,
    meta_display_get_tab_next,
    meta_display_get_workspace_manager,
};

use crate::meta::{
    Window,
    Workspace,
    WorkspaceManager,
};

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

    pub fn get_monitor_geometry(&mut self, monitor: libc::c_int) -> MetaRectangle {
        let mut rect = MetaRectangle {
            x: 0,
            y: 0,
            width: 0,
            height: 0
        };
        unsafe { meta_display_get_monitor_geometry(self.0, monitor, &mut rect); }
        rect
    }

    pub fn get_n_monitors(&mut self) -> libc::c_int {
        unsafe { meta_display_get_n_monitors(self.0) }
    }

    pub fn get_tab_current(
        &mut self,
        type_: MetaTabList,
        workspace: &mut Workspace
    ) -> Option<Window> {
        unsafe {
            Window::from_ptr(meta_display_get_tab_current(
                self.0,
                type_,
                workspace.as_ptr()
            ))
        }
    }

    pub fn get_tab_next(
        &mut self,
        type_: MetaTabList,
        workspace: &mut Workspace,
        window: &mut Window,
        backward: bool
    ) -> Option<Window> {
        unsafe {
            Window::from_ptr(meta_display_get_tab_next(
                self.0,
                type_,
                workspace.as_ptr(),
                window.as_ptr(),
                if backward { GTRUE } else { GFALSE }
            ))
        }
    }
}
