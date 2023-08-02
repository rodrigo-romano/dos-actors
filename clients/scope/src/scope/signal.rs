use std::{
    any::type_name,
    sync::{Arc, RwLock},
};

use eframe::egui::plot::{Line, PlotPoints};
use gmt_dos_clients::interface::{Data, UniqueIdentifier};
use tracing::info;

#[derive(Debug, PartialEq, Clone)]
pub(super) enum SignalState {
    // receiving data
    Run,
    // waiting for data or all data receiver
    Idle,
}
pub(super) struct Signal<U: UniqueIdentifier> {
    rx: Option<flume::Receiver<Data<U>>>,
    data: Arc<RwLock<Vec<[f64; 2]>>>,
    sampling_period: f64,
    state: Arc<RwLock<SignalState>>,
}
impl<U: UniqueIdentifier> Signal<U> {
    pub fn new(sampling_period: f64, rx: Option<flume::Receiver<Data<U>>>) -> Self {
        Self {
            rx,
            data: Arc::new(RwLock::new(vec![[0f64; 2]])),
            sampling_period,
            state: Arc::new(RwLock::new(SignalState::Idle)),
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
    fn run(&mut self);
    fn points(&self) -> PlotPoints;
    fn line(&self) -> Line;
    fn state(&self) -> SignalState;
}
impl<U> SignalProcessing for Signal<U>
where
    <U as UniqueIdentifier>::DataType: Send + Sync + for<'a> serde::Deserialize<'a>,
    f64: From<<U as UniqueIdentifier>::DataType>,
    <U as UniqueIdentifier>::DataType: Copy,
    U: UniqueIdentifier + 'static,
{
    fn run(&mut self) {
        let rx = self.rx.take().unwrap();
        let values = self.data.clone();
        let tau = self.sampling_period;
        *self.state.write().unwrap() = SignalState::Run;
        let state = self.state.clone();
        info!("signal ({}) run", self.name());
        tokio::spawn(async move {
            while let Some(data) = rx.recv().ok() {
                let value: f64 = (**&data).into();
                let mut v = values.write().unwrap();
                let [x, _y] = *v.last().unwrap();
                v.append(&mut vec![[x, value], [x + tau, value]]);
            }
            info!("signal stream ended");
            drop(rx);
            *state.write().unwrap() = SignalState::Idle;
        });
    }
    fn points(&self) -> PlotPoints {
        PlotPoints::from_iter(self.data.read().unwrap().clone())
    }
    fn line(&self) -> Line {
        eframe::egui::plot::Line::new(self.points()).name(self.name())
    }
    fn state(&self) -> SignalState {
        self.state.read().unwrap().clone()
    }
}
