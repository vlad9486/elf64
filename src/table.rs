use core::marker::PhantomData;

use super::Encoding;

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
        let end = index + E::SIZE;
        E::new(&self.slice[index..end], self.encoding.clone())
    }
}
