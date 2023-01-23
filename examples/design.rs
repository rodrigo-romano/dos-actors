use std::{
    any::{type_name, Any},
    fmt,
    marker::PhantomData,
    sync::Arc,
};

#[derive(Debug)]
pub struct DataType<T, U, P, C>(
    T,
    PhantomData<*const U>,
    PhantomData<*const P>,
    PhantomData<*const C>,
);
impl<T, U, P, C> Data<T, U, P, C> {
    pub fn new(data: T) -> Self {
        Self(data, PhantomData, PhantomData, PhantomData)
    }
}

pub trait DataObject: fmt::Debug {
    //fn produce() -> Arc<Self>;
    fn consumer(&self, _client: &mut dyn Client) {}
    fn producer(&mut self, _client: &dyn Client) {}
}

//impl In1 for DataId1 {}
pub trait Trading<T, U, P, C> {
    fn consume(&mut self, _data: &Data<T, U, P, C>) {}
    fn produce(&self) -> Option<Data<T, U, P, C>> {
        None
    }
}

#[derive(Debug)]
pub enum DataId1 {}
#[derive(Debug)]
pub enum DataId2 {}
#[derive(Debug)]
pub enum DataId3 {}

#[derive(Debug)]
pub struct Consumer {
    data: Vec<f64>,
}
#[derive(Debug)]
pub struct Producer;
#[derive(Debug)]
pub struct Ground;

impl Trading<Vec<f64>, DataId1, Producer, Consumer> for Consumer {
    fn consume(&mut self, data: &Data<Vec<f64>, DataId1, Producer, Consumer>) {
        let n = self.data.len() / 2;
        let (left, _) = self.data.split_at_mut(n);
        left.copy_from_slice(&data.0[..n]);
    }
}
impl Trading<Vec<f64>, DataId2, Producer, Consumer> for Consumer {
    fn consume(&mut self, data: &Data<Vec<f64>, DataId2, Producer, Consumer>) {
        let n = self.data.len() / 2;
        let (_, right) = self.data.split_at_mut(n);
        right.copy_from_slice(&data.0[0..right.len()]);
    }
}
impl Trading<Vec<f64>, DataId1, Producer, Consumer> for Producer {
    fn produce(&self) -> Option<Data<Vec<f64>, DataId1, Producer, Consumer>> {
        Some(Data(
            (0..10)
                .map(|i| (-1f64).powi(i) * (10f64 - i as f64))
                .collect::<Vec<f64>>(),
            PhantomData,
            PhantomData,
            PhantomData,
        ))
    }
}
impl Trading<Vec<f64>, DataId2, Producer, Consumer> for Producer {
    fn produce(&self) -> Option<Data<Vec<f64>, DataId2, Producer, Consumer>> {
        Some(Data(
            (0..10)
                .map(|i| (-1f64).powi(i) * (i as f64 + 1f64))
                .collect::<Vec<f64>>(),
            PhantomData,
            PhantomData,
            PhantomData,
        ))
    }
}
impl Trading<Vec<f64>, DataId3, Producer, Consumer> for Producer {
    fn produce(&self) -> Option<Data<Vec<f64>, DataId3, Producer, Consumer>> {
        Some(Data(
            (0..10)
                .map(|i| (-1f64).powi(i) * (i as f64 + 1f64))
                .collect::<Vec<f64>>(),
            PhantomData,
            PhantomData,
            PhantomData,
        ))
    }
}

pub trait Client: fmt::Debug {
    fn as_any(&self) -> &dyn Any;
    fn as_mut_any(&mut self) -> &mut dyn Any;
}
impl Client for Consumer {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}
impl Client for Ground {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}
impl Client for Producer {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl<T, I, P, C> DataObject for Data<T, I, P, C>
where
    T: fmt::Debug,
    I: fmt::Debug,
    P: 'static + fmt::Debug + Trading<T, I, P, C> + Client,
    C: 'static + fmt::Debug + Trading<T, I, P, C> + Client,
{
    fn consumer(&self, client: &mut dyn Client) {
        let client: &mut C = client.as_mut_any().downcast_mut::<C>().expect(&format!(
            "failed to downcast Client to {:?}",
            type_name::<C>()
        ));
        Trading::<T, I, P, C>::consume(client, self);
        /*
                let (left, _) = client.data.split_at_mut(self.0.len());
                left.copy_from_slice(&self.0);
        */
    }
    fn producer(&mut self, client: &dyn Client) {
        let client: &P = client.as_any().downcast_ref::<P>().expect(&format!(
            "failed to downcast Client to {:?}",
            type_name::<P>()
        ));
        if let Some(data) = Trading::<T, I, P, C>::produce(client) {
            *self = data;
        }
    }
}
/*
impl DataObject for Data<Vec<f64>, DataId1, Consumer, Producer> {
    fn producer(&mut self, client: &dyn Client) {
        let client: &Consumer = client
            .as_any()
            .downcast_ref::<Consumer>()
            .expect("downcasting to ... failed");
        if let Some(data) = Trading::<Vec<f64>, DataId1, Consumer, Producer>::produce(client) {
            *self = data;
        }
    }
}
 */
/*
impl DataObject for Data<Vec<f64>, DataId2, Consumer, Consumer> {
    fn consumer(&self, client: &mut dyn Client) {
        let client: &mut Consumer = client
            .as_mut_any()
            .downcast_mut::<Consumer>()
            .expect("downcasting to ... failed");
        let n = client.data.len();
        let (_, right) = client.data.split_at_mut(n - self.0.len());
        right.copy_from_slice(&self.0);
    }
}
 */

fn main() {
    let mut container: Vec<Arc<dyn DataObject>> = vec![];

    let q: Option<Arc<dyn DataObject>> =
        Some(Arc::new(
            Data::<Vec<f64>, DataId1, Producer, Consumer>::new(vec![]),
        ));

    /*
        let source = Producer;

        {
            let mut data: Option<Arc<Data<Vec<f64>, DataId1, Producer, Consumer>>> = None;
            data.producer(&source);
            container.push(data.take().unwrap());
            dbg!(&data);
        }
        {
            let mut data: Option<Arc<Data<Vec<f64>, DataId2, Producer, Consumer>>> = None;
            data.producer(&source);
            container.push(data.take().unwrap());
            dbg!(&data);
        }
        let mut client1 = Consumer { data: vec![0.; 5] };
        dbg!(&client1);
        container.get(0).map(|x| x.consumer(&mut client1));
        container.get(1).map(|x| x.consumer(&mut client1));
        dbg!(&client1);
    */
}
