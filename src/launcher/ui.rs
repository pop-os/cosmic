use clutter::{
    Actor,
    ActorExt,
    Text,
    TextExt,
};
use evdev::{
    Key,
};
use gio::{
    AppLaunchContext,
    DesktopAppInfo,
    prelude::{
        AppInfoExt,
    },
};
use log::{
    error,
    info,
};
use meta::{
    Display,
};
use pop_launcher::{
    IconSource,
    Request,
    Response,
    SearchResult,
};
use std::{
    cell::Cell,
    rc::Rc,
};

use crate::{
    CosmicPlugin,
    Icon,
    RoundedRect,
    Theme,
};

pub struct LauncherEntry {
    actor: Text,
}

impl LauncherEntry {
    pub fn new(parent: &Actor) -> Self {
        let rect = RoundedRect::new(480 - 16, 32 - 4, 5.0, Some(&Theme::color_input()), Some(&Theme::color_border()));
        rect.actor().set_position(8.0, 8.0);
        parent.add_child(rect.actor());

        let actor = Text::new_full(Theme::font_name(), "", &Theme::color_text());
        ActorExt::set_position(&actor, 8.0, 6.0);
        actor.set_activatable(true);
        actor.set_cursor_visible(true);
        actor.set_editable(true);
        actor.set_reactive(true);
        actor.set_selectable(true);
        actor.set_selection_color(Some(&Theme::color_highlight()));
        actor.set_single_line_mode(true);
        rect.actor().add_child(&actor);

        Self {
            actor,
        }
    }
}

pub struct LauncherItem {
    rect: RoundedRect,
    category_icon: Icon,
    icon: Icon,
    name: Text,
    description: Text,
    active: Cell<bool>,
}

impl LauncherItem {
    pub fn new(parent: &Actor, i: usize) -> Self {
        let (w, h) = (480 - 16, 48 - 4);
        let rect = RoundedRect::new(w, h, 5.0, None, None);
        rect.actor().set_position(8.0, 2.0 + i as f32 * 48.0);
        parent.add_child(rect.actor());

        let mut x = 8.0;
        let mut y = 6.0;

        let category_icon_size = 16;
        let category_icon = Icon::new(category_icon_size);
        category_icon.actor().set_position(x, y + 8.0);
        rect.actor().add_child(category_icon.actor());
        x += category_icon_size as f32 + 8.0;

        let icon_size = 32;
        let icon = Icon::new(icon_size);
        icon.actor().set_position(x, y);
        rect.actor().add_child(icon.actor());
        x += icon_size as f32 + 8.0;

        let text_w = w as f32 - x - 8.0;
        let name = Text::new_full(Theme::font_name(), "", &Theme::color_text());
        ActorExt::set_position(&name, x, y);
        name.set_ellipsize(pango::EllipsizeMode::End);
        name.set_size(text_w, -1.0);
        rect.actor().add_child(&name);
        y += 16.0;

        let description = Text::new_full(Theme::small_font_name(), "", &Theme::color_text());
        ActorExt::set_position(&description, x, y);
        description.set_ellipsize(pango::EllipsizeMode::End);
        description.set_size(text_w, -1.0);
        rect.actor().add_child(&description);

        let active = Cell::new(false);

        Self {
            rect,
            category_icon,
            icon,
            name,
            description,
            active,
        }
    }

    pub fn clear(&self) {
        self.category_icon.clear();
        self.icon.clear();
        self.name.set_text(None);
        self.description.set_text(None);
        self.active.set(false);
    }

    pub fn select(&self, selected: bool) {
        if selected && self.active.get() {
            self.rect.set_fill_color(Some(&Theme::color_highlight()));
        } else {
            self.rect.set_fill_color(None);
        }
    }

    pub fn set(&self, result: &SearchResult) {
        if let Some(IconSource::Name(icon_name)) = &result.category_icon {
            self.category_icon.load(&icon_name);
        } else {
            self.category_icon.clear();
        }
        if let Some(IconSource::Name(icon_name)) = &result.icon {
            self.icon.load(&icon_name);
        } else {
            self.icon.clear();
        }
        self.name.set_text(Some(&result.name));
        self.description.set_text(Some(&result.description));
        self.active.set(true);
    }
}

pub struct LauncherUi {
    pub rect: RoundedRect,
    entry: LauncherEntry,
    items: Box<[LauncherItem]>,
    selected: Cell<usize>,
}

impl LauncherUi {
    pub fn new(parent: &Actor, plugin: &CosmicPlugin, display: &Display) -> Rc<Self> {
        let (w, h) = (480, 440);
        let (parent_w, parent_h) = parent.size();
        let x = (parent_w - w as f32) / 2.0;
        let y = (parent_h - h as f32) / 2.0;

        let rect = RoundedRect::new(w, h, 5.0, Some(&Theme::color_background()), None);
        rect.actor().set_position(x, y);
        parent.add_child(rect.actor());

        let entry = LauncherEntry::new(rect.actor());

        let items = {
            let mut items = Vec::new();
            while items.len() < 8 {
                let item = LauncherItem::new(
                    rect.actor(),
                    items.len() + 1
                );
                items.push(item);
            }
            items.into_boxed_slice()
        };

        let selected = Cell::new(0);

        let ret = Rc::new(Self {
            rect,
            entry,
            items,
            selected,
        });

        // Activate selected item and close launcher on enter
        {
            let plugin = plugin.clone();
            let display = display.clone();
            let this = ret.clone();
            ret.entry.actor.connect_activate(move |_entry_actor| {
                let cosmic = plugin.cosmic();

                let selected = this.selected.get();
                info!("activate {}", selected);
                let response_res = cosmic.launcher_request(
                    Request::Activate(selected as pop_launcher::Indice)
                );
                info!("response: {:#?}", response_res);
                if let Ok(Response::DesktopEntry { path, .. }) = response_res {
                    //TODO: gpu_preference
                    match DesktopAppInfo::from_filename(&path) {
                        Some(app_info) => {
                            //TODO: launch context?
                            let context: Option<&AppLaunchContext> = None;
                            match app_info.launch(&[], context) {
                                Ok(_) => (),
                                Err(err) => {
                                    error!("failed to launch entry {:?}: {}", path, err);
                                },
                            }
                        },
                        None => error!("failed to load entry {:?}", path),
                    }
                }

                // Close launcher on enter
                cosmic.toggle_launcher(&plugin, &display);
            });
        }

        // Detect special keys, like escape and arrows
        {
            let plugin = plugin.clone();
            let display = display.clone();
            let this = ret.clone();
            ret.entry.actor.connect_key_press_event(move |entry_actor, key_event| {
                match Key::new(key_event.evdev_code as u16) {
                    Key::KEY_ESC => {
                        // Close launcher on escape
                        plugin.cosmic().toggle_launcher(&plugin, &display);
                        true
                    },
                    Key::KEY_TAB => {
                        let cosmic = plugin.cosmic();

                        let selected = this.selected.get();
                        info!("complete {}", selected);
                        let response_res = cosmic.launcher_request(
                            Request::Complete(selected as pop_launcher::Indice)
                        );
                        info!("response: {:#?}", response_res);
                        if let Ok(Response::Fill(text)) = response_res {
                            this.selected.set(0);
                            // Automatically runs search again
                            entry_actor.set_text(Some(&text));
                        }
                        true
                    },
                    Key::KEY_UP => {
                        this.key_up();
                        true
                    },
                    Key::KEY_DOWN => {
                        this.key_down();
                        true
                    },
                    _ => false
                }
            });
        }

        // Update search results
        {
            let plugin = plugin.clone();
            let this = ret.clone();
            ret.entry.actor.connect_text_changed(move |entry_actor| {
                let cosmic = plugin.cosmic();

                this.clear();
                if let Some(text) = entry_actor.text() {
                    info!("search {}", text);
                    let response_res = cosmic.launcher_request(
                        Request::Search(text.to_string())
                    );
                    info!("response: {:#?}", response_res);
                    if let Ok(Response::Update(results)) = response_res {
                        this.set(&results);
                    }
                }
            });
        }

        //TODO: set clutter backend default input method so there are no errors
        ret.entry.actor.grab_key_focus();

        ret
    }

    pub fn clear(&self) {
        for item in self.items.iter() {
            item.clear();
        }
        self.select();
    }

    fn max_selected(&self) -> usize {
        self.items.len().checked_sub(1).unwrap_or(0)
    }

    pub fn key_up(&self) {
        let mut selected = self.selected.get();
        if selected > 0 {
            selected -= 1;
        } else {
            selected = self.max_selected();
        }
        self.selected.set(selected);
        self.select();
    }

    pub fn key_down(&self) {
        let mut selected = self.selected.get();
        if selected < self.max_selected() {
            selected += 1;
        } else {
            selected = 0;
        }
        self.selected.set(selected);
        self.select();
    }

    pub fn set(&self, results: &[SearchResult]) {
        for result in results.iter() {
            if let Some(item) = self.items.get(result.id as usize) {
                item.set(result);
            } else {
                error!("failed to find launcher item for {}", result.id);
            }
        }
        self.select();
    }

    pub fn select(&self) {
        let selected = self.selected.get();
        for (i, item) in self.items.iter().enumerate() {
            item.select(selected == i);
        }
    }
}
