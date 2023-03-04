use std::marker::PhantomData;

pub struct Model<State> {
    data: Vec<f64>,
    state: PhantomData<State>,
}

pub enum Unknown {}
pub enum Ready {}
pub enum Running {}
pub enum Completed {}

impl Model<Unknown> {
    fn new(data: Vec<f64>) -> Self {
        Self {
            data,
            state: PhantomData,
        }
    }
    fn check(self, data: Vec<f64>) -> Model<Ready> {
        // some check
        Model::<Ready> {
            data: self.data,
            state: PhantomData,
        }
    }
}

impl Model<Ready> {
    fn start(self) -> Model<Running> {
        // starting stuff
        Model::<Running> {
            data: self.data,
            state: PhantomData,
        }
    }
}

impl<T> Model<T> {
    fn affect_all(&self) {
        todo!()
    }
}

trait UnknowOrReady {}
impl UnknowOrReady for Unknown {}
impl UnknowOrReady for Ready {}

impl<T: UnknownOrReady> Model<T> {
    fn do_something(self) {
        todo!()
    }
}
