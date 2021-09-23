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
    TabList,
    WindowActor,
};
use std::{
    rc::Rc,
};

use crate::{
    RoundedRect,
    Theme,
};

pub struct WsPreviews {
    pub rect: RoundedRect,
    previews: Vec<RoundedRect>,
}

impl WsPreviews {
    pub fn new(parent: &Actor, plugin: &Plugin, display: &Display) -> Rc<Self> {
        let workspace_manager = display.workspace_manager().expect("WsPreviews could not find workspace manager");
        let ws_active = workspace_manager.active_workspace_index();

        //TODO: this allocates a vec of workspaces!
        let workspaces = workspace_manager.workspaces();

        let color_background = Theme::color_background();
        let color_border = Theme::color_border();
        let color_input = Theme::color_input();

        let scale = 8.0;
        let (parent_w, parent_h) = parent.size();
        let (preview_w, preview_h) = (
            (parent_w / scale) as i32,
            (parent_h / scale) as i32
        );
        let (rect_w, rect_h) = (
            preview_w + 16,
            (preview_h + 8) * workspaces.len() as i32 + 8
        );

        let rect = RoundedRect::new(
            rect_w,
            rect_h,
            5.0,
            Some(&color_background),
            None
        );
        rect.actor().set_position(
            16.0,
            (parent_h - rect_h as f32) / 2.0
        );
        parent.add_child(rect.actor());

        let mut previews = Vec::with_capacity(workspaces.len());
        for i in 0..workspaces.len() as i32 {
            let preview = RoundedRect::new(
                preview_w,
                preview_h,
                5.0,
                Some(&color_input),
                if i == ws_active {
                    Some(&color_border)
                } else {
                    None
                }
            );
            preview.actor().set_position(
                8.0,
                (i * (preview_h + 8)) as f32 + 8.0
            );
            rect.actor().add_child(preview.actor());
            previews.push(preview);
        }

        let mut ret = Rc::new(Self {
            rect,
            previews,
        });

        //TODO: this allocates a vec of windows!
        for window in display.tab_list(TabList::NormalAll, None) {
            println!("  Window {}", window.id());

            let actor = match window.compositor_private() {
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
                    let mini = Clone::new(&actor);
                    mini.set_position(
                        actor.x() / scale,
                        actor.y() / scale
                    );
                    mini.set_size(
                        actor.width() / scale,
                        actor.height() / scale
                    );
                    ret.previews[i].actor().add_child(&mini);
                }
            }
        }

        ret
    }
}
