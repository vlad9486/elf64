use super::{Address, Offset, Error, Encoding, Entry};

use core::fmt;

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

bitflags! {
    pub struct ProgramFlags: u32 {
        const EXECUTE = 0b00000001;
        const WRITE = 0b00000010;
        const READ = 0b00000100;
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct ProgramHeader {
    pub type_: ProgramType,
    pub flags: ProgramFlags,
    pub file_offset: Offset,
    pub virtual_address: Address,
    pub physical_address: Address,
    pub file_size: u64,
    pub memory_size: u64,
    pub address_alignment: u64,
}

impl<'a> fmt::Debug for ProgramHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProgramHeader")
            .field("type", &self.type_)
            .field("flags", &self.flags)
            .field("file_offset", &format_args!("0x{:016x}", self.file_offset))
            .field(
                "virtual_address",
                &format_args!("0x{:016x}", self.virtual_address),
            )
            .field(
                "physical_address",
                &format_args!("0x{:016x}", self.physical_address),
            )
            .field("file_size", &format_args!("0x{:016x}", self.file_size))
            .field("memory_size", &format_args!("0x{:016x}", self.memory_size))
            .field(
                "address_alignment",
                &format_args!("0x{:016x}", self.address_alignment),
            )
            .finish()
    }
}

impl Entry for ProgramHeader {
    type Error = Error;

    const SIZE: usize = 0x38;

    fn new(slice: &[u8], encoding: Encoding) -> Result<Self, Self::Error> {
        use byteorder::{ByteOrder, LittleEndian, BigEndian};

        let flags = ProgramFlags::from_bits_truncate;

        match encoding {
            Encoding::Little => Ok(ProgramHeader {
                type_: LittleEndian::read_u32(&slice[0x00..0x04]).into(),
                flags: flags(LittleEndian::read_u32(&slice[0x04..0x08])),
                file_offset: LittleEndian::read_u64(&slice[0x08..0x10]),
                virtual_address: LittleEndian::read_u64(&slice[0x10..0x18]),
                physical_address: LittleEndian::read_u64(&slice[0x18..0x20]),
                file_size: LittleEndian::read_u64(&slice[0x20..0x28]),
                memory_size: LittleEndian::read_u64(&slice[0x28..0x30]),
                address_alignment: LittleEndian::read_u64(&slice[0x30..0x38]),
            }),
            Encoding::Big => Ok(ProgramHeader {
                type_: BigEndian::read_u32(&slice[0x00..0x04]).into(),
                flags: flags(BigEndian::read_u32(&slice[0x04..0x08])),
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
