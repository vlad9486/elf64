use super::{Error, Encoding};

#[derive(Clone)]
pub struct StringTable<'a> {
    slice: &'a [u8],
}

impl<'a> StringTable<'a> {
    pub fn new(slice: &'a [u8]) -> Self {
        StringTable { slice: slice }
    }

    pub fn pick(&self, index: usize) -> Result<&'a str, Error> {
        use core::str::from_utf8;

        const MAX_LENGTH: usize = 0xff;
        let mut length = 0;
        loop {
            if index + length >= self.slice.len() {
                return Err(Error::NotEnoughData);
            }
            if self.slice[index + length] == 0 || length == MAX_LENGTH {
                break;
            } else {
                length += 1;
            }
        }

        from_utf8(&self.slice[index..(index + length)]).map_err(Error::Utf8Error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NoteEntry<'a> {
    pub type_: u64,
    pub name: &'a str,
    pub description: &'a [u8],
}

#[derive(Clone)]
pub struct NoteTable<'a> {
    slice: &'a [u8],
    encoding: Encoding,
}

impl<'a> NoteTable<'a> {
    pub fn new(slice: &'a [u8], encoding: Encoding) -> Self {
        NoteTable {
            slice: slice,
            encoding: encoding,
        }
    }

    pub fn next(&self, position: &mut usize) -> Result<NoteEntry<'a>, Error> {
        use core::str::from_utf8;
        use byteorder::{ByteOrder, LittleEndian, BigEndian};

        if self.slice.len() < position.clone() + 0x18 {
            return Err(Error::NotEnoughData);
        };

        let align8 = |x: usize| if x % 8 == 0 { x } else { x + 8 - x % 8 };

        let (name_size, description_size, type_) = match self.encoding {
            Encoding::Little => (
                LittleEndian::read_u64(&self.slice[0x00..0x08]) as usize,
                LittleEndian::read_u64(&self.slice[0x08..0x10]) as usize,
                LittleEndian::read_u64(&self.slice[0x10..0x18]),
            ),
            Encoding::Big => (
                BigEndian::read_u64(&self.slice[0x00..0x08]) as usize,
                BigEndian::read_u64(&self.slice[0x08..0x10]) as usize,
                BigEndian::read_u64(&self.slice[0x10..0x18]),
            ),
        };

        let name_size_aligned = align8(name_size);
        let description_size = align8(description_size);

        let new_position = position.clone() + 0x18 + name_size_aligned + description_size;
        if self.slice.len() < new_position {
            return Err(Error::NotEnoughData);
        };

        let str_start = position.clone() + 0x18;
        let str_end = str_start + name_size;

        let entry = NoteEntry {
            type_: type_,
            name: from_utf8(&self.slice[str_start..str_end]).map_err(Error::Utf8Error)?,
            description: &self.slice[str_end..new_position],
        };

        *position = new_position;

        Ok(entry)
    }
}
