use eframe::{
    egui::Context,
    epaint::{Color32, ColorImage, TextureHandle},
};

use crate::payload::Payload;

#[non_exhaustive]
pub(crate) enum SignalData {
    Signal {
        tag: String,
        tau: f64,
        points: Vec<[f64; 2]>,
    },
    Image {
        tag: String,
        size: [usize; 2],
        texture: Option<TextureHandle>,
    },
}

impl From<&Payload> for SignalData {
    fn from(payload: &Payload) -> Self {
        match payload {
            Payload::Signal { tag, tau, .. } => Self::Signal {
                tag: tag.clone(),
                tau: *tau,
                points: vec![[0f64; 2]],
            },
            Payload::Image { tag, size, .. } => Self::Image {
                tag: tag.clone(),
                size: *size,
                texture: None,
            },
        }
    }
}

impl SignalData {
    pub fn add_payload(&mut self, ctx: &mut Context, payload: &Payload) {
        match (payload, self) {
            (Payload::Signal { value, .. }, SignalData::Signal { tau, points, .. }) => {
                let &[x, _y] = points.last().unwrap();
                points.push([x, *value]);
                points.push([x + *tau, *value]);
            }
            (Payload::Image { size, pixels, .. }, SignalData::Image { tag, texture, .. }) => {
                // let (min, max) = (dbg!(payload.min()), dbg!(payload.max()));
                let (min, max) = (-1., 1.);
                let range = max - min;
                let colormap = colorous::CUBEHELIX;
                let mut img = ColorImage::new(*size, Color32::TRANSPARENT);
                pixels
                    .iter()
                    .map(|v| (v - min) / range)
                    .map(|t| colormap.eval_continuous(t))
                    .zip(img.pixels.iter_mut())
                    .for_each(|(rgb, px)| {
                        let colorous::Color { r, g, b } = rgb;
                        *px = Color32::from_rgb(r, g, b);
                    });
                texture.replace(ctx.load_texture(tag.as_str(), img, Default::default()));
            }
            _ => todo!(),
        };
    }
}

unsafe impl Send for SignalData {}
unsafe impl Sync for SignalData {}
