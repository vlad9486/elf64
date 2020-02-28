#![no_std]
#![forbid(unsafe_code)]
#![allow(non_shorthand_field_patterns)]

#[macro_use]
extern crate bitflags;

mod common;
pub use self::common::{Address, Offset, Error, UnexpectedSize, ErrorSliceLength};

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
            return Err(Error::slice_too_short());
        };

        let header = Header::new(&raw[0..Header::SIZE])?;
        let program_table = header.program_header_table(raw)?;

        let section_table = header.section_header_table(raw)?;
        let names = match &header.section_names {
            &Index::Regular(i) => {
                let names_section = section_table.pick(i as usize)?;
                match &names_section.type_ {
                    &SectionType::StringTable => {
                        let start = names_section.offset.clone() as usize;
                        let end = start + names_section.size.clone() as usize;
                        Some(StringTable::new(&raw[start..end]))
                    },
                    _ => None,
                }
            },
            _ => None,
        };

        Ok(Elf64 {
            raw: raw,
            header: header,
            program_table: program_table,
            section_table: section_table,
            names: names,
        })
    }

    pub fn class(&self) -> Class {
        self.header.identifier.class.clone()
    }

    pub fn encoding(&self) -> Encoding {
        self.header.identifier.encoding.clone()
    }

    pub fn version(&self) -> u8 {
        self.header.identifier.version.clone()
    }

    pub fn abi(&self) -> Abi {
        self.header.identifier.abi.clone()
    }

    pub fn abi_version(&self) -> u8 {
        self.header.identifier.abi_version.clone()
    }

    pub fn type_(&self) -> Type {
        self.header.type_.clone()
    }

    pub fn machine(&self) -> Machine {
        self.header.machine.clone()
    }

    pub fn format_version(&self) -> u32 {
        self.header.format_version.clone()
    }

    pub fn entry(&self) -> Address {
        self.header.entry.clone()
    }

    pub fn flags(&self) -> u32 {
        self.header.flags.clone()
    }

    pub fn program_number(&self) -> usize {
        self.program_table.length()
    }

    pub fn program(&self, index: usize) -> Result<Option<Program<'a>>, Error> {
        use core::str::from_utf8;

        let program_header = self.program_table.pick(index)?;
        let encoding = self.encoding();

        let start = program_header.file_offset.clone() as usize;
        let end = start + (program_header.file_size.clone() as usize);
        if self.raw.len() < end {
            return Err(Error::slice_too_short());
        };
        let slice = &self.raw[start..end];

        let data = match &program_header.type_ {
            &ProgramType::Null => None,
            &ProgramType::Load => Some(ProgramData::Load {
                data: slice,
                address: program_header.virtual_address.clone(),
            }),
            &ProgramType::Dynamic => unimplemented!("dynamic linking table"),
            &ProgramType::Interpreter => {
                let path = from_utf8(slice).map_err(Error::Utf8Error)?;
                Some(ProgramData::Interpreter(path))
            },
            &ProgramType::Note => Some(ProgramData::Note(NoteTable::new(slice, encoding))),
            &ProgramType::Shlib => None,
            &ProgramType::ProgramHeaderTable => None,
            &ProgramType::OsSpecific(code) => Some(ProgramData::OsSpecific {
                code: code,
                data: slice,
                address: program_header.virtual_address.clone(),
            }),
            &ProgramType::ProcessorSprcific(code) => Some(ProgramData::ProcessorSprcific {
                code: code,
                data: slice,
                address: program_header.virtual_address.clone(),
            }),
            &ProgramType::Unknown(code) => Some(ProgramData::Unknown {
                code: code,
                data: slice,
                address: program_header.virtual_address.clone(),
            }),
        };

        Ok(data.map(|d| Program {
            data: d,
            flags: program_header.flags.clone(),
            memory_size: program_header.memory_size.clone(),
            address_alignment: program_header.address_alignment.clone(),
        }))
    }

    pub fn section_number(&self) -> usize {
        self.section_table.length()
    }

    pub fn section(&self, index: usize) -> Result<Option<Section<'a>>, Error> {
        let section_header = self.section_table.pick(index)?;
        let encoding = self.encoding();

        let start = section_header.offset.clone() as usize;
        let end = start + (section_header.size.clone() as usize);
        if self.raw.len() < end {
            return Err(Error::slice_too_short());
        };
        let slice = &self.raw[start..end];

        let data = match &section_header.type_ {
            &SectionType::Null => None,
            &SectionType::ProgramBits => Some(SectionData::ProgramBits(slice)),
            &SectionType::SymbolTable => Some(SectionData::SymbolTable {
                table: Table::new(slice, encoding),
                number_of_locals: section_header.info.clone() as usize,
            }),
            &SectionType::StringTable => Some(SectionData::StringTable(StringTable::new(slice))),
            &SectionType::Rela => Some(SectionData::Rela {
                table: Table::new(slice, encoding),
                apply_to_section: (section_header.info.clone() as u16).into(),
            }),
            &SectionType::Hash => unimplemented!("hash table"),
            &SectionType::Dynamic => unimplemented!("dynamic linking table"),
            &SectionType::Note => Some(SectionData::Note(NoteTable::new(slice, encoding))),
            &SectionType::NoBits => None,
            &SectionType::Rel => Some(SectionData::Rel {
                table: Table::new(slice, encoding),
                apply_to_section: (section_header.info.clone() as u16).into(),
            }),
            &SectionType::Shlib => None,
            &SectionType::DynamicSymbolTable => Some(SectionData::DynamicSymbolTable {
                table: Table::new(slice, encoding),
                number_of_locals: section_header.info.clone() as usize,
            }),
            &SectionType::OsSpecific(code) => Some(SectionData::OsSpecific {
                code: code,
                data: slice,
            }),
            &SectionType::ProcessorSprcific(code) => Some(SectionData::ProcessorSprcific {
                code: code,
                data: slice,
            }),
            &SectionType::Unknown(code) => Some(SectionData::Unknown {
                code: code,
                data: slice,
            }),
        };

        let name = match &self.names {
            Some(ref table) => table.pick(section_header.name.clone() as usize)?,
            None => "",
        };

        Ok(data.map(|d| Section {
            data: d,
            name: name,
            flags: section_header.flags.clone(),
            address: section_header.address.clone(),
            address_alignment: section_header.address_alignment.clone(),
            link: section_header.link.clone(),
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
    Interpreter(&'a str),
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
        data: &'a [u8],
    },
    ProcessorSprcific {
        code: u32,
        data: &'a [u8],
    },
    Unknown {
        code: u32,
        data: &'a [u8],
    },
}

#[derive(Clone)]
pub struct Section<'a> {
    pub data: SectionData<'a>,
    pub name: &'a str,
    pub flags: SectionFlags,
    pub address: Address,
    pub address_alignment: u64,
    pub link: Index,
}
