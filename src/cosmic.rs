use clutter::{
    Actor,
    ActorExt,
    Color,
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
    DisplayCorner,
    KeyBinding,
    ModalOptions,
    Plugin,
    PluginExt,
    TabList,
    WindowActor,
};
use pop_launcher::{
    Request,
    Response,
};
use std::{
    cell::RefCell,
    io,
    rc::Rc,
};

use crate::{
    Direction,
    LauncherIpc,
    LauncherUi,
};

pub struct Cosmic {
    background_group: BackgroundGroup,
    launcher_ipc: RefCell<Option<LauncherIpc>>,
    launcher_ui: RefCell<Option<Rc<LauncherUi>>>,
}

impl Cosmic {
    pub fn new() -> Self {
        let launcher_ipc = match LauncherIpc::new() {
            Ok(ok) => Some(ok),
            Err(err) => {
                error!("failed to create LauncherIpc: {}", err);
                None
            }
        };
        Self {
            background_group: BackgroundGroup::new(),
            launcher_ipc: RefCell::new(launcher_ipc),
            launcher_ui: RefCell::new(None),
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
        // We use GTK to supply icons, it must be initialized when mutter is ready
        gtk::init().expect("failed to initialize gtk");

        match display.workspace_manager() {
            Some(workspace_manager) => {
                // Default to vertical workspaces
                workspace_manager.override_workspace_layout(
                    DisplayCorner::Topleft,
                    true,
                    -1,
                    1
                );
            },
            None => error!("failed to find workspace manager"),
        }

        meta::functions::window_group_for_display(&display)
            .expect("failed to find display window group")
            .insert_child_below::<_, Actor>(&self.background_group, None);

        self.on_monitors_changed(display);

        meta::functions::stage_for_display(&display)
            .expect("failed to find display stage")
            .show();
    }

    pub fn launcher_request(&self, request: Request) -> io::Result<Response> {
        match &mut *self.launcher_ipc.borrow_mut() {
            Some(launcher_ipc) => launcher_ipc.request(request),
            None => Err(io::Error::new(io::ErrorKind::NotFound, "launcher ipc not found")),
        }
    }

    pub fn toggle_launcher(&self, plugin: &Plugin, display: &Display) {
        let stage = match meta::functions::stage_for_display(&display) {
            Some(some) => some,
            None => {
                error!("failed to find display stage");
                return;
            }
        };

        if let Some(launcher_ui) = self.launcher_ui.replace(None) {
            stage.remove_child(launcher_ui.rect.actor());

            plugin.end_modal(Self::current_time(display));
        } else {
            plugin.begin_modal(ModalOptions::empty(), Self::current_time(display));

            let launcher_ui = LauncherUi::new(&stage, plugin, display);
            self.launcher_ui.replace(Some(launcher_ui));
        }
    }
}
