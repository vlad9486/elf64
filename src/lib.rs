#![no_std]
#![forbid(unsafe_code)]
#![allow(non_shorthand_field_patterns)]

mod common;
pub use self::common::{Address, Offset, Error};

mod header;
pub use self::header::{Class, Encoding, Abi, Type, Machine, Header};

mod tables;
pub use self::tables::{
    ProgramHeaderTable, SectionHeaderTable, StringTable, SymbolTable, NoteTable,
};

mod section;
pub use self::section::{Index, SectionType, SectionHeader};

mod program;
pub use self::program::{ProgramType, ProgramHeader};

mod entries;
pub use self::entries::{
    SymbolBinding, SymbolType, SymbolInfo, SymbolEntry, RelEntry, RelaEntry, NoteEntry,
};
