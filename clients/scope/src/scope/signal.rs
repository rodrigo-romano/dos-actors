use std::{
    any::type_name,
    sync::{Arc, RwLock},
};

use eframe::egui::{
    self,
    plot::{Line, PlotPoints},
};
use gmt_dos_clients::interface::{Data, UniqueIdentifier};
use tracing::warn;

pub(super) struct Signal<U: UniqueIdentifier> {
    rx: Option<flume::Receiver<Data<U>>>,
    data: Arc<RwLock<Vec<[f64; 2]>>>,
    sampling_period: f64,
}
impl<U: UniqueIdentifier> Signal<U> {
    pub fn new(sampling_period: f64, rx: Option<flume::Receiver<Data<U>>>) -> Self {
        Self {
            rx,
            data: Arc::new(RwLock::new(vec![[0f64; 2]])),
            sampling_period,
        }
    }
    pub fn name(&self) -> String {
        let long_name = type_name::<U>();
        long_name
            .rsplit_once("::")
            .map_or_else(|| long_name.to_string(), |(_, name)| name.to_string())
    }
}

pub(super) trait SignalProcessing {
    fn run(&mut self, ctx: egui::Context);
    fn points(&self) -> PlotPoints;
    fn line(&self) -> Line;
}
impl<U> SignalProcessing for Signal<U>
where
    <U as UniqueIdentifier>::DataType: Send + Sync + for<'a> serde::Deserialize<'a>,
    f64: From<<U as UniqueIdentifier>::DataType>,
    <U as UniqueIdentifier>::DataType: Copy,
    U: UniqueIdentifier + 'static,
{
    fn run(&mut self, ctx: egui::Context) {
        let rx = self.rx.take().unwrap();
        let values = self.data.clone();
        let tau = self.sampling_period;
        let name = self.name();
        tokio::spawn(async move {
            while let Some(data) = rx.recv().ok() {
                // debug!("received {name}");
                let value: f64 = (**&data).into();
                let mut v = values.write().unwrap();
                if v.len() == 1 {
                    warn!("{name}: streaming");
                }
                let [x, _y] = *v.last().unwrap();
                v.append(&mut vec![[x, value], [x + tau, value]]);
                ctx.request_repaint();
            }
            warn!("{name}: stream ended");
            drop(rx);
        });
    }
    fn points(&self) -> PlotPoints {
        PlotPoints::from_iter(self.data.read().unwrap().clone())
    }
    fn line(&self) -> Line {
        eframe::egui::plot::Line::new(self.points()).name(self.name())
    }
}
