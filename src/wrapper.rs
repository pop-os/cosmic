use clutter::{
    Actor,
    traits::ActorExt,
};
use clutter_sys::{
    CLUTTER_CURRENT_TIME,
    ClutterColor,
    ClutterKeyEvent,
    clutter_actor_add_child,
    clutter_actor_insert_child_below,
    clutter_actor_show,
    clutter_get_current_event_time,
};
use gdesktop_enums_sys::{
    G_DESKTOP_BACKGROUND_STYLE_ZOOM,
};
use glib::{
    Cast,
    translate::{FromGlibPtrNone, ToGlibPtr},
};
use glib_sys::{
    GTRUE,
    gpointer,
};
use gobject_sys::{
    g_object_unref,
};
use libc::c_int;
use log::{
    error,
    info,
};
use meta::{
    BackgroundActor,
    BackgroundContent,
    Display,
    Plugin,
    TabList,
    traits::PluginExt,
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
    meta_background_content_set_background,
    meta_background_group_new,
    meta_background_new,
    meta_background_set_color,
    meta_background_set_file,
    meta_display_add_keybinding,
    meta_get_stage_for_display,
    meta_get_window_group_for_display,
    meta_plugin_complete_display_change,
    meta_plugin_destroy_completed,
    meta_plugin_map_completed,
    meta_plugin_minimize_completed,
    meta_plugin_switch_workspace_completed,
    meta_plugin_unminimize_completed,
    meta_window_actor_get_meta_window,
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

unsafe extern "C" fn on_toggle_overview(
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

unsafe extern "C" fn on_focus_c(
    display: *mut MetaDisplay,
    _window: *mut MetaWindow,
    _key_event: *mut ClutterKeyEvent,
    _key_binding: *mut MetaKeyBinding,
    data: gpointer
) {
    let display = Display::from_glib_none(display);
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
pub unsafe extern "C" fn cosmic_plugin_confirm_display_change(plugin: *mut MetaPlugin) {
    meta_plugin_complete_display_change(plugin, GTRUE);
}

#[no_mangle]
pub unsafe extern "C" fn cosmic_plugin_destroy(plugin: *mut MetaPlugin, actor: *mut MetaWindowActor) {
    meta_plugin_destroy_completed(plugin, actor);
}

#[no_mangle]
pub unsafe extern "C" fn cosmic_plugin_hide_tile_preview(plugin: *mut MetaPlugin) {}

#[no_mangle]
pub unsafe extern "C" fn cosmic_plugin_info(plugin: *mut MetaPlugin) -> *const MetaPluginInfo {
    ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn cosmic_plugin_kill_switch_workspace(plugin: *mut MetaPlugin) {}

#[no_mangle]
pub unsafe extern "C" fn cosmic_plugin_kill_window_effects(plugin: *mut MetaPlugin, actor: *mut MetaWindowActor) {}

#[no_mangle]
pub unsafe extern "C" fn cosmic_plugin_map(plugin: *mut MetaPlugin, actor: *mut MetaWindowActor) {
    let window = meta_window_actor_get_meta_window(actor);
    //meta_window_move_resize_frame(window, GTRUE, 0, 0, 1920, 1080);
    meta_plugin_map_completed(plugin, actor);
}

#[no_mangle]
pub unsafe extern "C" fn cosmic_plugin_minimize(plugin: *mut MetaPlugin, actor: *mut MetaWindowActor) {
    meta_plugin_minimize_completed(plugin, actor);
}

#[no_mangle]
pub unsafe extern "C" fn cosmic_plugin_show_tile_preview(plugin: *mut MetaPlugin, window: *mut MetaWindow, tile_rect: *mut MetaRectangle, tile_monitor_number: c_int) {}

#[no_mangle]
pub unsafe extern "C" fn cosmic_plugin_size_changed(plugin: *mut MetaPlugin, actor: *mut MetaWindowActor) {}

#[no_mangle]
pub unsafe extern "C" fn cosmic_plugin_start(plugin: *mut MetaPlugin) {
    info!("STARTING COSMIC PLUGIN");

    let plugin = Plugin::from_glib_none(plugin);

    let display = plugin.display().expect("no display found");

    let background_group = meta_background_group_new();
    clutter_actor_insert_child_below(meta_get_window_group_for_display(display.to_glib_none().0), background_group, ptr::null_mut());

    let mut color = ClutterColor {
        red: 128,
        green: 128,
        blue: 128,
        alpha: 255,
    };

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

        let background = meta_background_new(display.to_glib_none().0);
        meta_background_set_color(background, &mut color);
        meta_background_set_file(background, background_file.to_glib_none().0, G_DESKTOP_BACKGROUND_STYLE_ZOOM);
        meta_background_content_set_background(background_content.to_glib_none().0, background);
        g_object_unref(background as *mut _);

        clutter_actor_add_child(background_group, background_actor.upcast::<Actor>().to_glib_none().0);
    }

    clutter_actor_show(meta_get_stage_for_display(display.to_glib_none().0));

    display.connect_overlay_key(|_display| {
        info!("overlay key");
    });

    let settings = gio::Settings::new("org.gnome.shell.keybindings");
    meta_display_add_keybinding(
        display.to_glib_none().0,
        c_str!("toggle-overview"),
        settings.to_glib_none().0,
        META_KEY_BINDING_NONE,
        Some(on_toggle_overview),
        ptr::null_mut(),
        None,
    );

    let settings = gio::Settings::new("org.gnome.shell.extensions.pop-shell");
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

#[no_mangle]
pub unsafe extern "C" fn cosmic_plugin_switch_workspace(plugin: *mut MetaPlugin, from: c_int, to: c_int, direction: MetaMotionDirection) {
    meta_plugin_switch_workspace_completed(plugin);
}

#[no_mangle]
pub unsafe extern "C" fn cosmic_plugin_unminimize(plugin: *mut MetaPlugin, actor: *mut MetaWindowActor) {
    meta_plugin_unminimize_completed(plugin, actor);
}
