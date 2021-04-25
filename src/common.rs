use core::str::Utf8Error;

pub type Address = u64;
pub type Offset = u64;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    SliceTooShort,
    WrongMagicNumber,
    UnknownEncoding(u8),
    Utf8Error(Utf8Error),
    UnexpectedSize(UnexpectedSize),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnexpectedSize {
    Header,
    ProgramHeader,
    SectionHeader,
}
