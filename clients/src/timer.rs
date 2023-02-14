// use super::ProgressBar;
use gmt_dos_actors_interface::{Data, UniqueIdentifier, Update, Write};
// use linya::{Bar, Progress};
use std::sync::Arc;

/// Simple digital timer
pub struct Timer {
    tick: usize,
    // progress_bar: Option<ProgressBar>,
}
impl Timer {
    /// Initializes the timer based on the duration in # of samples
    pub fn new(duration: usize) -> Self {
        Self {
            tick: 1 + duration,
            // progress_bar: None,
        }
    }
    /*     pub fn progress(self) -> Self {
        let mut progress = Progress::new();
        let bar: Bar = progress.bar(self.tick, "Timer:");
        Self {
            progress_bar: Some(ProgressBar {
                progress: Arc::new(Mutex::new(progress)),
                bar,
            }),
            ..self
        }
    }
    pub fn progress_with(self, progress: Arc<Mutex<Progress>>) -> Self {
        let bar: Bar = progress.lock().unwrap().bar(self.tick, "Timer:");
        Self {
            progress_bar: Some(ProgressBar { progress, bar }),
            ..self
        }
    } */
}
impl Update for Timer {
    fn update(&mut self) {
        /*         if let Some(pb) = self.progress_bar.as_mut() {
            pb.progress.lock().unwrap().inc_and_draw(&pb.bar, 1)
        } */
        self.tick -= 1;
    }
}
pub enum Tick {}
pub type Void = ();
impl UniqueIdentifier for Tick {
    type DataType = Void;
}
impl Write<Tick> for Timer {
    fn write(&mut self) -> Option<Arc<Data<Tick>>> {
        if self.tick > 0 {
            Some(Arc::new(Data::new(())))
        } else {
            None
        }
    }
}
