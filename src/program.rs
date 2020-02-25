use super::{Address, Offset, Error, Encoding};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProgramType {
    Null,
    Load,
    Dynamic,
    Interpreter,
    Note,
    Shlib,
    ProgramHeaderTable,
    OsSpecific(u32),
    ProcessorSprcific(u32),
    Unknown(u32),
}

impl From<u32> for ProgramType {
    fn from(v: u32) -> Self {
        match v {
            0x00000000 => ProgramType::Null,
            0x00000001 => ProgramType::Load,
            0x00000002 => ProgramType::Dynamic,
            0x00000003 => ProgramType::Interpreter,
            0x00000004 => ProgramType::Note,
            0x00000005 => ProgramType::Shlib,
            0x00000006 => ProgramType::ProgramHeaderTable,
            t @ 0x60000000..=0x6fffffff => ProgramType::OsSpecific(t),
            t @ 0x70000000..=0x7fffffff => ProgramType::ProcessorSprcific(t),
            t => ProgramType::Unknown(t),
        }
    }
}

impl From<ProgramType> for u32 {
    fn from(v: ProgramType) -> Self {
        match v {
            ProgramType::Null => 0x00000000,
            ProgramType::Load => 0x00000001,
            ProgramType::Dynamic => 0x00000002,
            ProgramType::Interpreter => 0x00000003,
            ProgramType::Note => 0x00000004,
            ProgramType::Shlib => 0x00000005,
            ProgramType::ProgramHeaderTable => 0x00000006,
            ProgramType::OsSpecific(t) => 0x60000000 + t & 0x0fffffff,
            ProgramType::ProcessorSprcific(t) => 0x70000000 + t & 0x0fffffff,
            ProgramType::Unknown(t) => t,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgramHeader {
    type_: ProgramType,
    flags: u64,
    file_offset: Offset,
    virtual_address: Address,
    physical_address: Address,
    file_size: u64,
    memory_size: u64,
    address_alignment: u64,
}

impl ProgramHeader {
    pub const SIZE: usize = 0x38;

    pub fn new(slice: &[u8], encoding: Encoding) -> Result<Self, Error> {
        use byteorder::{ByteOrder, LittleEndian, BigEndian};

        if slice.len() < Self::SIZE {
            return Err(Error::NotEnoughData);
        };

        match encoding {
            Encoding::Little => Ok(ProgramHeader {
                type_: LittleEndian::read_u32(&slice[0x00..0x04]).into(),
                flags: LittleEndian::read_u64(&slice[0x04..0x08]),
                file_offset: LittleEndian::read_u64(&slice[0x08..0x10]),
                virtual_address: LittleEndian::read_u64(&slice[0x10..0x18]),
                physical_address: LittleEndian::read_u64(&slice[0x18..0x20]),
                file_size: LittleEndian::read_u64(&slice[0x20..0x28]),
                memory_size: LittleEndian::read_u64(&slice[0x28..0x30]),
                address_alignment: LittleEndian::read_u64(&slice[0x30..0x38]),
            }),
            Encoding::Big => Ok(ProgramHeader {
                type_: BigEndian::read_u32(&slice[0x00..0x04]).into(),
                flags: BigEndian::read_u64(&slice[0x04..0x08]),
                file_offset: BigEndian::read_u64(&slice[0x08..0x10]),
                virtual_address: BigEndian::read_u64(&slice[0x10..0x18]),
                physical_address: BigEndian::read_u64(&slice[0x18..0x20]),
                file_size: BigEndian::read_u64(&slice[0x20..0x28]),
                memory_size: BigEndian::read_u64(&slice[0x28..0x30]),
                address_alignment: BigEndian::read_u64(&slice[0x30..0x38]),
            }),
        }
    }
}

#[derive(Clone)]
pub struct ProgramHeaderTable<'a> {
    slice: &'a [u8],
    encoding: Encoding,
}

impl<'a> ProgramHeaderTable<'a> {
    pub fn new(slice: &'a [u8], encoding: Encoding) -> Self {
        ProgramHeaderTable {
            slice: slice,
            encoding: encoding,
        }
    }

    pub fn size(&self) -> usize {
        self.slice.len() / ProgramHeader::SIZE
    }

    pub fn pick(&self, index: usize) -> Result<ProgramHeader, Error> {
        let end = index + ProgramHeader::SIZE;
        ProgramHeader::new(&self.slice[index..end], self.encoding.clone())
    }
}
