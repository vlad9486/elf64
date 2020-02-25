use super::{Address, Offset, Error, Encoding, Entry, Table, NoteTable};

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
    flags: u32,
    file_offset: Offset,
    virtual_address: Address,
    physical_address: Address,
    file_size: u64,
    memory_size: u64,
    address_alignment: u64,
}

impl Entry for ProgramHeader {
    type Error = Error;

    const SIZE: usize = 0x38;

    fn new(slice: &[u8], encoding: Encoding) -> Result<Self, Self::Error> {
        use byteorder::{ByteOrder, LittleEndian, BigEndian};

        if slice.len() < Self::SIZE {
            return Err(Error::NotEnoughData);
        };

        match encoding {
            Encoding::Little => Ok(ProgramHeader {
                type_: LittleEndian::read_u32(&slice[0x00..0x04]).into(),
                flags: LittleEndian::read_u32(&slice[0x04..0x08]),
                file_offset: LittleEndian::read_u64(&slice[0x08..0x10]),
                virtual_address: LittleEndian::read_u64(&slice[0x10..0x18]),
                physical_address: LittleEndian::read_u64(&slice[0x18..0x20]),
                file_size: LittleEndian::read_u64(&slice[0x20..0x28]),
                memory_size: LittleEndian::read_u64(&slice[0x28..0x30]),
                address_alignment: LittleEndian::read_u64(&slice[0x30..0x38]),
            }),
            Encoding::Big => Ok(ProgramHeader {
                type_: BigEndian::read_u32(&slice[0x00..0x04]).into(),
                flags: BigEndian::read_u32(&slice[0x04..0x08]),
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
pub enum ProgramData<'a> {
    Null,
    Load {
        data: &'a [u8],
        address: Address,
    },
    Note(NoteTable<'a>),
    ProgramHeaderTable(Table<'a, ProgramHeader>),
    OsSpecific {
        code: u32,
        data: &'a [u8],
        address: Address,
    },
    ProcessorSprcific {
        code: u32,
        data: &'a [u8],
        address: Address,
    },
    Unknown {
        code: u32,
        data: &'a [u8],
        address: Address,
    },
}

#[derive(Clone)]
pub struct Program<'a> {
    pub data: ProgramData<'a>,
    pub flags: u32,
    pub memory_size: u64,
    pub address_alignment: u64,
}

impl ProgramHeader {
    pub fn get_data<'a>(
        &self,
        raw: &'a [u8],
        encoding: Encoding,
    ) -> Result<Option<Program<'a>>, Error> {
        let start = self.file_offset.clone() as usize;
        let end = start + (self.file_size.clone() as usize);
        let slice = &raw[start..end];

        let data = match &self.type_ {
            &ProgramType::Null => None,
            &ProgramType::Load => Some(ProgramData::Load {
                data: slice,
                address: self.virtual_address.clone(),
            }),
            &ProgramType::Dynamic => unimplemented!(),
            &ProgramType::Interpreter => unimplemented!(),
            &ProgramType::Note => Some(ProgramData::Note(NoteTable::new(slice, encoding))),
            &ProgramType::Shlib => None,
            &ProgramType::ProgramHeaderTable => Some(ProgramData::ProgramHeaderTable(Table::new(slice, encoding))),
            &ProgramType::OsSpecific(code) => Some(ProgramData::OsSpecific {
                code: code,
                data: slice,
                address: self.virtual_address.clone(),
            }),
            &ProgramType::ProcessorSprcific(code) => Some(ProgramData::ProcessorSprcific {
                code: code,
                data: slice,
                address: self.virtual_address.clone(),
            }),
            &ProgramType::Unknown(code) => Some(ProgramData::Unknown {
                code: code,
                data: slice,
                address: self.virtual_address.clone(),
            }),
        };

        Ok(data.map(|d| {
            Program {
                data: d,
                flags: self.flags.clone(),
                memory_size: self.memory_size.clone(),
                address_alignment: self.address_alignment.clone(),
            }
        }))
    }
}
