use clutter::{
    Actor,
    ActorExt,
    Clone,
};
use glib::{
    Cast,
};
use log::{
    error,
};
use meta::{
    Display,
    Plugin,
    Rectangle,
    TabList,
    Window,
    WindowActor,
    Workspace,
};
use std::{
    rc::Rc,
};

use crate::{
    RoundedRect,
    Theme,
};

pub struct WsPreviewMonitor {
    pub rect: RoundedRect,
    previews: Vec<RoundedRect>,
}

impl WsPreviewMonitor {
    pub fn new(parent: &Actor, monitor_rect: Rectangle, active_workspace: i32, workspaces: &[Workspace], windows: &[Window]) -> Self {
        let border_radius = 5.0;
        let color_background = Theme::color_background();
        let color_border = Theme::color_border();
        let color_input = Theme::color_input();
        let margin = 16;
        let padding = 8;
        let scale = 8;

        let (parent_w, parent_h) = (monitor_rect.width(), monitor_rect.height());
        let (preview_w, preview_h) = (
            parent_w / scale + padding * 2,
            parent_h / scale + padding * 2
        );
        let (rect_w, rect_h) = (
            preview_w + padding * 2,
            (preview_h + padding) * workspaces.len() as i32 + padding
        );

        let rect = RoundedRect::new(
            rect_w,
            rect_h,
            border_radius,
            Some(&color_background),
            None
        );
        rect.actor().set_position(
            (monitor_rect.x() + margin) as f32,
            (monitor_rect.y() + (parent_h - rect_h) / 2) as f32
        );
        parent.add_child(rect.actor());

        let mut previews = Vec::with_capacity(workspaces.len());
        for i in 0..workspaces.len() as i32 {
            let preview = RoundedRect::new(
                preview_w,
                preview_h,
                border_radius,
                Some(&color_input),
                if i == active_workspace {
                    Some(&color_border)
                } else {
                    None
                }
            );
            preview.actor().set_position(
                padding as f32,
                ((preview_h + padding) * i + padding) as f32
            );
            rect.actor().add_child(preview.actor());
            previews.push(preview);
        }

        for window in windows.iter().rev() {
            let window_rect = window.frame_rect();

            if ! window_rect.overlap(&monitor_rect) {
                continue;
            }

            let window_actor = match window.compositor_private() {
                Some(some) => match some.downcast::<WindowActor>() {
                    Ok(ok) => ok,
                    Err(_) => {
                        error!("Window compositor_private is not a WindowActor");
                        continue;
                    },
                },
                None => {
                    error!("Window missing compositor_private");
                    continue;
                }
            };

            for (i, workspace) in workspaces.iter().enumerate() {
                if window.located_on_workspace(workspace) {
                    let mini = Clone::new(&window_actor);
                    mini.set_position(
                        ((window_rect.x() - monitor_rect.x()) / scale + padding) as f32,
                        ((window_rect.y() - monitor_rect.y()) / scale + padding) as f32
                    );
                    mini.set_size(
                        (window_rect.width() / scale) as f32,
                        (window_rect.height() / scale) as f32
                    );
                    previews[i].actor().add_child(&mini);
                }
            }
        }

        Self {
            rect,
            previews,
        }
    }
}

pub struct WsPreviews {
    pub monitors: Vec<WsPreviewMonitor>,
}

impl WsPreviews {
    pub fn new(parent: &Actor, plugin: &Plugin, display: &Display) -> Rc<Self> {
        let workspace_manager = display.workspace_manager().expect("WsPreviews could not find workspace manager");
        let active_workspace = workspace_manager.active_workspace_index();

        //TODO: this allocates a vec of workspaces!
        let workspaces = workspace_manager.workspaces();

        //TODO: this allocates a vec of windows!
        let windows = display.tab_list(TabList::NormalAll, None);

        let n_monitors = display.n_monitors();
        let mut monitors = Vec::with_capacity(n_monitors as usize);
        for monitor in 0..n_monitors {
            monitors.push(WsPreviewMonitor::new(
                parent,
                display.monitor_geometry(monitor),
                active_workspace,
                &workspaces,
                &windows
            ));
        }
        Rc::new(Self {
            monitors,
        })
    }
}
