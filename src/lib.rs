use glib::prelude::*;
use std::{
    env,
    ffi::CString,
    os::unix::ffi::OsStringExt,
    process,
    ptr,
};

pub use self::cosmic::Cosmic;
mod cosmic;

pub use self::launcher::{LauncherIpc, LauncherUi};
mod launcher;

pub use self::theme::Theme;
mod theme;

pub use self::widget::{Icon, RoundedRect};
mod widget;

pub use self::ws_previews::WsPreviews;
mod ws_previews;

pub use self::wrapper::CosmicPlugin;
mod wrapper;

#[macro_export]
macro_rules! c_str {
    ($str:expr) => {
        concat!($str, "\0").as_ptr() as *const libc::c_char
    }
}

pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

pub fn run() {
    unsafe {
        // Nasty stuff to convert to C arguments
        let mut args = env::args_os().map(|arg_os| {
            CString::new(arg_os.into_vec()).unwrap().into_raw()
        }).collect::<Vec<_>>();
        let mut argc = args.len() as i32;
        args.push(ptr::null_mut());
        let mut argv = args.as_mut_ptr();

        // Allow mutter to handle arguments
        let ctx = meta_sys::meta_get_option_context();
        let mut error = ptr::null_mut();
        if glib_sys::g_option_context_parse(
            ctx,
            &mut argc,
            &mut argv,
            &mut error
        ) == glib_sys::GFALSE {
            glib_sys::g_printerr(
                c_str!("%s: %s\n"),
                args[0],
                (*error).message
            );
            process::exit(1);
        }
        glib_sys::g_option_context_free(ctx);

        // Run mutter
        meta::Plugin::manager_set_plugin_type(CosmicPlugin::static_type());
        meta::set_wm_name("COSMIC");
        meta::init();
        meta::register_with_session();
        process::exit(meta::run());
    }
}
