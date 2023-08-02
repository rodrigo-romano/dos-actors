use std::{
    any::type_name,
    sync::{Arc, RwLock},
};

use eframe::egui::{
    self,
    plot::{Line, PlotPoints},
};
use gmt_dos_clients::interface::{Data, UniqueIdentifier};
use gmt_dos_clients_transceiver::{CompactRecvr, Monitor, Transceiver, TransceiverError};
use tracing::info;

#[derive(Debug, thiserror::Error)]
pub enum ScopeError {
    #[error("failed to build transceiver")]
    Transceiver(#[from] TransceiverError),
}
pub type Result<T> = std::result::Result<T, ScopeError>;

struct Signal<U: UniqueIdentifier> {
    rx: Option<flume::Receiver<Data<U>>>,
    data: Arc<RwLock<Vec<[f64; 2]>>>,
}
impl<U: UniqueIdentifier> Signal<U> {
    pub fn new(rx: Option<flume::Receiver<Data<U>>>) -> Self {
        Self {
            rx,
            data: Arc::new(RwLock::new(vec![[0f64; 2]])),
        }
    }
    pub fn name(&self) -> String {
        let long_name = type_name::<U>();
        long_name
            .rsplit_once("::")
            .map_or_else(|| long_name.to_string(), |(_, name)| name.to_string())
    }
}

trait SignalProcessing {
    fn run(&mut self);
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
    fn run(&mut self) {
        let rx = self.rx.take().unwrap();
        let values = self.data.clone();
        info!("signal run");
        tokio::spawn(async move {
            while let Some(data) = rx.recv().ok() {
                let value: f64 = (**&data).into();
                let mut v = values.write().unwrap();
                let [x, _y] = *v.last().unwrap();
                v.append(&mut vec![[x, value], [x + 1f64, value]]);
            }
        });
    }
    fn points(&self) -> PlotPoints {
        PlotPoints::from_iter(self.data.read().unwrap().clone())
    }
    fn line(&self) -> Line {
        egui::plot::Line::new(self.points()).name(self.name())
    }
}
/// Data scope viewer
pub struct Scope {
    server_address: String,
    client_address: String,
    monitor: Monitor,
    signals: Vec<Box<dyn SignalProcessing>>,
    min_recvr: Option<CompactRecvr>,
}
impl Scope {
    /// Creates a new scope
    ///
    /// A scope is build from both the transmitter and the scope receiver internet socket addresses
    pub fn new<S: Into<String>>(server_address: S, client_address: S) -> Self {
        Self {
            monitor: Monitor::new(),
            server_address: server_address.into(),
            client_address: client_address.into(),
            signals: Vec::new(),
            min_recvr: None,
        }
    }
    /// Adds a signal to the scope
    pub fn signal<U>(mut self) -> Result<Self>
    where
        <U as UniqueIdentifier>::DataType: Send + Sync + for<'a> serde::Deserialize<'a>,
        f64: From<<U as UniqueIdentifier>::DataType>,
        <U as UniqueIdentifier>::DataType: Copy,
        U: UniqueIdentifier + 'static,
    {
        let rx = if let Some(min_recvr) = self.min_recvr.as_ref() {
            min_recvr.into()
        } else {
            let recvr = Transceiver::<U>::receiver(&self.server_address, &self.client_address)?;
            self.min_recvr = Some(CompactRecvr::from(&recvr));
            recvr
        }
        .run(&mut self.monitor)
        .take_channel_receiver();
        self.signals.push(Box::new(Signal::new(rx)));
        Ok(self)
    }
    /// Runs the scope
    pub fn run(mut self) -> Self {
        info!("scope run");
        self.signals.iter_mut().for_each(|signal| {
            let _ = signal.run();
        });
        self
    }
}

impl eframe::App for Scope {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let plot = egui::plot::Plot::new("Scope").legend(Default::default());
            plot.show(ui, |plot_ui| {
                for signal in &mut self.signals {
                    plot_ui.line(signal.line());
                }
            });
        });
        ctx.request_repaint();
    }
}
