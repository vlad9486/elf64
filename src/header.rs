use super::{
    Error, UnexpectedSize, ErrorSliceLength, Address, Offset, Index, SectionHeader, ProgramHeader,
    Entry, Table,
};

use core::{convert::TryFrom, fmt};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Class {
    _32,
    _64,
    Unknown(u8),
}

impl From<u8> for Class {
    fn from(v: u8) -> Self {
        match v {
            1 => Class::_32,
            2 => Class::_64,
            t => Class::Unknown(t),
        }
    }
}

impl From<Class> for u8 {
    fn from(v: Class) -> Self {
        match v {
            Class::_32 => 0x01,
            Class::_64 => 0x02,
            Class::Unknown(t) => t,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Encoding {
    Little,
    Big,
}

impl TryFrom<u8> for Encoding {
    type Error = u8;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            1 => Ok(Encoding::Little),
            2 => Ok(Encoding::Big),
            t => Err(t),
        }
    }
}

impl From<Encoding> for u8 {
    fn from(v: Encoding) -> Self {
        match v {
            Encoding::Little => 0x01,
            Encoding::Big => 0x02,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Abi {
    SystemV,
    HpUx,
    NetBSD,
    Linux,
    Solaris,
    Aix,
    Irix,
    FreeBSD,
    OpenBSD,
    OpenVMS,
    Standalone,
    Unknown(u8),
}

impl From<u8> for Abi {
    fn from(v: u8) -> Self {
        match v {
            0x00 => Abi::SystemV,
            0x01 => Abi::HpUx,
            0x02 => Abi::NetBSD,
            0x03 => Abi::Linux,
            0x06 => Abi::Solaris,
            0x07 => Abi::Aix,
            0x08 => Abi::Irix,
            0x09 => Abi::FreeBSD,
            0x0c => Abi::OpenBSD,
            0x0d => Abi::OpenVMS,
            0xff => Abi::Standalone,
            t => Abi::Unknown(t),
        }
    }
}

impl From<Abi> for u8 {
    fn from(v: Abi) -> Self {
        match v {
            Abi::SystemV => 0x00,
            Abi::HpUx => 0x01,
            Abi::NetBSD => 0x02,
            Abi::Linux => 0x03,
            Abi::Solaris => 0x06,
            Abi::Aix => 0x07,
            Abi::Irix => 0x08,
            Abi::FreeBSD => 0x09,
            Abi::OpenBSD => 0x0c,
            Abi::OpenVMS => 0x0d,
            Abi::Standalone => 0xff,
            Abi::Unknown(t) => t,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Identifier {
    pub class: Class,
    pub encoding: Encoding,
    pub version: u8,
    pub abi: Abi,
    pub abi_version: u8,
}

impl Identifier {
    pub fn new(slice: &[u8]) -> Result<Self, Error> {
        use core::convert::TryInto;

        if !(slice[0x00] == 0x7f && slice[0x01..0x04].eq(b"ELF")) {
            return Err(Error::WrongMagicNumber);
        };
        Ok(Identifier {
            class: slice[0x04].into(),
            encoding: slice[0x05].try_into().map_err(Error::UnknownEncoding)?,
            version: slice[0x06],
            abi: slice[0x07].into(),
            abi_version: slice[0x08],
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type {
    None,
    Relocatable,
    Executable,
    SharedObject,
    Core,
    OsSpecific(u8),
    ProcessorSpecific(u8),
    Unknown(u16),
}

impl From<u16> for Type {
    fn from(v: u16) -> Self {
        match v {
            0x0000 => Type::None,
            0x0001 => Type::Relocatable,
            0x0002 => Type::Executable,
            0x0003 => Type::SharedObject,
            0x0004 => Type::Core,
            t @ 0xfe00..=0xfeff => Type::OsSpecific((t & 0x00ff) as u8),
            t @ 0xff00..=0xffff => Type::ProcessorSpecific((t & 0x00ff) as u8),
            t => Type::Unknown(t),
        }
    }
}

impl From<Type> for u16 {
    fn from(v: Type) -> Self {
        match v {
            Type::None => 0x0000,
            Type::Relocatable => 0x0001,
            Type::Executable => 0x0002,
            Type::SharedObject => 0x0003,
            Type::Core => 0x0004,
            Type::OsSpecific(t) => 0xfe00 + (t as u16),
            Type::ProcessorSpecific(t) => 0xff00 + (t as u16),
            Type::Unknown(t) => t,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Machine {
    None,
    Sparc,
    X86,
    Mips,
    PowerPC,
    Arm,
    SuperH,
    Ia64,
    X86_64,
    AArch64,
    BPF,
    Unknown(u16),
}

impl From<u16> for Machine {
    fn from(v: u16) -> Self {
        match v {
            0x0000 => Machine::None,
            0x0002 => Machine::Sparc,
            0x0003 => Machine::X86,
            0x0008 => Machine::Mips,
            0x0014 => Machine::PowerPC,
            0x0028 => Machine::Arm,
            0x002a => Machine::SuperH,
            0x0032 => Machine::Ia64,
            0x003e => Machine::X86_64,
            0x00b7 => Machine::AArch64,
            0x00f7 => Machine::BPF,
            t => Machine::Unknown(t),
        }
    }
}

impl From<Machine> for u16 {
    fn from(v: Machine) -> Self {
        match v {
            Machine::None => 0x0000,
            Machine::Sparc => 0x0002,
            Machine::X86 => 0x0003,
            Machine::Mips => 0x0008,
            Machine::PowerPC => 0x0014,
            Machine::Arm => 0x0028,
            Machine::SuperH => 0x002a,
            Machine::Ia64 => 0x0032,
            Machine::X86_64 => 0x003e,
            Machine::AArch64 => 0x00b7,
            Machine::BPF => 0x00f7,
            Machine::Unknown(t) => t,
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Header {
    pub identifier: Identifier,
    pub type_: Type,
    pub machine: Machine,
    pub format_version: u32,
    pub entry: Address,
    pub program_headers_offset: Offset,
    pub section_headers_offset: Offset,
    pub flags: u32,
    pub program_header_number: u16,
    pub section_header_number: u16,
    pub section_names: Index,
}

impl<'a> fmt::Debug for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Header")
            .field("class", &self.identifier.class)
            .field("encoding", &self.identifier.encoding)
            .field("version", &self.identifier.version)
            .field("abi", &self.identifier.abi)
            .field("abi_version", &self.identifier.abi_version)
            .field("type", &self.type_)
            .field("machine", &self.machine)
            .field("format_version", &self.format_version)
            .field("entry", &format_args!("0x{:08x}", self.entry))
            .field("flags", &self.flags)
            .field("section_names", &self.section_names)
            .finish()
    }
}

impl Header {
    pub const SIZE: usize = 0x40;

    pub fn new(slice: &[u8]) -> Result<Self, Error> {
        use byteorder::{ByteOrder, LittleEndian, BigEndian};

        let identifier = Identifier::new(&slice[0x00..0x10])?;
        match identifier.encoding {
            Encoding::Little => {
                if LittleEndian::read_u16(&slice[0x34..0x36]) as usize != Self::SIZE {
                    return Err(Error::UnexpectedSize(UnexpectedSize::Header));
                };
                if LittleEndian::read_u16(&slice[0x36..0x38]) as usize != ProgramHeader::SIZE {
                    return Err(Error::UnexpectedSize(UnexpectedSize::ProgramHeader));
                };
                if LittleEndian::read_u16(&slice[0x3a..0x3c]) as usize != SectionHeader::SIZE {
                    return Err(Error::UnexpectedSize(UnexpectedSize::SectionHeader));
                };
                Ok(Header {
                    identifier: identifier,
                    type_: LittleEndian::read_u16(&slice[0x10..0x12]).into(),
                    machine: LittleEndian::read_u16(&slice[0x12..0x14]).into(),
                    format_version: LittleEndian::read_u32(&slice[0x14..0x18]),
                    entry: LittleEndian::read_u64(&slice[0x18..0x20]),
                    program_headers_offset: LittleEndian::read_u64(&slice[0x20..0x28]),
                    section_headers_offset: LittleEndian::read_u64(&slice[0x28..0x30]),
                    flags: LittleEndian::read_u32(&slice[0x30..0x34]),
                    program_header_number: LittleEndian::read_u16(&slice[0x38..0x3a]),
                    section_header_number: LittleEndian::read_u16(&slice[0x3c..0x3e]),
                    section_names: LittleEndian::read_u16(&slice[0x3e..0x40]).into(),
                })
            },
            Encoding::Big => {
                if BigEndian::read_u16(&slice[0x34..0x36]) as usize != Self::SIZE {
                    return Err(Error::UnexpectedSize(UnexpectedSize::Header));
                };
                if BigEndian::read_u16(&slice[0x36..0x38]) as usize != ProgramHeader::SIZE {
                    return Err(Error::UnexpectedSize(UnexpectedSize::ProgramHeader));
                };
                if BigEndian::read_u16(&slice[0x3a..0x3c]) as usize != SectionHeader::SIZE {
                    return Err(Error::UnexpectedSize(UnexpectedSize::SectionHeader));
                };
                Ok(Header {
                    identifier: identifier,
                    type_: BigEndian::read_u16(&slice[0x10..0x12]).into(),
                    machine: BigEndian::read_u16(&slice[0x12..0x14]).into(),
                    format_version: BigEndian::read_u32(&slice[0x14..0x18]),
                    entry: BigEndian::read_u64(&slice[0x18..0x20]),
                    program_headers_offset: BigEndian::read_u64(&slice[0x20..0x28]),
                    section_headers_offset: BigEndian::read_u64(&slice[0x28..0x30]),
                    flags: BigEndian::read_u32(&slice[0x30..0x34]),
                    program_header_number: BigEndian::read_u16(&slice[0x38..0x3a]),
                    section_header_number: BigEndian::read_u16(&slice[0x3c..0x3e]),
                    section_names: BigEndian::read_u16(&slice[0x3e..0x40]).into(),
                })
            },
        }
    }

    pub fn program_header_table<'a>(
        &self,
        raw: &'a [u8],
    ) -> Result<Table<'a, ProgramHeader>, Error> {
        let start = self.program_headers_offset as usize;
        let end = start + (self.program_header_number as usize) * ProgramHeader::SIZE;
        if raw.len() < end {
            return Err(Error::slice_too_short());
        };
        Ok(Table::new(
            &raw[start..end],
            self.identifier.encoding.clone(),
        ))
    }

    pub fn section_header_table<'a>(
        &self,
        raw: &'a [u8],
    ) -> Result<Table<'a, SectionHeader>, Error> {
        let start = self.section_headers_offset as usize;
        let end = start + (self.section_header_number as usize) * SectionHeader::SIZE;
        if raw.len() < end {
            return Err(Error::slice_too_short());
        };
        Ok(Table::new(
            &raw[start..end],
            self.identifier.encoding.clone(),
        ))
    }
}
