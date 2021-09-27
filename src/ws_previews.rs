use clutter::{
    Actor,
    ActorExt,
    AnimationMode,
    Clone,
};
use glib::{
    Cast,
    ObjectExt,
    SignalHandlerId,
};
use log::{
    error,
    info,
};
use meta::{
    BackgroundGroup,
    Display,
    Plugin,
    Rectangle,
    TabList,
    Window,
    WindowActor,
    Workspace,
    WorkspaceManager,
};
use std::{
    cell::RefCell,
    rc::Rc,
};

use crate::{
    RoundedRect,
    Theme,
};

pub struct WsPreviewMonitor {
    pub rect: RoundedRect,
    previews: Vec<RoundedRect>,
    active: RoundedRect,
}

impl WsPreviewMonitor {
    pub fn new(
        parent: &Actor,
        monitor_rect: Rectangle,
        active_workspace: i32,
        workspaces: &[Workspace],
        background_opt: Option<Actor>,
        windows: &[Window]
    ) -> Self {
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
                None
            );
            preview.actor().set_position(
                padding as f32,
                ((preview_h + padding) * i + padding) as f32
            );
            rect.actor().add_child(preview.actor());

            if let Some(background) = &background_opt {
                let mini = Clone::new(background);
                mini.set_position(
                    ((background.x() as i32 - monitor_rect.x()) / scale + padding) as f32,
                    ((background.y() as i32 - monitor_rect.y()) / scale + padding) as f32
                );
                mini.set_size(
                    (background.width() as i32 / scale) as f32,
                    (background.height() as i32 / scale) as f32
                );
                preview.actor().add_child(&mini);
            }

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

        let active = RoundedRect::new(
            preview_w,
            preview_h,
            border_radius,
            None,
            Some(&color_border)
        );
        rect.actor().add_child(active.actor());

        let this = Self {
            rect,
            previews,
            active,
        };

        this.update_workspace(active_workspace);
        this.active.actor().set_easing_duration(150);
        this.active.actor().set_easing_mode(AnimationMode::EaseOutQuad);

        this
    }

    pub fn update_workspace(&self, active_workspace: i32) {
        if let Some(preview) = self.previews.get(active_workspace as usize) {
            self.active.actor().set_position(
                preview.actor().x(),
                preview.actor().y()
            );
        }
    }
}

pub struct WsPreviews {
    pub monitors: Vec<WsPreviewMonitor>,
    workspace_manager: WorkspaceManager,
    workspace_switched_id: RefCell<Option<SignalHandlerId>>,
}

impl WsPreviews {
    pub fn new(parent: &Actor, plugin: &Plugin, display: &Display, background_group: &BackgroundGroup) -> Rc<Self> {
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
                background_group.child_at_index(monitor),
                &windows
            ));
        }

        let mut this = Rc::new(Self {
            monitors,
            workspace_manager,
            workspace_switched_id: RefCell::new(None),
        });

        let workspace_switched_id = {
            let color_border = Theme::color_border();
            let that = this.clone();
            this.workspace_manager.connect_workspace_switched(move |_, from, to, direction| {
                info!("from {} to {} dir {}", from, to, direction);
                for monitor in that.monitors.iter() {
                    monitor.update_workspace(to);
                }
            })
        };

        this.workspace_switched_id.replace(Some(workspace_switched_id));

        this
    }
}

impl Drop for WsPreviews {
    fn drop(&mut self) {
        if let Some(workspace_switched_id) = self.workspace_switched_id.replace(None) {
            self.workspace_manager.disconnect(workspace_switched_id);
        }
    }
}
