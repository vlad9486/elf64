use super::{Address, Error, Encoding, Index};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SymbolBinding {
    Local,
    Global,
    Weak,
    OsSpecific(u8),
    ProcessorSpecific(u8),
    Unknown(u8),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SymbolType {
    Nothing,
    Object,
    Function,
    Section,
    File,
    OsSpecific(u8),
    ProcessorSpecific(u8),
    Unknown(u8),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SymbolInfo {
    pub binding: SymbolBinding,
    pub type_: SymbolType,
}

impl From<u8> for SymbolInfo {
    fn from(v: u8) -> Self {
        SymbolInfo {
            binding: match (v & 0xf0) / 0x10 {
                0x00 => SymbolBinding::Local,
                0x01 => SymbolBinding::Global,
                0x02 => SymbolBinding::Weak,
                t @ 0x0a..=0x0c => SymbolBinding::OsSpecific(t - 0x0a),
                t @ 0x0d..=0x0f => SymbolBinding::ProcessorSpecific(t - 0x0d),
                t => SymbolBinding::Unknown(t),
            },
            type_: match v & 0x0f {
                0x00 => SymbolType::Nothing,
                0x01 => SymbolType::Object,
                0x02 => SymbolType::Function,
                0x03 => SymbolType::Section,
                0x04 => SymbolType::File,
                t @ 0x0a..=0x0c => SymbolType::OsSpecific(t - 0x0a),
                t @ 0x0d..=0x0f => SymbolType::ProcessorSpecific(t - 0x0d),
                t => SymbolType::Unknown(t),
            },
        }
    }
}

impl From<SymbolInfo> for u8 {
    fn from(v: SymbolInfo) -> Self {
        let SymbolInfo {
            binding: binding,
            type_: type_,
        } = v;
        let high = match binding {
            SymbolBinding::Local => 0x00,
            SymbolBinding::Global => 0x01,
            SymbolBinding::Weak => 0x02,
            SymbolBinding::OsSpecific(t) => t + 0x0a,
            SymbolBinding::ProcessorSpecific(t) => t + 0x0d,
            SymbolBinding::Unknown(t) => t,
        };
        let low = match type_ {
            SymbolType::Nothing => 0x00,
            SymbolType::Object => 0x01,
            SymbolType::Function => 0x02,
            SymbolType::Section => 0x03,
            SymbolType::File => 0x04,
            SymbolType::OsSpecific(t) => t + 0x0a,
            SymbolType::ProcessorSpecific(t) => t + 0x0d,
            SymbolType::Unknown(t) => t,
        };
        high * 0x10 + low
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SymbolEntry {
    name: u32,
    info: SymbolInfo,
    reserved: u8,
    section_index: Index,
    value: Address,
    size: u64,
}

impl SymbolEntry {
    pub const SIZE: usize = 0x18;

    pub fn new(slice: &[u8], encoding: Encoding) -> Result<Self, Error> {
        use byteorder::{ByteOrder, LittleEndian, BigEndian};

        if slice.len() < Self::SIZE {
            return Err(Error::NotEnoughData);
        };

        if slice[0x05] != 0x00 {
            return Err(Error::ReservedFieldIsNotZero);
        };

        match encoding {
            Encoding::Little => Ok(SymbolEntry {
                name: LittleEndian::read_u32(&slice[0x00..0x04]),
                info: slice[0x04].into(),
                reserved: 0,
                section_index: LittleEndian::read_u16(&slice[0x06..0x08]).into(),
                value: LittleEndian::read_u64(&slice[0x08..0x10]),
                size: LittleEndian::read_u64(&slice[0x10..0x18]),
            }),
            Encoding::Big => Ok(SymbolEntry {
                name: BigEndian::read_u32(&slice[0x00..0x04]),
                info: slice[0x04].into(),
                reserved: 0,
                section_index: BigEndian::read_u16(&slice[0x06..0x08]).into(),
                value: BigEndian::read_u64(&slice[0x08..0x10]),
                size: BigEndian::read_u64(&slice[0x10..0x18]),
            }),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelEntry {
    address: Address,
    symbol_index: u32,
    relocation_type: u32,
}

impl RelEntry {
    pub const SIZE: usize = 0x10;

    pub fn new(slice: &[u8], encoding: Encoding) -> Result<Self, Error> {
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
    address: Address,
    symbol_index: u32,
    relocation_type: u32,
    addend: i64,
}

impl RelaEntry {
    pub const SIZE: usize = 0x18;

    pub fn new(slice: &[u8], encoding: Encoding) -> Result<Self, Error> {
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NoteEntry<'a> {
    pub type_: u64,
    pub name: &'a str,
    pub description: &'a [u8],
}
