use clutter::{
    Actor,
    ActorExt,
    Color,
};
use clutter_sys::{
    ClutterKeyEvent,
};
use gdesktop_enums::{
    BackgroundStyle,
};
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
    Cast,
    translate::{
        FromGlibPtrNone,
        IntoGlib,
        ToGlibPtr,
    },
};
use glib_sys::{
    gboolean,
    gpointer,
    GFALSE,
    GTRUE,
};
use libc::c_int;
use log::{
    error,
    info,
};
use meta::{
    Background,
    BackgroundActor,
    BackgroundContent,
    BackgroundGroup,
    Display,
    KeyBinding,
    KeyBindingFlags,
    Plugin,
    PluginExt,
    WindowActor,
};
use meta_sys::{
    MetaDisplay,
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

fn with_cosmic<T, F: FnMut(&Cosmic) -> T>(plugin: &Plugin, mut f: F) -> Option<T> {
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

    let background_group = BackgroundGroup::new();
    meta::functions::window_group_for_display(&display)
        .expect("failed to find display window group")
        .insert_child_below::<_, Actor>(&background_group, None);

    let mut color = Color::new(128, 128, 128, 255);

    let background_file = gio::File::for_path(
        "/usr/share/backgrounds/pop/kate-hazen-COSMIC-desktop-wallpaper.png"
    );

    for monitor in 0..display.n_monitors() {
        let rect = display.monitor_geometry(monitor);

        let background_actor = BackgroundActor::new(&display, monitor);
        let content = background_actor.content().expect("no BackgroundActor content");
        let background_content = content.downcast::<BackgroundContent>()
            .expect("failed to downcast BackgroundActor content to BackgroundContent");

        background_actor.set_position(rect.x() as f32, rect.y() as f32);
        background_actor.set_size(rect.width() as f32, rect.height() as f32);

        let background = Background::new(&display);
        background.set_color(&mut color);
        background.set_file(Some(&background_file), BackgroundStyle::Zoom);
        background_content.set_background(&background);

        background_group.add_child(&background_actor);
    }

    meta::functions::stage_for_display(&display)
        .expect("failed to find display stage")
        .show();

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
            app_info.launch::<AppLaunchContext>(&[], None);
        }
    });

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
