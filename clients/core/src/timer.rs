use super::Progress;
use interface::{Data, Tick, Update, Write};

/// Simple digital timer
#[derive(Default, Debug)]
pub struct Timer<T = indicatif::ProgressBar> {
    tick: usize,
    progress_bar: Option<T>,
    name: String,
}
impl<T: Progress> Timer<T> {
    /// Initializes the timer based on the duration in # of samples
    pub fn new(duration: usize) -> Self {
        Self {
            tick: 1 + duration,
            progress_bar: None,
            name: String::from("Timer"),
        }
    }
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = name.into();
        self
    }
    pub fn progress(&mut self) {
        self.progress_bar = Some(<T as Progress>::progress(&self.name, self.tick));
    }
    /*     pub fn progress(&mut self) {
        let progress = ProgressBar::new(self.tick as u64);
        progress.set_style(
            ProgressStyle::with_template("{msg} [{eta_precise}] {bar:50.cyan/blue} {percent:>3}%")
                .unwrap(),
        );
        progress.set_message(self.name.clone().unwrap_or("Timer".into()));
        // let bar: Bar = progress.bar(self.tick, "Timer:");
        self.progress_bar = Some(progress);
    } */
    /*     pub fn progress_with(&mut self, progress: Arc<Mutex<Progress>>) {
        let bar: Bar = progress.lock().unwrap().bar(self.tick, "Timer:");
        self.progress_bar = Some(ProgressBar { progress, bar });
    } */
}
impl<T> Update for Timer<T>
where
    T: Progress + Send + Sync,
{
    fn update(&mut self) {
        if let Some(pb) = self.progress_bar.as_mut() {
            pb.increment()
        };
        self.tick -= 1;
    }
}

impl<T> Write<Tick> for Timer<T>
where
    T: Progress + Send + Sync,
{
    fn write(&mut self) -> Option<Data<Tick>> {
        if self.tick > 0 {
            Some(Data::new(()))
        } else {
            if let Some(pb) = self.progress_bar.as_mut() {
                pb.finish()
            };
            None
        }
    }
}
