/*!
# gateway interface for actors [SubSystem]s

The module implements the [Gateway] interface allowing to seamlessly insert a [Model]
as a [SubSystem] within a [Model].
The interface between the main [Model] and the sub-[Model] is managed by [Gateway]s
that routes inputs and outputs between the [Model] and the sub-[Model].

There are 2 types of gateway: [Gateway]`<_,`[Ins]`>` for inputs and [Gateway]`<_,`[Outs]`>` for outputs.

[Model]: crate::model::Model
[SubSystem]: super::SubSystem
*/

use std::{marker::PhantomData, sync::Arc};

use interface::{Data, Read, UniqueIdentifier, Update, Write};

/// Inputs gateway
pub type WayIn<M> = Gateway<M, Ins>;
/// Outputs gateway
pub type WayOut<M> = Gateway<M, Outs>;

/// Inputs gateway type
pub enum Ins {}
/// Outputs gateway type
pub enum Outs {}

/// [Ins] and [Outs] marker for [Gateway]
pub trait GatewayIO {}
impl GatewayIO for Ins {}
impl GatewayIO for Outs {}

/// Gateways specifications
///
/// Set the number of inputs to the sub-model, the number of outputs from the sub-model
/// and the gateways inputs and outputs datatype
pub trait Gateways {
    const N_IN: usize = 1;
    const N_OUT: usize = 1;
    type DataType: Default;
}

/// Gateway client
pub struct Gateway<M: Gateways, K> {
    data: Vec<Arc<<M as Gateways>::DataType>>,
    kind: PhantomData<K>,
}

unsafe impl<M: Gateways, K: GatewayIO> Send for Gateway<M, K> {}
unsafe impl<M: Gateways, K: GatewayIO> Sync for Gateway<M, K> {}

impl<M: Gateways, K: GatewayIO> Gateway<M, K> {
    pub fn read(
        &mut self,
        i: usize,
        data: Data<impl UniqueIdentifier<DataType = <M as Gateways>::DataType>>,
    ) {
        self.data[i] = data.into_arc();
    }
    pub fn write<U>(&self, i: usize) -> Option<Data<U>>
    where
        U: UniqueIdentifier<DataType = <M as Gateways>::DataType>,
    {
        self.data.get(i).map(|data| data.into())
    }
}

impl<M: Gateways> Gateway<M, Ins> {
    pub fn new() -> Self {
        Self {
            data: vec![Default::default(); <M as Gateways>::N_IN],
            kind: PhantomData,
        }
    }
}

impl<M: Gateways> Gateway<M, Outs> {
    pub fn new() -> Self {
        Self {
            data: vec![Default::default(); <M as Gateways>::N_OUT],
            kind: PhantomData,
        }
    }
}

impl<M: Gateways, K: GatewayIO> Update for Gateway<M, K> {}

/// Gateway input marker
///
/// Set the input index for data in [Gateway]`<_,`[Ins]`>`
pub trait In: UniqueIdentifier {
    const IDX: usize = 0;
}
/// Gateway output marker
///
/// Set the output index for data in [Gateway]`<_,`[Outs]`>`
pub trait Out: UniqueIdentifier {
    const IDX: usize = 0;
}

impl<U, M> Read<U> for Gateway<M, Ins>
where
    U: In<DataType = <M as Gateways>::DataType>,
    M: Gateways,
{
    fn read(&mut self, data: Data<U>) {
        Gateway::read(self, <U as In>::IDX, data);
    }
}

impl<U, M> Write<U> for Gateway<M, Ins>
where
    U: In<DataType = <M as Gateways>::DataType>,
    M: Gateways,
{
    fn write(&mut self) -> Option<Data<U>> {
        Gateway::write(self, 0)
    }
}

impl<U, M> Read<U> for Gateway<M, Outs>
where
    U: Out<DataType = <M as Gateways>::DataType>,
    M: Gateways,
{
    fn read(&mut self, data: Data<U>) {
        Gateway::read(self, <U as Out>::IDX, data);
    }
}

impl<U, M> Write<U> for Gateway<M, Outs>
where
    U: Out<DataType = <M as Gateways>::DataType>,
    M: Gateways,
{
    fn write(&mut self) -> Option<Data<U>> {
        Gateway::write(self, <U as Out>::IDX)
    }
}
