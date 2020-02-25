use super::{Address, Offset, Error, Encoding};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Index {
    Undefined,
    ProcessorSecific(u8),
    EnvironmentSpecific(u8),
    AbsoluteValue,
    Common,
    Regular(u16),
}

impl From<u16> for Index {
    fn from(v: u16) -> Self {
        match v {
            0x0000 => Index::Undefined,
            t @ 0xff00..=0xff1f => Index::ProcessorSecific((t & 0x001f) as u8),
            t @ 0xff20..=0xff3f => Index::EnvironmentSpecific((t & 0x001f) as u8),
            0xfff1 => Index::AbsoluteValue,
            0xfff2 => Index::Common,
            t => Index::Regular(t),
        }
    }
}

impl From<Index> for u16 {
    fn from(v: Index) -> Self {
        match v {
            Index::Undefined => 0x0000,
            Index::ProcessorSecific(t) => 0xff00 + ((t as u16) & 0x001f),
            Index::EnvironmentSpecific(t) => 0xff20 + ((t as u16) & 0x001f),
            Index::AbsoluteValue => 0xfff1,
            Index::Common => 0xfff2,
            Index::Regular(t) => t,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SectionType {
    Null,
    ProgramBits,
    SymbolTable,
    StringTable,
    Rela,
    Hash,
    Dynamic,
    Note,
    NoBits,
    Rel,
    Shlib,
    DynamicSymbolTable,
    OsSpecific(u32),
    ProcessorSprcific(u32),
    Unknown(u32),
}

impl From<u32> for SectionType {
    fn from(v: u32) -> Self {
        match v {
            0x00000000 => SectionType::Null,
            0x00000001 => SectionType::ProgramBits,
            0x00000002 => SectionType::SymbolTable,
            0x00000003 => SectionType::StringTable,
            0x00000004 => SectionType::Rela,
            0x00000005 => SectionType::Hash,
            0x00000006 => SectionType::Dynamic,
            0x00000007 => SectionType::Note,
            0x00000008 => SectionType::NoBits,
            0x00000009 => SectionType::Rel,
            0x0000000a => SectionType::Shlib,
            0x0000000b => SectionType::DynamicSymbolTable,
            t @ 0x60000000..=0x6fffffff => SectionType::OsSpecific(t),
            t @ 0x70000000..=0x7fffffff => SectionType::ProcessorSprcific(t),
            t => SectionType::Unknown(t),
        }
    }
}

impl From<SectionType> for u32 {
    fn from(v: SectionType) -> Self {
        match v {
            SectionType::Null => 0x00000000,
            SectionType::ProgramBits => 0x00000001,
            SectionType::SymbolTable => 0x00000002,
            SectionType::StringTable => 0x00000003,
            SectionType::Rela => 0x00000004,
            SectionType::Hash => 0x00000005,
            SectionType::Dynamic => 0x00000006,
            SectionType::Note => 0x00000007,
            SectionType::NoBits => 0x00000008,
            SectionType::Rel => 0x00000009,
            SectionType::Shlib => 0x0000000a,
            SectionType::DynamicSymbolTable => 0x0000000b,
            SectionType::OsSpecific(t) => 0x60000000 + t & 0x0fffffff,
            SectionType::ProcessorSprcific(t) => 0x70000000 + t & 0x0fffffff,
            SectionType::Unknown(t) => t,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SectionHeader {
    name: u32,
    type_: SectionType,
    flags: u64,
    address: Address,
    offset: Offset,
    size: u64,
    link: SectionType,
    info: SectionType,
    address_alignment: u64,
    number_of_entries: u64,
}

impl SectionHeader {
    pub const SIZE: usize = 0x40;

    pub fn new(slice: &[u8], encoding: Encoding) -> Result<Self, Error> {
        use byteorder::{ByteOrder, LittleEndian, BigEndian};

        if slice.len() < Self::SIZE {
            return Err(Error::NotEnoughData);
        };

        match encoding {
            Encoding::Little => Ok(SectionHeader {
                name: LittleEndian::read_u32(&slice[0x00..0x04]),
                type_: LittleEndian::read_u32(&slice[0x04..0x08]).into(),
                flags: LittleEndian::read_u64(&slice[0x08..0x10]),
                address: LittleEndian::read_u64(&slice[0x10..0x18]),
                offset: LittleEndian::read_u64(&slice[0x18..0x20]),
                size: LittleEndian::read_u64(&slice[0x20..0x28]),
                link: LittleEndian::read_u32(&slice[0x28..0x2c]).into(),
                info: LittleEndian::read_u32(&slice[0x2c..0x30]).into(),
                address_alignment: LittleEndian::read_u64(&slice[0x30..0x38]),
                number_of_entries: LittleEndian::read_u64(&slice[0x38..0x40]),
            }),
            Encoding::Big => Ok(SectionHeader {
                name: BigEndian::read_u32(&slice[0x00..0x04]),
                type_: BigEndian::read_u32(&slice[0x04..0x08]).into(),
                flags: BigEndian::read_u64(&slice[0x08..0x10]),
                address: BigEndian::read_u64(&slice[0x10..0x18]),
                offset: BigEndian::read_u64(&slice[0x18..0x20]),
                size: BigEndian::read_u64(&slice[0x20..0x28]),
                link: BigEndian::read_u32(&slice[0x28..0x2c]).into(),
                info: BigEndian::read_u32(&slice[0x2c..0x30]).into(),
                address_alignment: BigEndian::read_u64(&slice[0x30..0x38]),
                number_of_entries: BigEndian::read_u64(&slice[0x38..0x40]),
            }),
        }
    }
}

#[derive(Clone)]
pub struct SectionHeaderTable<'a> {
    slice: &'a [u8],
    encoding: Encoding,
}

impl<'a> SectionHeaderTable<'a> {
    pub fn new(slice: &'a [u8], encoding: Encoding) -> Self {
        SectionHeaderTable {
            slice: slice,
            encoding: encoding,
        }
    }

    pub fn size(&self) -> usize {
        self.slice.len() / SectionHeader::SIZE
    }

    pub fn pick(&self, index: usize) -> Result<SectionHeader, Error> {
        let end = index + SectionHeader::SIZE;
        SectionHeader::new(&self.slice[index..end], self.encoding.clone())
    }
}
