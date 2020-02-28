use core::marker::PhantomData;

use super::{ErrorSliceLength, Encoding};

pub trait Entry
where
    Self: Sized,
{
    type Error: ErrorSliceLength;

    const SIZE: usize;

    fn new(slice: &[u8], encoding: Encoding) -> Result<Self, Self::Error>;
}

#[derive(Clone)]
pub struct Table<'a, E>
where
    E: Entry,
{
    slice: &'a [u8],
    encoding: Encoding,
    phantom_data: PhantomData<E>,
}

impl<'a, E> Table<'a, E>
where
    E: Entry,
{
    pub fn new(slice: &'a [u8], encoding: Encoding) -> Self {
        Table {
            slice: slice,
            encoding: encoding,
            phantom_data: PhantomData,
        }
    }

    pub fn length(&self) -> usize {
        self.slice.len() / E::SIZE
    }

    pub fn pick(&self, index: usize) -> Result<E, E::Error> {
        let start = index * E::SIZE;
        let end = start + E::SIZE;

        if self.slice.len() < end {
            return Err(E::Error::slice_too_short());
        };

        E::new(&self.slice[start..end], self.encoding.clone())
    }
}
