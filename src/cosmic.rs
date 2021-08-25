use clutter::{
    Actor,
    ActorExt,
    Color,
    Text,
};
use log::error;
use meta::{
    Display,
    KeyBinding,
    ModalOptions,
    Plugin,
    PluginExt,
    TabList,
};
use std::{
    cell::RefCell,
};

use crate::Direction;

pub struct Cosmic {
    launcher_actor: RefCell<Option<Actor>>,
}

impl Cosmic {
    pub fn new() -> Self {
        Self {
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
            window.focus(Self::current_time(display));
        }
    }

    pub fn keybinding_filter(&self, plugin: &Plugin, key_binding: &KeyBinding) -> bool {
        println!("{:?}", key_binding);
        false
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

            let fg = Color::new(0xFF, 0xFF, 0xFF, 0xFF);
            let bg = Color::new(0x20, 0x20, 0x20, 0xFF);

            let actor = Actor::new();
            actor.set_position(600.0, 600.0);
            actor.set_size(300.0, 300.0);
            actor.set_background_color(Some(&bg));
            stage.add_child(&actor);

            let text = Text::new_full("IBM Plex Mono 48", "COSMIC", &fg);
            actor.add_child(&text);

            self.launcher_actor.replace(Some(actor));
        }
    }
}
