//! MBC1 minimal: ROM banking, sem RAM. spec: `docs/reference/08-cartridges-mbc.md` § MBC1.
//! ROADMAP 4.2, adiantado parcialmente na 0046 para destravar o 1.13.
//! Testes de banking pertencem ao 4.2; sem eles clippy enxerga dead code.
#![allow(dead_code)]

use std::fmt;
use std::ops::RangeInclusive;

use super::{Cartridge, CartridgeError, OPEN_BUS};

const ROM_WINDOW: RangeInclusive<u16> = 0x0000..=0x7FFF;
const BANK_LEN: usize = 0x4000;

pub struct Mbc1 {
    rom: Box<[u8]>,
    rom_bank: u8,
    bank_mask: u8,
}

impl Mbc1 {
    pub fn new(rom: Vec<u8>, rom_banks: usize) -> Result<Self, CartridgeError> {
        if rom.len() > rom_banks * BANK_LEN {
            return Err(CartridgeError::RomTooLarge { len: rom.len() });
        }

        let required_bits = (rom_banks - 1).ilog2() + 1;
        let bank_mask = if required_bits >= 5 {
            0x1F
        } else {
            (1u8 << required_bits) - 1
        };

        Ok(Self {
            rom: rom.into_boxed_slice(),
            rom_bank: 0x01,
            bank_mask,
        })
    }

    fn rom_addr(&self, addr: u16) -> usize {
        if !ROM_WINDOW.contains(&addr) {
            return usize::MAX;
        }

        let addr = addr as usize;
        if addr < BANK_LEN {
            addr
        } else {
            let bank = 1u8;
            bank as usize * BANK_LEN + (addr - BANK_LEN)
        }
    }

    fn effective_bank(&self) -> u8 {
        let masked = self.rom_bank & self.bank_mask;
        if masked == 0 && self.rom_bank == 0 {
            1
        } else {
            masked
        }
    }
}

impl Cartridge for Mbc1 {
    fn read(&self, addr: u16) -> u8 {
        let offset = self.rom_addr(addr);
        self.rom.get(offset).copied().unwrap_or(OPEN_BUS)
    }

    fn write(&mut self, addr: u16, value: u8) {
        if (0x2000..=0x3FFF).contains(&addr) {
            self.rom_bank = value & 0x1F;
        }
    }
}

impl fmt::Debug for Mbc1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Mbc1")
            .field("rom_len", &self.rom.len())
            .finish()
    }
}
