#![no_std]
#![forbid(unsafe_code)]
#![allow(non_shorthand_field_patterns)]

mod common;
pub use self::common::{Address, Offset, Error};

mod header;
pub use self::header::{Class, Encoding, Abi, Type, Machine, Header};

mod section;
pub use self::section::{Index, SectionType, SectionHeader};

mod program;
pub use self::program::{ProgramType, ProgramHeader};

mod symbol;
pub use self::symbol::{SymbolBinding, SymbolType, SymbolInfo, SymbolEntry};

mod rel_rela;
pub use self::rel_rela::{RelEntry, RelaEntry};

mod string_note;
pub use self::string_note::{StringTable, NoteEntry, NoteTable};

mod table;
pub use self::table::{Entry, Table};
