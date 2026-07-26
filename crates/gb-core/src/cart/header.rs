//! Cabeçalho do cartucho ($0100–$014F). spec: `docs/reference/08-cartridges-mbc.md` § The Cartridge Header.

use std::fmt;
use std::ops::{Range, RangeInclusive};

const TITLE: Range<usize> = 0x0134..0x0144;
const CARTRIDGE_TYPE: usize = 0x0147;
const ROM_SIZE: usize = 0x0148;
const RAM_SIZE: usize = 0x0149;
const CHECKSUMMED: RangeInclusive<usize> = 0x0134..=0x014C;
const CHECKSUM: usize = 0x014D;

pub const MIN_ROM_LEN: usize = 0x0150;

const ROM_BANK_SIZE: u32 = 16 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderError {
    TooShort { len: usize },
}

impl fmt::Display for HeaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooShort { len } => write!(
                f,
                "ROM tem {len} bytes e o cabeçalho vai até $014F: \
                 são necessários pelo menos {MIN_ROM_LEN}"
            ),
        }
    }
}

impl std::error::Error for HeaderError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CartridgeType(u8);

impl CartridgeType {
    #[must_use]
    pub const fn code(self) -> u8 {
        self.0
    }

    #[must_use]
    pub const fn name(self) -> Option<&'static str> {
        Some(match self.0 {
            0x00 => "ROM ONLY",
            0x01 => "MBC1",
            0x02 => "MBC1+RAM",
            0x03 => "MBC1+RAM+BATTERY",
            0x05 => "MBC2",
            0x06 => "MBC2+BATTERY",
            0x08 => "ROM+RAM",
            0x09 => "ROM+RAM+BATTERY",
            0x0B => "MMM01",
            0x0C => "MMM01+RAM",
            0x0D => "MMM01+RAM+BATTERY",
            0x0F => "MBC3+TIMER+BATTERY",
            0x10 => "MBC3+TIMER+RAM+BATTERY",
            0x11 => "MBC3",
            0x12 => "MBC3+RAM",
            0x13 => "MBC3+RAM+BATTERY",
            0x19 => "MBC5",
            0x1A => "MBC5+RAM",
            0x1B => "MBC5+RAM+BATTERY",
            0x1C => "MBC5+RUMBLE",
            0x1D => "MBC5+RUMBLE+RAM",
            0x1E => "MBC5+RUMBLE+RAM+BATTERY",
            0x20 => "MBC6",
            0x22 => "MBC7+SENSOR+RUMBLE+RAM+BATTERY",
            0xFC => "POCKET CAMERA",
            0xFD => "BANDAI TAMA5",
            0xFE => "HuC3",
            0xFF => "HuC1+RAM+BATTERY",
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RomSize(u8);

impl RomSize {
    #[must_use]
    pub const fn code(self) -> u8 {
        self.0
    }

    // 32 KiB × (1 << code). $52/$53/$54 são None — spec diz que não são oficiais
    // e provavelmente estão errados (ver docs/iterations/0005, erro #2).
    #[must_use]
    pub const fn bytes(self) -> Option<u32> {
        match self.0 {
            code @ 0x00..=0x08 => Some((32 * 1024) << code),
            _ => None,
        }
    }

    #[must_use]
    pub const fn banks(self) -> Option<u16> {
        match self.bytes() {
            Some(bytes) => Some((bytes / ROM_BANK_SIZE) as u16),
            None => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RamSize(u8);

impl RamSize {
    #[must_use]
    pub const fn code(self) -> u8 {
        self.0
    }

    // Isto não é fórmula: $04 = 128 KiB, $05 = 64 KiB — a tabela não é monotônica.
    // $01 é None: "Unused", chip de 2 KiB nunca usado (ver docs/iterations/0005, erro #1).
    #[must_use]
    pub const fn bytes(self) -> Option<u32> {
        match self.0 {
            0x00 => Some(0),
            0x02 => Some(8 * 1024),
            0x03 => Some(32 * 1024),
            0x04 => Some(128 * 1024),
            0x05 => Some(64 * 1024),
            _ => None,
        }
    }
}

// Checksum do cabeçalho: gravado em $014D × calculado a partir de $0134–$014C.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeaderChecksum {
    stored: u8,
    computed: u8,
}

impl HeaderChecksum {
    #[must_use]
    pub const fn stored(self) -> u8 {
        self.stored
    }

    #[must_use]
    pub const fn computed(self) -> u8 {
        self.computed
    }

    #[must_use]
    pub const fn is_valid(self) -> bool {
        self.stored == self.computed
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CartridgeHeader {
    title: String,
    cartridge_type: CartridgeType,
    rom_size: RomSize,
    ram_size: RamSize,
    checksum: HeaderChecksum,
}

impl CartridgeHeader {
    pub fn parse(rom: &[u8]) -> Result<Self, HeaderError> {
        if rom.len() < MIN_ROM_LEN {
            return Err(HeaderError::TooShort { len: rom.len() });
        }

        Ok(Self {
            title: parse_title(&rom[TITLE]),
            cartridge_type: CartridgeType(rom[CARTRIDGE_TYPE]),
            rom_size: RomSize(rom[ROM_SIZE]),
            ram_size: RamSize(rom[RAM_SIZE]),
            checksum: HeaderChecksum {
                stored: rom[CHECKSUM],
                computed: compute_checksum(rom),
            },
        })
    }

    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    #[must_use]
    pub const fn cartridge_type(&self) -> CartridgeType {
        self.cartridge_type
    }

    #[must_use]
    pub const fn rom_size(&self) -> RomSize {
        self.rom_size
    }

    #[must_use]
    pub const fn ram_size(&self) -> RamSize {
        self.ram_size
    }

    #[must_use]
    pub const fn checksum(&self) -> HeaderChecksum {
        self.checksum
    }
}

// Trecho inicial de ASCII imprimível em $0134–$0143. Para no primeiro byte
// não imprimível (títulos curtos arrastariam o CGB flag).
fn parse_title(field: &[u8]) -> String {
    let printable: String = field
        .iter()
        .copied()
        .take_while(|byte| byte.is_ascii_graphic() || *byte == b' ')
        .map(char::from)
        .collect();

    printable.trim_end().to_string()
}

// checksum = checksum - rom[address] - 1 (para cada byte de $0134–$014C).
// O -1 por byte distingue de soma comum: cabeçalho zerado bateria com $00.
fn compute_checksum(rom: &[u8]) -> u8 {
    rom[CHECKSUMMED]
        .iter()
        .fold(0u8, |acc, &byte| acc.wrapping_sub(byte).wrapping_sub(1))
}
