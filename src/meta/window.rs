use meta_sys::{
    MetaRectangle,
    MetaWindow,
    meta_window_focus,
    meta_window_get_frame_rect,
};

//TODO: OWNERSHIP!
pub struct Window(*mut MetaWindow);

impl Window {
    pub unsafe fn as_ptr(&mut self) -> *mut MetaWindow {
        self.0
    }

    pub unsafe fn from_ptr(ptr: *mut MetaWindow) -> Option<Self> {
        if ! ptr.is_null() {
            Some(Self(ptr))
        } else {
            None
        }
    }

    pub fn focus(&mut self, timestamp: u32) {
        unsafe { meta_window_focus(self.0, timestamp); }
    }

    pub fn get_frame_rect(&self) -> MetaRectangle {
        let mut rect = MetaRectangle {
            x: 0,
            y: 0,
            width: 0,
            height: 0
        };
        unsafe { meta_window_get_frame_rect(self.0, &mut rect); }
        rect
    }
}
