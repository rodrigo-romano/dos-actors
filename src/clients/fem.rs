use super::Client;
use fem::dos::{DiscreteModalSolver, Solver};

impl<S: Default + std::fmt::Debug + Solver> Client for DiscreteModalSolver<S>
where
    DiscreteModalSolver<S>: Iterator,
{
    type I = Vec<f64>;
    type O = Vec<f64>;
    fn consume(&mut self, data: Vec<&Self::I>) -> &mut Self {
        log::debug!(
            "receive #{} inputs: {:?}",
            data.len(),
            data.iter().map(|x| x.len()).collect::<Vec<usize>>()
        );
        self.u = data.into_iter().cloned().flatten().collect();
        self
    }
    fn produce(&mut self) -> Option<Vec<Self::O>> {
        log::debug!("produce");
        let mut data: Vec<Self::O> = Vec::with_capacity(self.y.len());
        let mut m = 0;
        for n in &self.y_sizes {
            data.push(self.y.iter().skip(m).take(*n).cloned().collect());
            m += n;
        }
        Some(data)
    }
    fn update(&mut self) -> &mut Self {
        log::debug!("update");
        self.next();
        self
    }
}
