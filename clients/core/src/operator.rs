use std::{
    marker::PhantomData,
    ops::{Add, Sub},
    sync::Arc,
};

use interface::{Data, Read, UniqueIdentifier, Update, Write};

#[derive(Default, Debug, Clone)]
pub enum OperatorKind {
    #[default]
    Add,
    Sub,
}
impl<S: AsRef<str>> From<S> for OperatorKind {
    fn from(value: S) -> Self {
        match value.as_ref() {
            "+" => Self::Add,
            "-" => Self::Sub,
            _ => unimplemented!(r#"operators are either "+" or "-""#),
        }
    }
}

#[derive(Default, Debug)]
pub struct Operator<T> {
    kind: OperatorKind,
    left: Arc<Vec<T>>,
    right: Arc<Vec<T>>,
    output: Arc<Vec<T>>,
}

impl<T: Default> Operator<T> {
    pub fn new<O: Into<OperatorKind>>(o: O) -> Self {
        Self {
            kind: o.into(),
            ..Default::default()
        }
    }
}

impl<T> Update for Operator<T>
where
    T: Copy + Add<Output = T> + Sub<Output = T>,
{
    fn update(&mut self) {
        self.output = Arc::new(
            self.left
                .iter()
                .zip(&*self.right)
                .map(|(left, right)| match self.kind {
                    OperatorKind::Add => *left + *right,
                    OperatorKind::Sub => *left - *right,
                })
                .collect::<Vec<T>>(),
        );
    }
}

pub struct Left<U: UniqueIdentifier>(PhantomData<U>);
impl<U> UniqueIdentifier for Left<U>
where
    U: UniqueIdentifier,
{
    type DataType = <U as UniqueIdentifier>::DataType;

    const PORT: u32 = <U as UniqueIdentifier>::PORT;
}

pub struct Right<U: UniqueIdentifier>(PhantomData<U>);
impl<U> UniqueIdentifier for Right<U>
where
    U: UniqueIdentifier,
{
    type DataType = <U as UniqueIdentifier>::DataType;

    const PORT: u32 = <U as UniqueIdentifier>::PORT;
}

impl<T, U> Read<Left<U>> for Operator<T>
where
    U: UniqueIdentifier<DataType = Vec<T>>,
{
    fn read(&mut self, data: Data<Left<U>>) {
        self.left = data.as_arc()
    }
}

impl<T, U> Read<Right<U>> for Operator<T>
where
    U: UniqueIdentifier<DataType = Vec<T>>,
{
    fn read(&mut self, data: Data<Right<U>>) {
        self.right = data.as_arc()
    }
}

impl<T, U> Write<U> for Operator<T>
where
    U: UniqueIdentifier<DataType = Vec<T>>,
{
    fn write(&mut self) -> Option<Data<U>> {
        Some(self.output.clone().into())
    }
}
