use dos_actors::{io, Client, Initiator, Terminator};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Default, Debug)]
struct Sinusoide {
    pub sampling_frequency: f64,
    pub period: f64,
    pub n_step: usize,
    pub step: usize,
}
impl Client<f64, f64> for Sinusoide {
    fn produce(&mut self) -> Option<Vec<f64>> {
        if self.step < self.n_step {
            let value = (2.
                * std::f64::consts::PI
                * self.step as f64
                * (self.sampling_frequency * self.period).recip())
            .sin();
            self.step += 1;
            Some(vec![value])
        } else {
            None
        }
    }
}
#[derive(Default, Debug)]
struct DoNothing(Vec<f64>);
impl Client<f64, f64> for DoNothing {
    fn consume(&mut self, data: Vec<&f64>) -> &mut Self {
        self.0.extend(data.into_iter());
        self
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    const N: usize = 1;

    let (output, input) = io::channel();

    let client = Arc::new(Mutex::new(Sinusoide {
        sampling_frequency: 20f64,
        period: 1f64,
        n_step: 21,
        step: 0,
    }));
    let mut source = Initiator::<Sinusoide, f64, N>::build(client.clone());
    source.add_output(output);

    let do_nothing = Arc::new(Mutex::new(DoNothing::default()));
    let mut sink = Terminator::<DoNothing, f64, N>::build(do_nothing.clone());
    sink.add_input(input);

    tokio::spawn(async move {
        source.task().await;
    });
    match sink.task().await {
        Ok(_) => {}
        Err(e) => {
            println!("{}", e);
        }
    };
    dbg!(&do_nothing);
    Ok(())
}
