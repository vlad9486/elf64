use super::{Address, Error, Encoding, Index, Entry};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SymbolBinding {
    Local,
    Global,
    Weak,
    OsSpecific(u8),
    ProcessorSpecific(u8),
    Unknown(u8),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SymbolType {
    Nothing,
    Object,
    Function,
    Section,
    File,
    OsSpecific(u8),
    ProcessorSpecific(u8),
    Unknown(u8),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SymbolInfo {
    pub binding: SymbolBinding,
    pub type_: SymbolType,
}

impl From<u8> for SymbolInfo {
    fn from(v: u8) -> Self {
        SymbolInfo {
            binding: match (v & 0xf0) / 0x10 {
                0x00 => SymbolBinding::Local,
                0x01 => SymbolBinding::Global,
                0x02 => SymbolBinding::Weak,
                t @ 0x0a..=0x0c => SymbolBinding::OsSpecific(t - 0x0a),
                t @ 0x0d..=0x0f => SymbolBinding::ProcessorSpecific(t - 0x0d),
                t => SymbolBinding::Unknown(t),
            },
            type_: match v & 0x0f {
                0x00 => SymbolType::Nothing,
                0x01 => SymbolType::Object,
                0x02 => SymbolType::Function,
                0x03 => SymbolType::Section,
                0x04 => SymbolType::File,
                t @ 0x0a..=0x0c => SymbolType::OsSpecific(t - 0x0a),
                t @ 0x0d..=0x0f => SymbolType::ProcessorSpecific(t - 0x0d),
                t => SymbolType::Unknown(t),
            },
        }
    }
}

impl From<SymbolInfo> for u8 {
    fn from(v: SymbolInfo) -> Self {
        let SymbolInfo {
            binding: binding,
            type_: type_,
        } = v;
        let high = match binding {
            SymbolBinding::Local => 0x00,
            SymbolBinding::Global => 0x01,
            SymbolBinding::Weak => 0x02,
            SymbolBinding::OsSpecific(t) => t + 0x0a,
            SymbolBinding::ProcessorSpecific(t) => t + 0x0d,
            SymbolBinding::Unknown(t) => t,
        };
        let low = match type_ {
            SymbolType::Nothing => 0x00,
            SymbolType::Object => 0x01,
            SymbolType::Function => 0x02,
            SymbolType::Section => 0x03,
            SymbolType::File => 0x04,
            SymbolType::OsSpecific(t) => t + 0x0a,
            SymbolType::ProcessorSpecific(t) => t + 0x0d,
            SymbolType::Unknown(t) => t,
        };
        high * 0x10 + low
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SymbolEntry {
    pub name: u32,
    pub info: SymbolInfo,
    pub reserved: u8,
    pub section_index: Index,
    pub value: Address,
    pub size: u64,
}

impl Entry for SymbolEntry {
    type Error = Error;

    const SIZE: usize = 0x18;

    fn new(slice: &[u8], encoding: Encoding) -> Result<Self, Self::Error> {
        if slice.len() < Self::SIZE {
            return Err(Error::SliceTooShort);
        }

        Ok(SymbolEntry {
            name: read_int!(&slice[0x00..], &encoding, u32),
            info: slice[0x04].into(),
            reserved: slice[0x05],
            section_index: read_int!(&slice[0x06..], &encoding, u16).into(),
            value: read_int!(&slice[0x08..], &encoding, u64),
            size: read_int!(&slice[0x10..], &encoding, u64),
        })
    }
}
