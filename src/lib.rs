#![no_std]
#![forbid(unsafe_code)]

#[macro_use]
extern crate bitflags;

macro_rules! read_int {
    ($slice:expr, $encoding:expr, $ty:ty) => {{
        let mut a = [0; core::mem::size_of::<$ty>()];
        a.clone_from_slice(&$slice[..core::mem::size_of::<$ty>()]);
        match $encoding {
            &Encoding::Little => <$ty>::from_le_bytes(a),
            &Encoding::Big => <$ty>::from_be_bytes(a),
        }
    }};
}

mod common;
pub use self::common::{Address, Offset, Error, UnexpectedSize};

mod header;
use self::header::Header;
pub use self::header::{Class, Encoding, Abi, Type, Machine};

mod section;
use self::section::SectionHeader;
pub use self::section::{Index, SectionType, SectionFlags};

mod program;
use self::program::{ProgramType, ProgramHeader};
pub use self::program::ProgramFlags;

mod symbol;
pub use self::symbol::{SymbolBinding, SymbolType, SymbolInfo, SymbolEntry};

mod rel_rela;
pub use self::rel_rela::{RelEntry, RelaEntry};

mod string_note;
pub use self::string_note::{StringTable, NoteEntry, NoteTable};

mod table;
pub use self::table::{Entry, Table};

#[derive(Clone)]
pub struct Elf64<'a> {
    raw: &'a [u8],
    header: Header,
    program_table: Table<'a, ProgramHeader>,
    section_table: Table<'a, SectionHeader>,
    names: Option<StringTable<'a>>,
}

impl<'a> Elf64<'a> {
    pub fn new(raw: &'a [u8]) -> Result<Self, Error> {
        if raw.len() < Header::SIZE {
            return Err(Error::SliceTooShort);
        };

        let header = Header::new(&raw[0..Header::SIZE])?;
        let program_table = header.program_header_table(raw)?;

        let section_table = header.section_header_table(raw)?;
        let names = match header.section_names {
            Index::Regular(i) => {
                let names_section = section_table.pick(i as usize)?;
                match names_section.ty {
                    SectionType::StringTable => {
                        let start = names_section.offset as usize;
                        if raw.len() < start {
                            return Err(Error::SliceTooShort);
                        }
                        Some(StringTable::new(&raw[start..]))
                    }
                    _ => None,
                }
            }
            _ => None,
        };

        Ok(Elf64 {
            raw,
            header,
            program_table,
            section_table,
            names,
        })
    }

    pub fn class(&self) -> Class {
        self.header.identifier.class.clone()
    }

    pub fn encoding(&self) -> Encoding {
        self.header.identifier.encoding.clone()
    }

    pub fn version(&self) -> u8 {
        self.header.identifier.version
    }

    pub fn abi(&self) -> Abi {
        self.header.identifier.abi.clone()
    }

    pub fn abi_version(&self) -> u8 {
        self.header.identifier.abi_version
    }

    pub fn ty(&self) -> Type {
        self.header.ty.clone()
    }

    pub fn machine(&self) -> Machine {
        self.header.machine.clone()
    }

    pub fn format_version(&self) -> u32 {
        self.header.format_version
    }

    pub fn entry(&self) -> Address {
        self.header.entry
    }

    pub fn flags(&self) -> u32 {
        self.header.flags
    }

    pub fn program_number(&self) -> usize {
        self.header.program_header_number as usize
    }

    pub fn program(&self, index: usize) -> Result<Option<Program<'a>>, Error> {
        let program_header = self.program_table.pick(index)?;
        let encoding = self.encoding();

        let slice = if self.raw.len() < program_header.file_offset as usize {
            return Err(Error::SliceTooShort);
        } else {
            &self.raw[(program_header.file_offset as usize)..]
        };
        if slice.len() < program_header.file_size as usize {
            return Err(Error::SliceTooShort);
        };
        let slice = &slice[..(program_header.file_size as usize)];

        let data = match program_header.ty {
            ProgramType::Null => None,
            ProgramType::Load => Some(ProgramData::Load {
                data: slice,
                address: program_header.virtual_address,
            }),
            // TODO:
            ProgramType::Dynamic => None,
            ProgramType::Interpreter => Some(ProgramData::Interpreter(slice)),
            ProgramType::Note => Some(ProgramData::Note(NoteTable::new(slice, encoding))),
            ProgramType::Shlib => None,
            ProgramType::ProgramHeaderTable => None,
            ProgramType::OsSpecific(code) => Some(ProgramData::OsSpecific {
                code,
                data: slice,
                address: program_header.virtual_address,
            }),
            ProgramType::ProcessorSprcific(code) => Some(ProgramData::ProcessorSprcific {
                code,
                data: slice,
                address: program_header.virtual_address,
            }),
            ProgramType::Unknown(code) => Some(ProgramData::Unknown {
                code,
                data: slice,
                address: program_header.virtual_address,
            }),
        };

        Ok(data.map(|d| Program {
            data: d,
            flags: program_header.flags,
            memory_size: program_header.memory_size,
            address_alignment: program_header.address_alignment,
        }))
    }

    pub fn section_number(&self) -> usize {
        self.header.section_header_number as usize
    }

    pub fn section(&self, index: usize) -> Result<Option<Section<'a>>, Error> {
        let section_header = self.section_table.pick(index)?;
        let encoding = self.encoding();

        let start = section_header.offset as usize;
        let end = start + (section_header.size as usize);
        if self.raw.len() < end {
            return Err(Error::SliceTooShort);
        };
        let slice = &self.raw[start..end];

        let data = match section_header.ty {
            SectionType::Null => None,
            SectionType::ProgramBits => Some(SectionData::ProgramBits(slice)),
            SectionType::SymbolTable => Some(SectionData::SymbolTable {
                table: Table::new(slice, encoding),
                number_of_locals: section_header.info as usize,
            }),
            SectionType::StringTable => Some(SectionData::StringTable(StringTable::new(slice))),
            SectionType::Rela => Some(SectionData::Rela {
                table: Table::new(slice, encoding),
                apply_to_section: (section_header.info as u16).into(),
            }),
            // TODO:
            SectionType::Hash => None,
            SectionType::Dynamic => None,
            SectionType::Note => Some(SectionData::Note(NoteTable::new(slice, encoding))),
            SectionType::NoBits => None,
            SectionType::Rel => Some(SectionData::Rel {
                table: Table::new(slice, encoding),
                apply_to_section: (section_header.info as u16).into(),
            }),
            SectionType::Shlib => None,
            SectionType::DynamicSymbolTable => Some(SectionData::DynamicSymbolTable {
                table: Table::new(slice, encoding),
                number_of_locals: section_header.info as usize,
            }),
            SectionType::OsSpecific(code) => Some(SectionData::OsSpecific { code, slice }),
            SectionType::ProcessorSprcific(code) => {
                Some(SectionData::ProcessorSprcific { code, slice })
            }
            SectionType::Unknown(code) => Some(SectionData::Unknown { code, slice }),
        };

        let name = match &self.names {
            Some(ref table) => table.pick(section_header.name as usize)?,
            None => &[],
        };

        Ok(data.map(|data| Section {
            data,
            name,
            flags: section_header.flags,
            address: section_header.address,
            address_alignment: section_header.address_alignment,
            link: section_header.link,
        }))
    }
}

#[derive(Clone)]
pub enum ProgramData<'a> {
    Null,
    Load {
        data: &'a [u8],
        address: Address,
    },
    Interpreter(&'a [u8]),
    Note(NoteTable<'a>),
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
    pub flags: ProgramFlags,
    pub memory_size: u64,
    pub address_alignment: u64,
}

#[derive(Clone)]
pub enum SectionData<'a> {
    Null,
    ProgramBits(&'a [u8]),
    SymbolTable {
        table: Table<'a, SymbolEntry>,
        number_of_locals: usize,
    },
    StringTable(StringTable<'a>),
    Rela {
        table: Table<'a, RelaEntry>,
        apply_to_section: Index,
    },
    Note(NoteTable<'a>),
    Rel {
        table: Table<'a, RelEntry>,
        apply_to_section: Index,
    },
    DynamicSymbolTable {
        table: Table<'a, SymbolEntry>,
        number_of_locals: usize,
    },
    OsSpecific {
        code: u32,
        slice: &'a [u8],
    },
    ProcessorSprcific {
        code: u32,
        slice: &'a [u8],
    },
    Unknown {
        code: u32,
        slice: &'a [u8],
    },
}

#[derive(Clone)]
pub struct Section<'a> {
    pub data: SectionData<'a>,
    pub name: &'a [u8],
    pub flags: SectionFlags,
    pub address: Address,
    pub address_alignment: u64,
    pub link: Index,
}
