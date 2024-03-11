use apache_arrow::{
    array::{ListArray, PrimitiveArray},
    datatypes::ArrowPrimitiveType,
};

use crate::{Arrow, ArrowError, BufferDataType, Result};

pub trait Get<T>
where
    T: BufferDataType,
    <T as BufferDataType>::ArrayType: ArrowPrimitiveType,
    Vec<T>: FromIterator<<<T as BufferDataType>::ArrayType as ArrowPrimitiveType>::Native>,
{
    /// Return the record field entry
    fn get<S>(&mut self, field_name: S) -> Result<Vec<Vec<T>>>
    where
        S: AsRef<str>,
        String: From<S>;

    /// Return the record field entry skipping the first `skip` elements and taking all (None) or some (Some(`take`)) elements
    fn get_skip_take<S>(
        &mut self,
        field_name: S,
        skip: usize,
        take: Option<usize>,
    ) -> Result<Vec<Vec<T>>>
    where
        S: AsRef<str>,
        String: From<S>;
    /// Return the record field entry skipping the first `skip` elements
    fn get_skip<S>(&mut self, field_name: S, skip: usize) -> Result<Vec<Vec<T>>>
    where
        S: AsRef<str>,
        String: From<S>,
    {
        self.get_skip_take(field_name, skip, None)
    }
    /// Return the record field entry taking `take` elements
    fn get_take<S>(&mut self, field_name: S, take: usize) -> Result<Vec<Vec<T>>>
    where
        S: AsRef<str>,
        String: From<S>,
    {
        self.get_skip_take(field_name, 0, Some(take))
    }
}
impl<'a, T> Get<T> for Arrow
where
    T: BufferDataType,
    <T as BufferDataType>::ArrayType: ArrowPrimitiveType,
    Vec<T>: FromIterator<<<T as BufferDataType>::ArrayType as ArrowPrimitiveType>::Native>,
{
    /// Return the record field entry
    fn get<S>(&mut self, field_name: S) -> Result<Vec<Vec<T>>>
    where
        S: AsRef<str>,
        String: From<S>,
    {
        match self.record() {
            Ok(record) => match record.schema().column_with_name(field_name.as_ref()) {
                Some((idx, _)) => record
                    .column(dbg!(idx))
                    .as_any()
                    .downcast_ref::<ListArray>()
                    .map(|data| {
                        data.iter()
                            .map(|data| {
                                data.map(|data| {
                                    data.as_any()
                                        .downcast_ref::<PrimitiveArray<<T as BufferDataType>::ArrayType>>()
                                        .and_then(|data| data.iter().collect::<Option<Vec<T>>>())
                                })
                                .flatten()
                            })
                            .collect::<Option<Vec<Vec<T>>>>()
                    })
                    .flatten()
                    .ok_or_else(|| ArrowError::ParseField(field_name.into())),
                None => Err(ArrowError::FieldNotFound(field_name.into())),
            },
            Err(e) => Err(e),
        }
    }
    /// Return the record field entry skipping the first `skip` elements and taking all (None) or some (Some(`take`)) elements
    fn get_skip_take<S>(
        &mut self,
        field_name: S,
        skip: usize,
        take: Option<usize>,
    ) -> Result<Vec<Vec<T>>>
    where
        S: AsRef<str>,
        String: From<S>,
    {
        match self.record() {
            Ok(record) => match record.schema().column_with_name(field_name.as_ref()) {
                Some((idx, _)) => record
                    .column(idx)
                    .as_any()
                    .downcast_ref::<ListArray>()
                    .map(|data| {
                        data.iter()
                            .skip(skip)
                            .take(take.unwrap_or(usize::MAX))
                            .map(|data| {
                                data.map(|data| {
                                    data.as_any()
                                        .downcast_ref::<PrimitiveArray<<T as BufferDataType>::ArrayType>>()
                                        .and_then(|data| data.iter().collect::<Option<Vec<T>>>())
                                })
                                .flatten()
                            })
                            .collect::<Option<Vec<Vec<T>>>>()
                    })
                    .flatten()
                    .ok_or_else(|| ArrowError::ParseField(field_name.into())),
                None => Err(ArrowError::FieldNotFound(field_name.into())),
            },
            Err(e) => Err(e),
        }
    }
}
