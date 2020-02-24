use core::str::Utf8Error;

pub type Address = u64;
pub type Offset = u64;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    NotEnoughData,
    WrongMagicNumber,
    UnknownEncoding(u8),
    ReservedFieldIsNotZero,
    Utf8Error(Utf8Error),
    UnexpectedHeaderSize,
    UnexpectedProgramHeaderSize,
    UnexpectedSectionHeaderSize,
}
