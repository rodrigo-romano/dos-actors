use eframe::egui;
use gmt_dos_clients::interface::UniqueIdentifier;
use gmt_dos_clients_transceiver::{CompactRecvr, Monitor, Transceiver, TransceiverError};
use tokio::task::JoinError;
use tracing::info;

mod signal;
use signal::{Signal, SignalProcessing, SignalState};

#[derive(Debug, thiserror::Error)]
pub enum ScopeError {
    #[error("failed to build transceiver")]
    Transceiver(#[from] TransceiverError),
    #[error("some task didn't terminate successfully")]
    Join(#[from] JoinError),
}
pub type Result<T> = std::result::Result<T, ScopeError>;

/// Data scope viewer
pub struct Scope {
    server_address: String,
    client_address: String,
    monitor: Option<Monitor>,
    signals: Vec<Box<dyn SignalProcessing>>,
    min_recvr: Option<CompactRecvr>,
}
impl Scope {
    /// Creates a new scope
    ///
    /// A scope is build from both the transmitter and the scope receiver internet socket addresses
    pub fn new<S: Into<String>>(server_address: S, client_address: S) -> Self {
        Self {
            monitor: Some(Monitor::new()),
            server_address: server_address.into(),
            client_address: client_address.into(),
            signals: Vec::new(),
            min_recvr: None,
        }
    }
    /// Adds a signal to the scope
    pub fn signal<U>(mut self, sampling_period: f64) -> Result<Self>
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
        .run(self.monitor.as_mut().unwrap())
        .take_channel_receiver();
        self.signals
            .push(Box::new(Signal::new(sampling_period, rx)));
        Ok(self)
    }
    /// Initiates data acquisition
    pub fn run(mut self) -> Self {
        info!("scope run");
        self.signals.iter_mut().for_each(|signal| {
            let _ = signal.run();
        });
        // self.monitor.take().unwrap().await?;
        self
    }
    /// Display the scope
    pub fn show(self) {
        let native_options = eframe::NativeOptions::default();
        let _ = eframe::run_native(
            "GMT DOS Actors Scope",
            native_options,
            Box::new(|_cc| Box::new(self.run())),
        );
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
        if self
            .signals
            .iter()
            .any(|signal| signal.state() == SignalState::Run)
        {
            ctx.request_repaint();
        }
    }
}
