//! spec: `docs/reference/08-cartridges-mbc.md` § MBC1.
//! ROADMAP 4.2, adiantado parcialmente na 0046 para destravar o 1.13.
//! RAM e correção de rom_addr na 0053 para destravar as ROMs de timing (ROADMAP 2.4).

use std::fmt;
use std::ops::RangeInclusive;

use super::{Cartridge, CartridgeError, OPEN_BUS};

const ROM_WINDOW: RangeInclusive<u16> = 0x0000..=0x7FFF;
const RAM_WINDOW: RangeInclusive<u16> = 0xA000..=0xBFFF;
const BANK_LEN: usize = 0x4000;
const RAM_BANK_LEN: usize = 8 * 1024;

pub struct Mbc1 {
    rom: Box<[u8]>,
    rom_bank: u8,
    bank_mask: u8,
    ram: Box<[u8]>,
    ram_enabled: bool,
    ram_bank: u8,
    banking_mode: bool,
}

impl Mbc1 {
    pub fn new(rom: Vec<u8>, rom_banks: usize, ram_bytes: usize) -> Result<Self, CartridgeError> {
        if rom.len() > rom_banks * BANK_LEN {
            return Err(CartridgeError::RomTooLarge { len: rom.len() });
        }

        let required_bits = (rom_banks - 1).ilog2() + 1;
        let bank_mask = (1u8 << required_bits) - 1;

        let ram = if ram_bytes > 0 {
            vec![0x00; ram_bytes].into_boxed_slice()
        } else {
            Box::new([])
        };

        Ok(Self {
            rom: rom.into_boxed_slice(),
            rom_bank: 0x01,
            bank_mask,
            ram,
            ram_enabled: false,
            ram_bank: 0x00,
            banking_mode: false,
        })
    }

    fn rom_addr(&self, addr: u16) -> usize {
        if !ROM_WINDOW.contains(&addr) {
            return usize::MAX;
        }

        let addr = addr as usize;
        if addr < BANK_LEN {
            if self.banking_mode {
                let bank = ((self.ram_bank & 0x03) as usize) << 5;
                (bank & (self.bank_mask as usize)) * BANK_LEN + addr
            } else {
                addr
            }
        } else {
            let bank = self.effective_bank();
            bank as usize * BANK_LEN + (addr - BANK_LEN)
        }
    }

    fn ram_addr(&self, addr: u16) -> usize {
        let offset = (addr as usize) - (*RAM_WINDOW.start() as usize);
        let bank = if self.banking_mode {
            (self.ram_bank & 0x03) as usize
        } else {
            0
        };
        bank * RAM_BANK_LEN + offset
    }

    fn effective_bank(&self) -> u8 {
        let corrected = if self.rom_bank == 0 {
            0x01
        } else {
            self.rom_bank
        };
        let upper = (self.ram_bank & 0x03) as u16;
        let full = (upper << 5) | (corrected as u16);
        (full as u8) & self.bank_mask
    }
}

impl Cartridge for Mbc1 {
    fn read(&self, addr: u16) -> u8 {
        if ROM_WINDOW.contains(&addr) {
            let offset = self.rom_addr(addr);
            self.rom.get(offset).copied().unwrap_or(OPEN_BUS)
        } else if RAM_WINDOW.contains(&addr) {
            if self.ram_enabled && !self.ram.is_empty() {
                let offset = self.ram_addr(addr);
                self.ram.get(offset).copied().unwrap_or(OPEN_BUS)
            } else {
                OPEN_BUS
            }
        } else {
            OPEN_BUS
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => {
                self.ram_enabled = (value & 0x0F) == 0x0A;
            }
            0x2000..=0x3FFF => {
                self.rom_bank = value & 0x1F;
            }
            0x4000..=0x5FFF => {
                self.ram_bank = value & 0x03;
            }
            0x6000..=0x7FFF => {
                self.banking_mode = (value & 0x01) != 0;
            }
            _ => {
                if RAM_WINDOW.contains(&addr) && self.ram_enabled && !self.ram.is_empty() {
                    let offset = self.ram_addr(addr);
                    if let Some(slot) = self.ram.get_mut(offset) {
                        *slot = value;
                    }
                }
            }
        }
    }
}

impl fmt::Debug for Mbc1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Mbc1")
            .field("rom_len", &self.rom.len())
            .field("ram_len", &self.ram.len())
            .finish()
    }
}
