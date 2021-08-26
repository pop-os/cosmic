use clutter::{
    Actor,
    ActorExt,
    Color,
    Text,
    TextExt,
};
use gdesktop_enums::{
    BackgroundStyle,
};
use glib::{
    Cast,
};
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
    ModalOptions,
    Plugin,
    PluginExt,
    TabList,
    WindowActor,
};
use std::{
    cell::RefCell,
};

use crate::{
    Direction,
    wrapper::with_cosmic,
};

pub struct Cosmic {
    background_group: BackgroundGroup,
    launcher_actor: RefCell<Option<Actor>>,
}

impl Cosmic {
    pub fn new() -> Self {
        Self {
            background_group: BackgroundGroup::new(),
            launcher_actor: RefCell::new(None),
        }
    }

    pub fn current_time(display: &Display) -> u32 {
        let time = display.current_time();
        if time != clutter_sys::CLUTTER_CURRENT_TIME as u32 {
            return time;
        }
        unsafe { clutter_sys::clutter_get_current_event_time() }
    }

    pub fn focus_direction(&self, display: &Display, direction: Direction) {
        let workspace_manager = match display.workspace_manager() {
            Some(some) => some,
            None => {
                error!("failed to find workspace manager");
                return;
            }
        };
        let workspace = match workspace_manager.active_workspace() {
            Some(some) => some,
            None => {
                error!("failed to find active workspace");
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
            window.activate(Self::current_time(display));
            window.raise();
        }
    }

    pub fn keybinding_filter(&self, plugin: &Plugin, key_binding: &mut KeyBinding) -> bool {
        info!("key_binding {:?} builtin {}", key_binding.name(), key_binding.is_builtin());
        false
    }

    pub fn map(&self, plugin: &Plugin, actor: &WindowActor) {
        if let Some(window) = actor.meta_window() {
            let display = match plugin.display() {
                Some(some) => some,
                None => {
                    error!("failed to find plugin display");
                    return;
                },
            };
            window.activate(Self::current_time(&display));
            window.raise();
        }
    }

    pub fn on_monitors_changed(&self, display: &Display) {
        self.background_group.destroy_all_children();

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

            self.background_group.add_child(&background_actor);
        }
    }

    pub fn start(&self, display: &Display) {
        meta::functions::window_group_for_display(&display)
            .expect("failed to find display window group")
            .insert_child_below::<_, Actor>(&self.background_group, None);

        self.on_monitors_changed(display);

        meta::functions::stage_for_display(&display)
            .expect("failed to find display stage")
            .show();
    }

    pub fn toggle_launcher(&self, plugin: &Plugin, display: &Display) {
        let stage = match meta::functions::stage_for_display(&display) {
            Some(some) => some,
            None => {
                error!("failed to find display stage");
                return;
            }
        };

        if let Some(actor) = self.launcher_actor.replace(None) {
            stage.remove_child(&actor);

            plugin.end_modal(Self::current_time(display));
        } else {
            plugin.begin_modal(ModalOptions::empty(), Self::current_time(display));

            let color_fg = Color::new(0xFF, 0xFF, 0xFF, 0xFF);
            let color_sel = Color::new(0x00, 0x7F, 0xFF, 0xFF);
            let color_bg = Color::new(0x20, 0x20, 0x20, 0xFF);

            let (launcher_w, launcher_h) = (480.0, 48.0);
            let (stage_w, stage_h) = stage.size();
            let launcher_x = (stage_w - launcher_w) / 2.0;
            let launcher_y = (stage_h - launcher_h) / 2.0;

            let actor = Actor::new();
            actor.set_position(launcher_x, launcher_y);
            actor.set_size(launcher_w, launcher_h);
            actor.set_background_color(Some(&color_bg));
            stage.add_child(&actor);

            let text_actor = Text::new_full("IBM Plex Mono 16", "", &color_fg);
            ActorExt::set_position(&text_actor, 8.0, 8.0);
            text_actor.set_activatable(true);
            text_actor.set_cursor_visible(true);
            text_actor.set_editable(true);
            text_actor.set_reactive(true);
            text_actor.set_selectable(true);
            text_actor.set_selection_color(Some(&color_sel));
            {
                let plugin = plugin.clone();
                let display = display.clone();
                text_actor.connect_activate(move |text_actor| {
                    info!("launcher: {:?}", text_actor.text());
                    // Close launcher on enter
                    with_cosmic(&plugin, |cosmic| {
                        cosmic.toggle_launcher(&plugin, &display);
                    });
                });
            }
            {
                let plugin = plugin.clone();
                let display = display.clone();
                text_actor.connect_key_press_event(move |_, key_event| {
                    match char::from_u32(key_event.unicode_value) {
                        Some('\u{1b}') => {
                            // Close launcher on escape
                            with_cosmic(&plugin, |cosmic| {
                                cosmic.toggle_launcher(&plugin, &display);
                            });
                            true
                        },
                        _ => false
                    }
                });
            }
            actor.add_child(&text_actor);
            text_actor.grab_key_focus();
            //TODO: set clutter backend default input method so there are no errors

            self.launcher_actor.replace(Some(actor));
        }
    }
}
