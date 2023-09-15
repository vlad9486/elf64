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

    pub fn pick(&self, index: usize) -> Result<E, E::Error> {
        if self.slice.len() < index * E::SIZE {
            return Err(Error::SliceTooShort);
        }

        E::new(&self.slice[(index * E::SIZE)..], self.encoding.clone())
    }
}
