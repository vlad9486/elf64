use core::marker::PhantomData;
use super::{Encoding, Error};

pub trait Entry
where
    Self: Sized,
{
    type Error;

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
    E: Entry<Error = Error>,
{
    pub fn new(slice: &'a [u8], encoding: Encoding) -> Self {
        Table {
            slice,
            encoding,
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
            return Err(Error::SliceTooShort);
        };

        E::new(&self.slice[start..end], self.encoding.clone())
    }
}
