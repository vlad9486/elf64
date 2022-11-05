use super::{Error, Encoding};

#[derive(Clone)]
pub struct StringTable<'a> {
    slice: &'a [u8],
}

impl<'a> StringTable<'a> {
    pub fn new(slice: &'a [u8]) -> Self {
        StringTable { slice }
    }

    pub fn pick(&self, index: usize) -> Result<&'a [u8], Error> {
        const MAX_LENGTH: usize = 0xff;
        let mut length = 0;
        loop {
            if index + length > self.slice.len() {
                return Err(Error::SliceTooShort);
            }
            if self.slice[index + length] == 0 || length == MAX_LENGTH {
                length += 1;
                break;
            } else {
                length += 1;
            }
        }

        Ok(&self.slice[index..(index + length)])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NoteEntry<'a> {
    pub ty: u64,
    pub name: &'a [u8],
    pub description: &'a [u8],
}

#[derive(Clone)]
pub struct NoteTable<'a> {
    slice: &'a [u8],
    encoding: Encoding,
}

impl<'a> NoteTable<'a> {
    pub fn new(slice: &'a [u8], encoding: Encoding) -> Self {
        NoteTable { slice, encoding }
    }

    pub fn next(&self, position: &mut usize) -> Result<NoteEntry<'a>, Error> {
        if self.slice.len() < *position + 0x18 {
            return Err(Error::SliceTooShort);
        };

        let align8 = |x: usize| if x % 8 == 0 { x } else { x + 8 - x % 8 };

        let header = &self.slice[*position..];
        let name_size = read_int!(&header[0x00..], &self.encoding, u64) as usize;
        let description_size = read_int!(&header[0x08..], &self.encoding, u64) as usize;
        let ty = read_int!(&header[0x10..], &self.encoding, u64);

        let name_size_aligned = align8(name_size);
        let description_size = align8(description_size);

        let new_position = *position + 0x18 + name_size_aligned + description_size;
        if self.slice.len() < new_position {
            return Err(Error::SliceTooShort);
        };

        let str_start = *position + 0x18;
        let str_end = str_start + name_size;

        let entry = NoteEntry {
            ty,
            name: &self.slice[str_start..str_end],
            description: &self.slice[str_end..new_position],
        };

        *position = new_position;

        Ok(entry)
    }
}
