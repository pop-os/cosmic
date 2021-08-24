use meta_sys::{
    MetaWorkspace,
};

//TODO: OWNERSHIP!
pub struct Workspace(*mut MetaWorkspace);

impl Workspace {
    pub unsafe fn as_ptr(&mut self) -> *mut MetaWorkspace {
        self.0
    }

    pub unsafe fn from_ptr(ptr: *mut MetaWorkspace) -> Option<Self> {
        if ! ptr.is_null() {
            Some(Self(ptr))
        } else {
            None
        }
    }
}
