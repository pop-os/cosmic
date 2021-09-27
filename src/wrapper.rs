use gio::{
    AppInfo,
    AppLaunchContext,
    Settings,
    Subprocess,
    SubprocessFlags,
    prelude::{
        AppInfoExt,
        SettingsExt,
    },
};
use glib::{
    translate::{
        FromGlibPtrNone,
        IntoGlib,
        ToGlibPtr,
    },
};
use glib_sys::{
    gboolean,
};
use libc::c_int;
use log::{
    error,
    info,
};
use meta::{
    KeyBinding,
    KeyBindingFlags,
    MonitorManager,
    Plugin,
    PluginExt,
    WindowActor,
};
use meta_sys::{
    MetaKeyBinding,
    MetaMotionDirection,
    MetaPlugin,
    MetaPluginInfo,
    MetaRectangle,
    MetaWindow,
    MetaWindowActor,
};
use std::{
    ffi::OsStr,
    ptr,
};

use crate::{Cosmic, Direction};

// Not #[repr(C)], so it is exported opaque
pub struct CosmicPluginData(Cosmic);

impl CosmicPluginData {
    fn new() -> Self {
        Self(Cosmic::new())
    }

    //TODO: is this safe?
    fn from_plugin<'a>(plugin: &'a Plugin) -> Option<&'a Self> {
        unsafe {
            let ptr = cosmic_plugin_data(plugin.to_glib_none().0);
            if ! ptr.is_null() {
                Some(&*ptr)
            } else {
                None
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn cosmic_plugin_data_init() -> *mut CosmicPluginData {
    Box::into_raw(Box::new(CosmicPluginData::new()))
}

//TODO: will this ever be used?
#[no_mangle]
pub unsafe extern "C" fn cosmic_plugin_data_free(data: *mut CosmicPluginData) {
    drop(Box::from_raw(data));
}

#[link(name = "wrapper", kind = "static")]
extern "C" {
    pub fn cosmic_plugin_get_type() -> glib_sys::GType;
    pub fn cosmic_plugin_data(plugin: *mut MetaPlugin) -> *mut CosmicPluginData;
}

pub fn with_cosmic<T, F: FnMut(&Cosmic) -> T>(plugin: &Plugin, mut f: F) -> Option<T> {
    match CosmicPluginData::from_plugin(plugin) {
        Some(data) => Some(f(&data.0)),
        None => {
            error!("failed to get cosmic plugin data");
            None
        },
    }
}

#[no_mangle]
pub extern "C" fn cosmic_plugin_confirm_display_change(plugin: *mut MetaPlugin) {
    let plugin = unsafe { Plugin::from_glib_none(plugin) };
    plugin.complete_display_change(true);
}

#[no_mangle]
pub extern "C" fn cosmic_plugin_destroy(plugin: *mut MetaPlugin, actor: *mut MetaWindowActor) {
    let plugin = unsafe { Plugin::from_glib_none(plugin) };
    let actor = unsafe { WindowActor::from_glib_none(actor) };
    plugin.destroy_completed(&actor);
}

#[no_mangle]
pub extern "C" fn cosmic_plugin_hide_tile_preview(_plugin: *mut MetaPlugin) {}

#[no_mangle]
pub extern "C" fn cosmic_plugin_info(_plugin: *mut MetaPlugin) -> *const MetaPluginInfo {
    ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn cosmic_plugin_keybinding_filter(plugin: *mut MetaPlugin, key_binding: *mut MetaKeyBinding) -> gboolean {
    let plugin = unsafe { Plugin::from_glib_none(plugin) };
    let mut key_binding = unsafe { KeyBinding::from_glib_none(key_binding) };
    with_cosmic(&plugin, |cosmic| {
        cosmic.keybinding_filter(&plugin, &mut key_binding)
    }).unwrap_or(false).into_glib()
}

#[no_mangle]
pub extern "C" fn cosmic_plugin_kill_switch_workspace(_plugin: *mut MetaPlugin) {}

#[no_mangle]
pub extern "C" fn cosmic_plugin_kill_window_effects(_plugin: *mut MetaPlugin, _actor: *mut MetaWindowActor) {}

#[no_mangle]
pub extern "C" fn cosmic_plugin_map(plugin: *mut MetaPlugin, actor: *mut MetaWindowActor) {
    let plugin = unsafe { Plugin::from_glib_none(plugin) };
    let actor = unsafe { WindowActor::from_glib_none(actor) };
    with_cosmic(&plugin, |cosmic| {
        cosmic.map(&plugin, &actor);
    });
    plugin.map_completed(&actor);
}

#[no_mangle]
pub extern "C" fn cosmic_plugin_minimize(plugin: *mut MetaPlugin, actor: *mut MetaWindowActor) {
    let plugin = unsafe { Plugin::from_glib_none(plugin) };
    let actor = unsafe { WindowActor::from_glib_none(actor) };
    plugin.minimize_completed(&actor);
}

#[no_mangle]
pub extern "C" fn cosmic_plugin_show_tile_preview(_plugin: *mut MetaPlugin, _window: *mut MetaWindow, _tile_rect: *mut MetaRectangle, _tile_monitor_number: c_int) {}

#[no_mangle]
pub extern "C" fn cosmic_plugin_size_changed(_plugin: *mut MetaPlugin, _actor: *mut MetaWindowActor) {}

#[no_mangle]
pub extern "C" fn cosmic_plugin_start(plugin: *mut MetaPlugin) {
    info!("STARTING COSMIC PLUGIN");

    let plugin = unsafe { Plugin::from_glib_none(plugin) };

    let display = plugin.display().expect("failed to find plugin display");

    with_cosmic(&plugin, |cosmic| {
        cosmic.start(&display);
    });

    match MonitorManager::get() {
        Some(monitor_manager) => {
            let plugin = plugin.clone();
            let display = display.clone();
            monitor_manager.connect_monitors_changed(move |_| {
                with_cosmic(&plugin, |cosmic| {
                    cosmic.on_monitors_changed(&display);
                });
            });
        },
        None => {
            error!("failed to find monitor manager");
        },
    }

    {
        let plugin = plugin.clone();
        display.connect_overlay_key(move |display| {
            info!("overlay key");
            with_cosmic(&plugin, |cosmic| {
                cosmic.toggle_launcher(&plugin, display);
            });
        });
    }

    //TODO: make gnome-settings-daemon media-keys function on its own
    {
        let settings = Settings::new("org.gnome.settings-daemon.plugins.media-keys");
        display.add_keybinding("terminal", &settings, KeyBindingFlags::NONE, |_display, _window, _key_event, _key_binding| {
            let settings = Settings::new("org.gnome.desktop.default-applications.terminal");
            let command = settings.string("exec");
            //TODO: launch context, launch with AppInfo::create_from_commandline
            match Subprocess::newv(&[OsStr::new(&command)], SubprocessFlags::NONE) {
                Ok(_subprocess) => (),
                Err(err) => {
                    error!("failed to launch terminal {:?}: {}", command, err);
                }
            }
        });
        display.add_keybinding("www", &settings, KeyBindingFlags::NONE, |_display, _window, _key_event, _key_binding| {
            if let Some(app_info) = AppInfo::default_for_uri_scheme("http") {
                //TODO: launch context?
                let context: Option<&AppLaunchContext> = None;
                match app_info.launch(&[], context) {
                    Ok(_) => (),
                    Err(err) => {
                        error!("failed to launch web browser: {}", err);
                    },
                }
            }
        });
    }

    //TODO: make these cosmic settings instead of gnome-shell
    {
        let settings = Settings::new("org.gnome.shell.keybindings");
        {
            let plugin = plugin.clone();
            display.add_keybinding("toggle-overview", &settings, KeyBindingFlags::NONE, move |display, _window, _key_event, _key_binding| {
                with_cosmic(&plugin, |cosmic| {
                    cosmic.toggle_ws_previews(&plugin, display);
                });
            });
        }
    }

    //TODO: make these cosmic settings instead of pop-shell
    {
        let settings = Settings::new("org.gnome.shell.extensions.pop-shell");
        {
            let plugin = plugin.clone();
            display.add_keybinding("focus-left", &settings, KeyBindingFlags::NONE, move |display, _window, _key_event, _key_binding| {
                with_cosmic(&plugin, |cosmic| {
                    cosmic.focus_direction(display, Direction::Left);
                });
            });
        }
        {
            let plugin = plugin.clone();
            display.add_keybinding("focus-right", &settings, KeyBindingFlags::NONE, move |display, _window, _key_event, _key_binding| {
                with_cosmic(&plugin, |cosmic| {
                    cosmic.focus_direction(display, Direction::Right);
                });
            });
        }
        {
            let plugin = plugin.clone();
            display.add_keybinding("focus-up", &settings, KeyBindingFlags::NONE, move |display, _window, _key_event, _key_binding| {
                with_cosmic(&plugin, |cosmic| {
                    cosmic.focus_direction(display, Direction::Up);
                });
            });
        }
        {
            let plugin = plugin.clone();
            display.add_keybinding("focus-down", &settings, KeyBindingFlags::NONE, move |display, _window, _key_event, _key_binding| {
                with_cosmic(&plugin, |cosmic| {
                    cosmic.focus_direction(display, Direction::Down);
                });
            });
        }
    }
}

#[no_mangle]
pub extern "C" fn cosmic_plugin_switch_workspace(plugin: *mut MetaPlugin, _from: c_int, _to: c_int, _direction: MetaMotionDirection) {
    let plugin = unsafe { Plugin::from_glib_none(plugin) };
    plugin.switch_workspace_completed();
}

#[no_mangle]
pub extern "C" fn cosmic_plugin_unminimize(plugin: *mut MetaPlugin, actor: *mut MetaWindowActor) {
    let plugin = unsafe { Plugin::from_glib_none(plugin) };
    let actor = unsafe { WindowActor::from_glib_none(actor) };
    plugin.unminimize_completed(&actor);
}
