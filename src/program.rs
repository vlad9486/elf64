use core::fmt;
use super::{Address, Offset, Error, Encoding, Entry};

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

bitflags! {
    pub struct ProgramFlags: u32 {
        const EXECUTE = 0b00000001;
        const WRITE = 0b00000010;
        const READ = 0b00000100;
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct ProgramHeader {
    pub ty: ProgramType,
    pub flags: ProgramFlags,
    pub file_offset: Offset,
    pub virtual_address: Address,
    pub physical_address: Address,
    pub file_size: u64,
    pub memory_size: u64,
    pub address_alignment: u64,
}

impl fmt::Debug for ProgramHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProgramHeader")
            .field("type", &self.ty)
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
        if slice.len() < Self::SIZE {
            return Err(Error::SliceTooShort);
        }

        Ok(ProgramHeader {
            ty: read_int!(&slice[0x00..], &encoding, u32).into(),
            flags: ProgramFlags::from_bits_truncate(read_int!(&slice[0x04..], &encoding, u32)),
            file_offset: read_int!(&slice[0x08..], &encoding, u64),
            virtual_address: read_int!(&slice[0x10..], &encoding, u64),
            physical_address: read_int!(&slice[0x18..], &encoding, u64),
            file_size: read_int!(&slice[0x20..], &encoding, u64),
            memory_size: read_int!(&slice[0x28..], &encoding, u64),
            address_alignment: read_int!(&slice[0x30..], &encoding, u64),
        })
    }
}
