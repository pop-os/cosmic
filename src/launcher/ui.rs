use clutter::{
    Actor,
    ActorExt,
    Canvas,
    CanvasExt,
    Color,
    ContentExt,
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
use glib::Cast;
use log::{
    error,
    info,
};
use meta::{
    Display,
    Plugin,
};
use pop_launcher::{
    Request,
    Response,
    SearchResult,
};
use std::{
    cell::Cell,
    f64,
    rc::Rc,
};

use crate::wrapper::with_cosmic;

pub struct Theme;

impl Theme {
    pub fn font_name() -> &'static str {
        "Fira Sans Semi-Light 10"
    }

    pub fn small_font_name() -> &'static str {
        "Fira Sans Semi-Light 9"
    }

    pub fn color_background() -> Color {
        Color::new(0x33, 0x30, 0x2F, 0xFF)
    }

    pub fn color_border() -> Color {
        Color::new(0xFB, 0xB8, 0x6C, 0xFF)
    }

    pub fn color_highlight() -> Color {
        Color::new(0x5A, 0x57, 0x57, 0xFF)
    }

    pub fn color_input() -> Color {
        Color::new(0x2B, 0x29, 0x28, 0xFF)
    }

    pub fn color_text() -> Color {
        Color::new(0xE6, 0xE6, 0xE6, 0xFF)
    }
}

pub struct RoundedRect {
    actor: Actor,
    canvas: Canvas,
    fill_color: Rc<Cell<u32>>,
    stroke_color: Rc<Cell<u32>>,
}

impl RoundedRect {
    pub fn new(w: i32, h: i32, radius: f64, fill_color: Option<&Color>, stroke_color: Option<&Color>) -> Self {
        //TODO: find out why this requires so much sugar
        let canvas = Canvas::new().unwrap().dynamic_cast::<Canvas>().unwrap();
        canvas.set_size(w, h);

        let actor = Actor::new();
        //actor.set_size(w as f32, h as f32);
        actor.set_content(Some(&canvas));
        actor.set_content_scaling_filters(clutter::ScalingFilter::Trilinear, clutter::ScalingFilter::Linear);
        actor.set_request_mode(clutter::RequestMode::ContentSize);

        let fill_color = Rc::new(Cell::new(fill_color.map_or(0, |x| x.to_pixel())));
        let stroke_color = Rc::new(Cell::new(stroke_color.map_or(0, |x| x.to_pixel())));

        {
            let actor = actor.clone();
            let fill_color = fill_color.clone();
            let stroke_color = stroke_color.clone();
            canvas.connect_draw(move |canvas, cairo, surface_width, surface_height| {
                let x = 1.0;
                let y = 1.0;
                let w = surface_width as f64 - 2.0;
                let h = surface_height as f64 - 2.0;
                let degrees = f64::consts::PI / 180.0;

                cairo.save();
                cairo.set_operator(cairo::Operator::Clear);
                cairo.paint();
                cairo.restore();

                for (color, fill) in &[
                    (fill_color.get(), true),
                    (stroke_color.get(), false),
                ] {
                    cairo.new_sub_path();
                    cairo.arc(x + w - radius, y + radius, radius, -90.0 * degrees, 0.0 * degrees);
                    cairo.arc(x + w - radius, y + h - radius, radius, 0.0 * degrees, 90.0 * degrees);
                    cairo.arc(x + radius, y + h - radius, radius, 90.0 * degrees, 180.0 * degrees);
                    cairo.arc(x + radius, y + radius, radius, 180.0 * degrees, 270.0 * degrees);
                    cairo.close_path();

                    cairo.set_source_rgba(
                        ((color >> 24) & 0xFF) as f64 / 255.0,
                        ((color >> 16) & 0xFF) as f64 / 255.0,
                        ((color >> 8) & 0xFF) as f64 / 255.0,
                        ((color >> 0) & 0xFF) as f64 / 255.0
                    );
                    if *fill {
                        cairo.fill();
                    } else {
                        cairo.stroke();
                    }
                }

                true
            });
        }

        canvas.invalidate();

        Self {
            actor,
            canvas,
            fill_color,
            stroke_color,
        }
    }

    pub fn actor(&self) -> &Actor {
        &self.actor
    }

    pub fn set_fill_color(&self, color: Option<&Color>) {
        self.fill_color.set(color.map_or(0, |x| x.to_pixel()));
        self.canvas.invalidate();
    }

    pub fn set_stroke_color(&self, color: Option<&Color>) {
        self.fill_color.set(color.map_or(0, |x| x.to_pixel()));
        self.canvas.invalidate();
    }
}

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
    name: Text,
    description: Text,
    active: Cell<bool>,
}

impl LauncherItem {
    pub fn new(parent: &Actor, i: usize) -> Self {
        let rect = RoundedRect::new(480 - 16, 48 - 4, 5.0, None, None);
        rect.actor().set_position(8.0, 2.0 + i as f32 * 48.0);
        parent.add_child(rect.actor());

        let name = Text::new_full(Theme::font_name(), "", &Theme::color_text());
        ActorExt::set_position(&name, 8.0, 6.0);
        name.set_size(480.0 - 32.0, -1.0);
        rect.actor().add_child(&name);

        let description = Text::new_full(Theme::small_font_name(), "", &Theme::color_text());
        ActorExt::set_position(&description, 8.0, 22.0);
        description.set_ellipsize(pango::EllipsizeMode::End);
        description.set_size(480.0 - 32.0, -1.0);
        rect.actor().add_child(&description);

        let active = Cell::new(false);

        Self {
            rect,
            name,
            description,
            active,
        }
    }

    pub fn clear(&self) {
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
    pub fn new(parent: &Actor, plugin: &Plugin, display: &Display) -> Rc<Self> {
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
                with_cosmic(&plugin, |cosmic| {
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
                                match app_info.launch::<AppLaunchContext>(&[], None) {
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
                        with_cosmic(&plugin, |cosmic| {
                            cosmic.toggle_launcher(&plugin, &display);
                        });
                        true
                    },
                    Key::KEY_TAB => {
                        with_cosmic(&plugin, |cosmic| {
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
                        });
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
                with_cosmic(&plugin, |cosmic| {
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
