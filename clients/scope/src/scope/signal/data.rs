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
        minmax: Option<(f64, f64)>,
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
            Payload::Image {
                tag,
                size,
                mut minmax,
                ..
            } => Self::Image {
                tag: tag.clone(),
                size: *size,
                texture: None,
                minmax: minmax.take(),
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
            (
                Payload::Image {
                    size, pixels, mask, ..
                },
                SignalData::Image {
                    tag,
                    texture,
                    minmax,
                    ..
                },
            ) => {
                let (min, max) = if let Some((min, max)) = minmax {
                    (*min, *max)
                } else {
                    (payload.min(), payload.max())
                };
                let range = max - min;
                let colormap = colorous::CIVIDIS;
                let mut img = ColorImage::new(*size, Color32::TRANSPARENT);
                match mask {
                    Some(mask) => mask
                        .iter()
                        .zip(img.pixels.iter_mut())
                        .filter(|(&m, _)| m)
                        .zip(pixels)
                        .map(|((_, u), v)| (u, (v - min) / range))
                        .map(|(u, t)| (u, colormap.eval_continuous(t)))
                        .for_each(|(px, rgb)| {
                            let colorous::Color { r, g, b } = rgb;
                            *px = Color32::from_rgb(r, g, b);
                        }),
                    None => pixels
                        .iter()
                        .map(|v| (v - min) / range)
                        .map(|t| colormap.eval_continuous(t))
                        .zip(img.pixels.iter_mut())
                        .for_each(|(rgb, px)| {
                            let colorous::Color { r, g, b } = rgb;
                            *px = Color32::from_rgb(r, g, b);
                        }),
                };
                texture.replace(ctx.load_texture(tag.as_str(), img, Default::default()));
            }
            _ => todo!(),
        };
    }
}

unsafe impl Send for SignalData {}
unsafe impl Sync for SignalData {}