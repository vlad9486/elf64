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
        use byteorder::{ByteOrder, LittleEndian, BigEndian};

        if slice.len() < Self::SIZE {
            return Err(Error::NotEnoughData);
        };

        match encoding {
            Encoding::Little => {
                let temp = LittleEndian::read_u64(&slice[0x08..0x10]);
                Ok(RelEntry {
                    address: LittleEndian::read_u64(&slice[0x00..0x08]),
                    symbol_index: (temp / 0x100000000) as u32,
                    relocation_type: (temp & 0xffffffff) as u32,
                })
            },
            Encoding::Big => {
                let temp = BigEndian::read_u64(&slice[0x08..0x10]);
                Ok(RelEntry {
                    address: BigEndian::read_u64(&slice[0x00..0x08]),
                    symbol_index: (temp / 0x100000000) as u32,
                    relocation_type: (temp & 0xffffffff) as u32,
                })
            },
        }
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
        use byteorder::{ByteOrder, LittleEndian, BigEndian};

        if slice.len() < Self::SIZE {
            return Err(Error::NotEnoughData);
        };

        match encoding {
            Encoding::Little => {
                let temp = LittleEndian::read_u64(&slice[0x08..0x10]);
                Ok(RelaEntry {
                    address: LittleEndian::read_u64(&slice[0x00..0x08]),
                    symbol_index: (temp / 0x100000000) as u32,
                    relocation_type: (temp & 0xffffffff) as u32,
                    addend: LittleEndian::read_i64(&slice[0x10..0x18]),
                })
            },
            Encoding::Big => {
                let temp = BigEndian::read_u64(&slice[0x08..0x10]);
                Ok(RelaEntry {
                    address: BigEndian::read_u64(&slice[0x00..0x08]),
                    symbol_index: (temp / 0x100000000) as u32,
                    relocation_type: (temp & 0xffffffff) as u32,
                    addend: BigEndian::read_i64(&slice[0x10..0x18]),
                })
            },
        }
    }
}
