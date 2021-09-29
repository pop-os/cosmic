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
use log::{
    error,
    info,
};
use meta::{
    KeyBindingFlags,
    MonitorManager,
    PluginExt,
    subclass::prelude::*,
};
use std::{
    ffi::OsStr,
};

use crate::{Cosmic, Direction};

pub struct CosmicPluginInner(Cosmic);

#[glib::object_subclass]
impl ObjectSubclass for CosmicPluginInner {
    const NAME: &'static str = "S76CosmicPlugin";
    type ParentType = meta::Plugin;
    type Type = CosmicPlugin;

    fn new() -> Self  {
        Self(Cosmic::new())
    }
}

impl ObjectImpl for CosmicPluginInner {}

impl PluginImpl for CosmicPluginInner {
    fn confirm_display_change(&self, plugin: &CosmicPlugin) {
        plugin.complete_display_change(true);
    }

    fn destroy(&self, plugin: &CosmicPlugin, actor: &meta::WindowActor) {
        plugin.destroy_completed(actor);
    }

    fn hide_tile_preview(&self, _plugin: &CosmicPlugin) {}

    fn plugin_info(&self, _plugin: &CosmicPlugin) -> Option<&'static meta::ffi::MetaPluginInfo> {
        None
    }

    fn keybinding_filter(&self, plugin: &CosmicPlugin, key_binding: &meta::KeyBinding) -> bool {
        plugin.cosmic().keybinding_filter(plugin, key_binding)
    }

    fn kill_switch_workspace(&self, _plugin: &CosmicPlugin) {}

    fn kill_window_effects(&self, _plugin: &CosmicPlugin, _actor: &meta::WindowActor) {}

    fn map(&self, plugin: &CosmicPlugin, actor: &meta::WindowActor) {
        plugin.cosmic().map(plugin, actor);
        plugin.map_completed(actor);
    }

    fn minimize(&self, plugin: &CosmicPlugin, actor: &meta::WindowActor) {
        plugin.minimize_completed(actor);
    }

    fn show_tile_preview(&self, _plugin: &CosmicPlugin, _window: &meta::Window, _tile_rect: &meta::Rectangle, _tile_monitor_number: i32) {}

    fn size_changed(&self, _plugin: &CosmicPlugin, _actor: &meta::WindowActor) {}

    fn start(&self, plugin: &CosmicPlugin) {
        info!("STARTING COSMIC PLUGIN");

        let display = plugin.display().expect("failed to find plugin display");

        plugin.cosmic().start(&display);

        match MonitorManager::get() {
            Some(monitor_manager) => {
                let plugin = plugin.clone();
                let display = display.clone();
                monitor_manager.connect_monitors_changed(move |_| {
                    plugin.cosmic().on_monitors_changed(&display);
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
                plugin.cosmic().toggle_launcher(&plugin, display);
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
                    plugin.cosmic().toggle_ws_previews(&plugin, display);
                });
            }
        }

        //TODO: make these cosmic settings instead of pop-shell
        {
            let settings = Settings::new("org.gnome.shell.extensions.pop-shell");
            {
                let plugin = plugin.clone();
                display.add_keybinding("focus-left", &settings, KeyBindingFlags::NONE, move |display, _window, _key_event, _key_binding| {
                    plugin.cosmic().focus_direction(display, Direction::Left);
                });
            }
            {
                let plugin = plugin.clone();
                display.add_keybinding("focus-right", &settings, KeyBindingFlags::NONE, move |display, _window, _key_event, _key_binding| {
                    plugin.cosmic().focus_direction(display, Direction::Right);
                });
            }
            {
                let plugin = plugin.clone();
                display.add_keybinding("focus-up", &settings, KeyBindingFlags::NONE, move |display, _window, _key_event, _key_binding| {
                    plugin.cosmic().focus_direction(display, Direction::Up);
                });
            }
            {
                let plugin = plugin.clone();
                display.add_keybinding("focus-down", &settings, KeyBindingFlags::NONE, move |display, _window, _key_event, _key_binding| {
                    plugin.cosmic().focus_direction(display, Direction::Down);
                });
            }
        }
    }

    fn switch_workspace(&self, plugin: &CosmicPlugin, _from: i32, _to: i32, _direction: meta::MotionDirection) {
        plugin.switch_workspace_completed();
    }

    fn unminimize(&self, plugin: &CosmicPlugin, actor: &meta::WindowActor) {
        plugin.unminimize_completed(actor);
    }
}

glib::wrapper! {
    pub struct CosmicPlugin(ObjectSubclass<CosmicPluginInner>)
        @extends meta::Plugin;
}

impl CosmicPlugin {
    pub fn cosmic(&self) -> &Cosmic {
        &CosmicPluginInner::from_instance(self).0
    }
}
