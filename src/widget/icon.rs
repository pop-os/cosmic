use clutter::{
    Actor,
    ActorExt,
    Image,
    ImageExt,
    ScalingFilter,
};
use glib::{
    Cast,
};
use log::error;

pub struct Icon {
    actor: Actor,
    image: Image,
    size: i32,
}

impl Icon {
    pub fn new(size: i32) -> Self {
        //TODO: find out why this requires so much sugar
        let image = Image::new().unwrap().dynamic_cast::<Image>().unwrap();

        let actor = Actor::new();
        actor.set_content(Some(&image));
        actor.set_content_scaling_filters(ScalingFilter::Trilinear, ScalingFilter::Linear);
        actor.set_size(size as f32, size as f32);

        Self {
            actor,
            image,
            size,
        }
    }

    pub fn actor(&self) -> &Actor {
        &self.actor
    }

    pub fn clear(&self) -> Result<(), ()> {
        // This must have at least one pixel, so we provide a transparent one
        match self.image.set_bytes(
            &glib::Bytes::from_static(&[0, 0, 0, 0]),
            cogl::PixelFormat::RGBA_8888,
            1,
            1,
            1
        ) {
            Ok(()) => Ok(()),
            Err(err) => {
                eprintln!("failed to clear icon: {}", err);
                Err(())
            }
        }
    }

    //TODO: cache icons by name?
    pub fn load(&self, name: &str) -> Result<(), ()> {
        use gtk::prelude::IconThemeExt;

        let theme = match gtk::IconTheme::default() {
            Some(some) => some,
            None => {
                error!("failed to get default icon theme");
                return Err(());
            }
        };

        let info = match theme.lookup_icon(name, self.size, gtk::IconLookupFlags::empty()) {
            Some(some) => some,
            None => {
                error!("failed to lookup icon {} with size {}", name, self.size);
                return Err(());
            }
        };

        //TODO: what should this be?
        let fg = gdk::RGBA {
            red: 0.9,
            green: 0.9,
            blue: 0.9,
            alpha: 1.0,
        };
        let pixbuf = match info.load_symbolic(&fg, None, None, None) {
            Ok(ok) => ok.0,
            Err(err) => {
                error!("failed to load icon {} with size {}: {}", name, self.size, err);
                return Err(());
            }
        };

        let bytes = match pixbuf.pixel_bytes() {
            Some(some) => some,
            None => {
                error!("failed to load icon {} with size {}: no bytes found", name, self.size);
                return Err(());
            }
        };

        match self.image.set_bytes(
            &bytes,
            if pixbuf.has_alpha() {
                cogl::PixelFormat::RGBA_8888
            } else {
                cogl::PixelFormat::RGB_888
            },
            pixbuf.width() as u32,
            pixbuf.height() as u32,
            pixbuf.rowstride() as u32
        ) {
            Ok(()) => Ok(()),
            Err(err) => {
                error!("failed to convert icon {} with size {}: {}", name, self.size, err);
                Err(())
            }
        }
    }
}
