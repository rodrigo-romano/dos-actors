use std::{
    ops::Deref,
    sync::{Arc, RwLock},
};

use eframe::egui::{
    self,
    plot::{Line, PlotImage, PlotPoint, PlotUi},
};
use gmt_dos_clients::interface::{Data, UniqueIdentifier};
// use tracing::warn;

mod data;
use data::SignalData;

type D<U> = Data<crate::payload::ScopeData<U>>;
pub(super) struct Signal<U>
where
    U: UniqueIdentifier,
{
    rx: Option<flume::Receiver<D<U>>>,
    data: Arc<RwLock<Option<SignalData>>>,
}
impl<U> Signal<U>
where
    U: UniqueIdentifier,
{
    pub fn new(rx: Option<flume::Receiver<D<U>>>) -> Self {
        Self {
            rx,
            data: Arc::new(RwLock::new(None)),
        }
    }
}

pub(super) trait SignalProcessing {
    fn run(&mut self, ctx: egui::Context);
    fn plot_ui(&self, ui: &mut PlotUi);
    fn minmax(&self) -> Option<(f64, f64)>;
}

impl<U> SignalProcessing for Signal<U>
where
    U: UniqueIdentifier + 'static,
{
    fn run(&mut self, mut ctx: egui::Context) {
        let rx = self.rx.take().unwrap();
        let data = self.data.clone();
        tokio::spawn(async move {
            while let Some(wrap) = rx.recv().ok() {
                let payload = wrap.deref();
                data.write()
                    .unwrap()
                    .get_or_insert(SignalData::from(payload))
                    .add_payload(&mut ctx, payload);
                ctx.request_repaint();
            }
            // warn!("{name}: stream ended");
            drop(rx);
        });
    }
    fn plot_ui(&self, ui: &mut PlotUi) {
        if let Some(data) = self.data.read().unwrap().as_ref() {
            match data {
                SignalData::Signal { tag, points, .. } => {
                    let line = Line::new(points.clone()).name(tag);
                    ui.line(line);
                }
                SignalData::Image { size, texture, .. } => {
                    texture.as_ref().map(|texture| {
                        let image = PlotImage::new(
                            texture,
                            PlotPoint::new(0., 0.),
                            (2f32 * size[0] as f32 / size[1] as f32, 2f32),
                        );
                        ui.image(image);
                    });
                }
            }
        };
    }

    fn minmax(&self) -> Option<(f64, f64)> {
        if let Some(data) = self.data.read().unwrap().as_ref() {
            match data {
                SignalData::Signal { .. } => None,
                SignalData::Image { minmax, .. } => minmax.clone(),
            }
        } else {
            None
        }
    }
}
