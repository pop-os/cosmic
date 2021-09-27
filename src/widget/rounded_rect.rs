use clutter::{
    Actor,
    ActorExt,
    Canvas,
    CanvasExt,
    Color,
    ContentExt,
    RequestMode,
    ScalingFilter,
};
use glib::{
    Cast,
};
use std::{
    cell::Cell,
    f64,
    rc::Rc,
};

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
        actor.set_content_scaling_filters(ScalingFilter::Trilinear, ScalingFilter::Linear);
        actor.set_request_mode(RequestMode::ContentSize);

        let fill_color = Rc::new(Cell::new(fill_color.map_or(0, |x| x.to_pixel())));
        let stroke_color = Rc::new(Cell::new(stroke_color.map_or(0, |x| x.to_pixel())));

        {
            let fill_color = fill_color.clone();
            let stroke_color = stroke_color.clone();
            canvas.connect_draw(move |canvas, cairo, surface_width, surface_height| {
                let x = 1.0;
                let y = 1.0;
                let w = surface_width as f64 - 2.0;
                let h = surface_height as f64 - 2.0;
                let degrees = f64::consts::PI / 180.0;

                //TODO: why do these return results?
                let _ = cairo.save();
                let _ = cairo.set_operator(cairo::Operator::Clear);
                let _ = cairo.paint();
                let _ = cairo.restore();

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
                        let _ = cairo.fill();
                    } else {
                        let _ = cairo.stroke();
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
        self.stroke_color.set(color.map_or(0, |x| x.to_pixel()));
        self.canvas.invalidate();
    }
}
