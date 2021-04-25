use super::{Address, Error, Encoding, Entry};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelEntry {
    address: Address,
    symbol_index: u32,
    relocation_type: u32,
}

impl Entry for RelEntry {
    type Error = Error;

    const SIZE: usize = 0x10;

    fn new(slice: &[u8], encoding: Encoding) -> Result<Self, Self::Error> {
        if slice.len() < Self::SIZE {
            return Err(Error::SliceTooShort);
        }

        let temp = read_int!(&slice[0x08..], &encoding, u64);
        Ok(RelEntry {
            address: read_int!(&slice[0x00..], &encoding, u64),
            symbol_index: (temp / 0x100000000) as u32,
            relocation_type: (temp & 0xffffffff) as u32,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelaEntry {
    pub address: Address,
    pub symbol_index: u32,
    pub relocation_type: u32,
    pub addend: i64,
}

impl Entry for RelaEntry {
    type Error = Error;

    const SIZE: usize = 0x18;

    fn new(slice: &[u8], encoding: Encoding) -> Result<Self, Self::Error> {
        if slice.len() < Self::SIZE {
            return Err(Error::SliceTooShort);
        }

        let temp = read_int!(&slice[0x08..], &encoding, u64);
        Ok(RelaEntry {
            address: read_int!(&slice[0x00..], &encoding, u64),
            symbol_index: (temp / 0x100000000) as u32,
            relocation_type: (temp & 0xffffffff) as u32,
            addend: read_int!(&slice[0x10..], &encoding, i64),
        })
    }
}
