use clutter::{
    Actor,
    ActorExt,
    Color,
};
use clutter_sys::{
    CLUTTER_CURRENT_TIME,
    ClutterKeyEvent,
    clutter_actor_insert_child_below,
    clutter_actor_show,
    clutter_get_current_event_time,
};
use gdesktop_enums::{
    BackgroundStyle,
};
use glib::{
    Cast,
    translate::{FromGlibPtrNone, ToGlibPtr},
};
use glib_sys::{
    gpointer,
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
    Plugin,
    PluginExt,
    TabList,
    WindowActor,
};
use meta_sys::{
    META_KEY_BINDING_NONE,
    MetaDisplay,
    MetaKeyBinding,
    MetaMotionDirection,
    MetaPlugin,
    MetaPluginInfo,
    MetaRectangle,
    MetaWindow,
    MetaWindowActor,
    meta_display_add_keybinding,
    meta_get_stage_for_display,
    meta_get_window_group_for_display,
};
use std::{
    ptr
};

use crate::c_str;

#[repr(C)]
pub struct CosmicPluginData;

impl CosmicPluginData {
    pub fn new() -> Self {
        Self
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

extern "C" fn on_toggle_overview(
    _display: *mut MetaDisplay,
    _window: *mut MetaWindow,
    _key_event: *mut ClutterKeyEvent,
    _key_binding: *mut MetaKeyBinding,
    _data: gpointer
) {
    info!("on_toggle_overview");
}

enum Direction {
    Left,
    Right,
    Up,
    Down,
}

fn current_time(display: &Display) -> u32 {
    let time = display.current_time();
    if time != CLUTTER_CURRENT_TIME as u32 {
        return time;
    }
    unsafe { clutter_get_current_event_time() }
}

fn focus_direction(display: &Display, direction: Direction) {
    let workspace_manager = match display.workspace_manager() {
        Some(some) => some,
        None => {
            error!("failed to get workspace manager");
            return;
        }
    };
    let workspace = match workspace_manager.active_workspace() {
        Some(some) => some,
        None => {
            error!("failed to get active workspace");
            return;
        }
    };

    let current_window = match display.tab_current(TabList::NormalAll, &workspace) {
        Some(some) => some,
        None => return,
    };
    let current_rect = current_window.frame_rect();
    let (current_left, current_right, current_top, current_bottom) = (
        current_rect.x(),
        current_rect.x() + current_rect.width(),
        current_rect.y(),
        current_rect.y() + current_rect.height(),
    );

    let mut closest_dist = 0;
    let mut closest = None;
    let mut window = current_window.clone();
    loop {
        match display.tab_next(TabList::NormalAll, &workspace, Some(&window), false) {
            Some(some) => window = some,
            None => break,
        }

        if window.id() == current_window.id() {
            break;
        }

        let rect = window.frame_rect();
        let (window_left, window_right, window_top, window_bottom) = (
            rect.x(),
            rect.x() + rect.width(),
            rect.y(),
            rect.y() + rect.height(),
        );

        // Window is not intersecting vertically
        let out_of_bounds_vertical = || {
            window_top >= current_bottom || window_bottom <= current_top
        };
        // Window is not intersecting horizontally
        let out_of_bounds_horizontal = || {
            window_left >= current_right || window_right <= current_left
        };

        // The distance must be that of the shortest straight line that can be
        // drawn from the current window, in the specified direction, to the window
        // we are evaluating.
        let dist = match direction {
            Direction::Left => {
                if out_of_bounds_vertical() { continue; }
                if window_right <= current_left {
                    // To the left, with space
                    current_left - window_right
                } else if window_left <= current_left {
                    // To the left, overlapping
                    0
                } else {
                    // Not to the left, skipping
                    continue;
                }
            },
            Direction::Right => {
                if out_of_bounds_vertical() { continue; }
                if window_left >= current_right {
                    // To the right, with space
                    window_left - current_right
                } else if window_right >= current_right {
                    // To the right, overlapping
                    0
                } else {
                    // Not to the right, skipping
                    continue;
                }
            },
            Direction::Up => {
                if out_of_bounds_horizontal() { continue; }
                if window_bottom <= current_top {
                    // To the top, with space
                    current_top - window_bottom
                } else if window_top <= current_top {
                    // To the top, overlapping
                    0
                } else {
                    // Not to the top, skipping
                    continue;
                }
            },
            Direction::Down => {
                if out_of_bounds_horizontal() { continue; }
                if window_top >= current_bottom {
                    // To the bottom, with space
                    window_top - current_bottom
                } else if window_bottom >= current_bottom {
                    // To the bottom, overlapping
                    0
                } else {
                    // Not to the bottom, skipping
                    continue;
                }
            },
        };

        // Distance in wrong direction, skip
        if dist < 0 { continue; }

        // Save if closer than closest distance
        if dist < closest_dist || closest.is_none() {
            closest_dist = dist;
            closest = Some(window.clone());
        }
    }

    if let Some(window) = closest {
        window.focus(current_time(display));
    }
}

extern "C" fn on_focus_c(
    display: *mut MetaDisplay,
    _window: *mut MetaWindow,
    _key_event: *mut ClutterKeyEvent,
    _key_binding: *mut MetaKeyBinding,
    data: gpointer
) {
    let display = unsafe { Display::from_glib_none(display) };
    let direction = match data as usize {
        1 => Direction::Left,
        2 => Direction::Right,
        3 => Direction::Up,
        4 => Direction::Down,
        other => {
            error!("unknown direction {}", other);
            return;
        }
    };
    focus_direction(&display, direction);
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
pub extern "C" fn cosmic_plugin_kill_switch_workspace(_plugin: *mut MetaPlugin) {}

#[no_mangle]
pub extern "C" fn cosmic_plugin_kill_window_effects(_plugin: *mut MetaPlugin, _actor: *mut MetaWindowActor) {}

#[no_mangle]
pub extern "C" fn cosmic_plugin_map(plugin: *mut MetaPlugin, actor: *mut MetaWindowActor) {
    let plugin = unsafe { Plugin::from_glib_none(plugin) };
    let actor = unsafe { WindowActor::from_glib_none(actor) };
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

    let display = plugin.display().expect("no display found");

    let background_group = BackgroundGroup::new();
    unsafe {
        clutter_actor_insert_child_below(
            meta_get_window_group_for_display(display.to_glib_none().0),
            background_group.upcast_ref::<Actor>().to_glib_none().0,
            ptr::null_mut()
        );
    }

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

    unsafe { clutter_actor_show(meta_get_stage_for_display(display.to_glib_none().0)); }

    display.connect_overlay_key(|_display| {
        info!("overlay key");
    });

    let settings = gio::Settings::new("org.gnome.shell.keybindings");
    unsafe {
        meta_display_add_keybinding(
            display.to_glib_none().0,
            c_str!("toggle-overview"),
            settings.to_glib_none().0,
            META_KEY_BINDING_NONE,
            Some(on_toggle_overview),
            ptr::null_mut(),
            None,
        );
    }

    let settings = gio::Settings::new("org.gnome.shell.extensions.pop-shell");
    unsafe {
        meta_display_add_keybinding(
            display.to_glib_none().0,
            c_str!("focus-left"),
            settings.to_glib_none().0,
            META_KEY_BINDING_NONE,
            Some(on_focus_c),
            1 as *mut _,
            None,
        );
        meta_display_add_keybinding(
            display.to_glib_none().0,
            c_str!("focus-right"),
            settings.to_glib_none().0,
            META_KEY_BINDING_NONE,
            Some(on_focus_c),
            2 as *mut _,
            None,
        );
        meta_display_add_keybinding(
            display.to_glib_none().0,
            c_str!("focus-up"),
            settings.to_glib_none().0,
            META_KEY_BINDING_NONE,
            Some(on_focus_c),
            3 as *mut _,
            None,
        );
        meta_display_add_keybinding(
            display.to_glib_none().0,
            c_str!("focus-down"),
            settings.to_glib_none().0,
            META_KEY_BINDING_NONE,
            Some(on_focus_c),
            4 as *mut _,
            None,
        );
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
