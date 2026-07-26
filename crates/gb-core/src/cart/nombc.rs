//! Cartucho sem mapeador. spec: `docs/reference/08-cartridges-mbc.md` § No MBC.
//! RAM opcional ($08/$09) recusada — comportamento desconhecido (ver docs/iterations/0007).

use std::fmt;
use std::ops::RangeInclusive;

use super::{Cartridge, CartridgeError, OPEN_BUS};

const ROM_WINDOW: RangeInclusive<u16> = 0x0000..=0x7FFF;

pub struct NoMbc {
    rom: Box<[u8]>,
}

impl NoMbc {
    pub const MAX_ROM_LEN: usize = 32 * 1024;

    pub fn new(rom: Vec<u8>) -> Result<Self, CartridgeError> {
        if rom.len() > Self::MAX_ROM_LEN {
            return Err(CartridgeError::RomTooLarge { len: rom.len() });
        }

        Ok(Self {
            rom: rom.into_boxed_slice(),
        })
    }
}

impl Cartridge for NoMbc {
    fn read(&self, addr: u16) -> u8 {
        if !ROM_WINDOW.contains(&addr) {
            return OPEN_BUS;
        }

        self.rom.get(addr as usize).copied().unwrap_or(OPEN_BUS)
    }

    fn write(&mut self, _addr: u16, _value: u8) {}
}

impl fmt::Debug for NoMbc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NoMbc")
            .field("rom_len", &self.rom.len())
            .finish()
    }
}
