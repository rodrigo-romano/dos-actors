use std::marker::PhantomData;

use eframe::{
    egui,
    egui::{Margin, Vec2},
};
use gmt_dos_clients_transceiver::{CompactRecvr, Monitor, Transceiver, TransceiverError};
use interface::UniqueIdentifier;
use tokio::task::JoinError;
use tracing::debug;

mod signal;
use signal::{Signal, SignalProcessing};

use crate::{GmtScope, ImageScope, PlotScope, ScopeKind};

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("failed to build transceiver")]
    Transceiver(#[from] TransceiverError),
    #[error("some task didn't terminate successfully")]
    Join(#[from] JoinError),
}
pub type Result<T> = std::result::Result<T, ClientError>;

/// Data scope client
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
    /// A scope is build from both the server IP and the client internet socket addresses
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
    pub fn run(&mut self, ctx: egui::Context) {
        debug!("scope run");
        self.signals.iter_mut().for_each(|signal| {
            let _ = signal.run(ctx.clone());
        });
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
        let native_options = eframe::NativeOptions {
            initial_window_size: Some(Vec2::from(<K as ScopeKind>::window_size())),
            ..Default::default()
        };
        let _ = eframe::run_native(
            "GMT DOS Actors Scope",
            native_options,
            Box::new(|cc| {
                Box::new({
                    self.run(cc.egui_ctx.clone());
                    self
                })
            }),
        );
    }
}

/// Signal plotting scope
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

struct NodeScope {
    indices: (usize, usize),
    scope: Scope,
}

const PLOT_SIZE: (f32, f32) = (600f32, 500f32);
const MAX_WINDOW_SIZE: (f32, f32) = (1200f32, 1000f32);

/// Display [Scope]s in a grid like pattern
pub struct GridScope {
    size: (usize, usize),
    scopes: Vec<NodeScope>,
    plot_size: (f32, f32),
}
impl GridScope {
    /// Creates a new grid layout for [Scope]s
    ///
    /// `size` sets the number of rows and columns
    pub fn new(size: (usize, usize)) -> Self {
        let (rows, cols) = size;
        let width = MAX_WINDOW_SIZE.0.min(PLOT_SIZE.0 * cols as f32) / cols as f32;
        let height = MAX_WINDOW_SIZE.1.min(PLOT_SIZE.1 * rows as f32) / rows as f32;
        Self {
            size,
            scopes: vec![],
            plot_size: (width, height),
        }
    }
    fn window_size(&self) -> (f32, f32) {
        let (rows, cols) = self.size;
        let (width, height) = self.plot_size;
        (width * cols as f32, height * rows as f32)
    }
    /// Sets a [Scope] at position `(row,column)` in the grid layout
    pub fn pin(mut self, indices: (usize, usize), scope: Scope) -> Self {
        let (rows, cols) = self.size;
        let (row, col) = indices;
        assert!(
            row < rows,
            "The row index in the scopes grid must be less than {}",
            rows
        );
        assert!(
            col < cols,
            "The columm index in the scopes grid must be less than {}",
            cols
        );
        self.scopes.push(NodeScope { indices, scope });
        self
    }
    /// Display the scope
    pub fn show(mut self) {
        for node in self.scopes.iter_mut() {
            let monitor = node.scope.monitor.take().unwrap();
            tokio::spawn(async move {
                match monitor.join().await {
                    Ok(_) => println!("*** data streaming complete ***"),
                    Err(e) => println!("!!! data streaming error with {:?} !!!", e),
                }
            });
        }
        let native_options = eframe::NativeOptions {
            initial_window_size: Some(Vec2::from(self.window_size())),
            ..Default::default()
        };
        let _ = eframe::run_native(
            "GMT DOS Actors Scope",
            native_options,
            Box::new(|cc| {
                for node in self.scopes.iter_mut() {
                    let scope = &mut node.scope;
                    scope.run(cc.egui_ctx.clone());
                }
                Box::new(self)
            }),
        );
    }
}

impl eframe::App for GridScope {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let (rows, cols) = self.size;
            let style = ui.style_mut();
            style.spacing.item_spacing = egui::vec2(0.0, 0.0);
            for row in 0..rows {
                ui.horizontal(|ui| {
                    for col in 0..cols {
                        self.scopes
                            .iter_mut()
                            .find(|node| node.indices == (row, col))
                            .map(|node| {
                                let plot = egui::plot::Plot::new("Scope")
                                    .legend(Default::default())
                                    .width(self.plot_size.0)
                                    .height(self.plot_size.1)
                                    .set_margin_fraction(Vec2::from((0.05, 0.05)));
                                plot.show(ui, |plot_ui: &mut egui::plot::PlotUi| {
                                    for signal in &mut node.scope.signals {
                                        signal.plot_ui(plot_ui)
                                    }
                                });
                            });
                    }
                });
            }
        });
    }
}

/// Image display scope
pub type Shot = XScope<ImageScope>;

impl eframe::App for Shot {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let plot = egui::plot::Plot::new("Scope")
                //.show_axes([false; 2])
                .show_x(false)
                .show_y(false)
                .allow_scroll(false)
                .data_aspect(1f32);
            plot.show(ui, |plot_ui: &mut egui::plot::PlotUi| {
                for signal in &mut self.signals {
                    // plot_ui.line(signal.line());
                    signal.plot_ui(plot_ui)
                }
            });
        });
    }
}

/// GMT scope
///
/// Image display scope which data is masked by the GMT exit pupil mask
pub type GmtShot = XScope<GmtScope>;

impl eframe::App for GmtShot {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        for signal in &mut self.signals {
            // plot_ui.line(signal.line());
            signal.plot_stats_ui(ctx)
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            let plot = egui::plot::Plot::new("Scope")
                // .show_axes([false; 2])
                .show_x(false)
                .show_y(false)
                // .allow_drag(false)
                .allow_scroll(false)
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
