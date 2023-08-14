use std::marker::PhantomData;

use eframe::egui;
use gmt_dos_clients::interface::UniqueIdentifier;
use gmt_dos_clients_transceiver::{CompactRecvr, Monitor, Transceiver, TransceiverError};
use tokio::task::JoinError;
use tracing::debug;

mod signal;
use signal::{Signal, SignalProcessing};

use crate::{GmtScope, ImageScope, PlotScope, ScopeKind};

#[derive(Debug, thiserror::Error)]
pub enum ScopeError {
    #[error("failed to build transceiver")]
    Transceiver(#[from] TransceiverError),
    #[error("some task didn't terminate successfully")]
    Join(#[from] JoinError),
}
pub type Result<T> = std::result::Result<T, ScopeError>;

/// Data scope viewer
pub struct XScope<K = PlotScope>
where
    K: ScopeKind,
{
    server_ip: String,
    client_address: String,
    monitor: Option<Monitor>,
    signals: Vec<Box<dyn SignalProcessing>>,
    min_recvr: Option<CompactRecvr>,
    kind: PhantomData<K>,
}
impl<K: ScopeKind> XScope<K> {
    /// Creates a new scope
    ///
    /// A scope is build from both the transmitter and the scope receiver internet socket addresses
    pub fn new<S: Into<String>>(server_ip: S, client_address: S) -> Self {
        Self {
            monitor: Some(Monitor::new()),
            server_ip: server_ip.into(),
            client_address: client_address.into(),
            signals: Vec::new(),
            min_recvr: None,
            kind: PhantomData,
        }
    }
    /// Adds a signal to the scope
    pub fn signal<U>(mut self, port: u32) -> Result<Self>
    where
        U: UniqueIdentifier + 'static,
    {
        let server_address = format!("{}:{}", self.server_ip, port);
        let rx = if let Some(min_recvr) = self.min_recvr.as_ref() {
            min_recvr.spawn(server_address)?
        } else {
            let recvr = Transceiver::<crate::payload::ScopeData<U>>::receiver(
                server_address,
                &self.client_address,
            )?;
            self.min_recvr = Some(CompactRecvr::from(&recvr));
            recvr
        }
        .run(self.monitor.as_mut().unwrap())
        .take_channel_receiver();
        self.signals.push(Box::new(Signal::new(rx)));
        Ok(self)
    }
    /// Initiates data acquisition
    pub fn run(mut self, ctx: egui::Context) -> Self {
        debug!("scope run");
        self.signals.iter_mut().for_each(|signal| {
            let _ = signal.run(ctx.clone());
        });
        // self.monitor.take().unwrap().await?;
        self
    }
    /// Takes ownership of [Monitor]
    pub fn take_monitor(&mut self) -> Monitor {
        self.monitor.take().unwrap()
    }
}

impl<K> XScope<K>
where
    XScope<K>: eframe::App,
    K: ScopeKind + 'static,
{
    /// Display the scope
    pub fn show(mut self) {
        let monitor = self.monitor.take().unwrap();
        tokio::spawn(async move {
            match monitor.join().await {
                Ok(_) => println!("*** data streaming complete ***"),
                Err(e) => println!("!!! data streaming error with {:?} !!!", e),
            }
        });
        let native_options = eframe::NativeOptions::default();
        let _ = eframe::run_native(
            "GMT DOS Actors Scope",
            native_options,
            Box::new(|cc| Box::new(self.run(cc.egui_ctx.clone()))),
        );
    }
}

/// A scope for plotting signals
pub type Scope = XScope<PlotScope>;

impl eframe::App for Scope {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let plot = egui::plot::Plot::new("Scope").legend(Default::default());
            plot.show(ui, |plot_ui: &mut egui::plot::PlotUi| {
                for signal in &mut self.signals {
                    // plot_ui.line(signal.line());
                    signal.plot_ui(plot_ui)
                }
            });
        });
    }
}

/// A scope for displaying images
pub type Shot = XScope<ImageScope>;

impl eframe::App for Shot {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let plot = egui::plot::Plot::new("Scope")
                .show_axes([false; 2])
                .show_x(false)
                .show_y(false)
                .data_aspect(1f32);
            // .view_aspect(1f32);
            plot.show(ui, |plot_ui: &mut egui::plot::PlotUi| {
                for signal in &mut self.signals {
                    // plot_ui.line(signal.line());
                    signal.plot_ui(plot_ui)
                }
            });
        });
    }
}

/// A scope for displaying images
pub type GmtShot = XScope<GmtScope>;

impl eframe::App for GmtShot {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let plot = egui::plot::Plot::new("Scope")
                // .show_axes([false; 2])
                // .show_axes([false; 2])
                .show_x(false)
                .show_y(false)
                .data_aspect(1f32);
            // .view_aspect(1f32);
            plot.show(ui, |plot_ui: &mut egui::plot::PlotUi| {
                for signal in &mut self.signals {
                    // plot_ui.line(signal.line());
                    signal.plot_ui(plot_ui)
                }
            });
        });
    }
}
