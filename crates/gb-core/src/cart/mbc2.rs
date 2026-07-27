//! spec: `docs/reference/08-cartridges-mbc.md` § MBC2.
//! ROADMAP 5.1.

use std::fmt;
use std::ops::RangeInclusive;

use super::{Cartridge, CartridgeError, OPEN_BUS};

const ROM_WINDOW: RangeInclusive<u16> = 0x0000..=0x7FFF;
const RAM_WINDOW: RangeInclusive<u16> = 0xA000..=0xBFFF;
const BANK_LEN: usize = 0x4000;

const RAM_LEN: usize = 512;

pub struct Mbc2 {
    rom: Box<[u8]>,
    rom_bank: u8,
    ram: Box<[u8]>,
    ram_enabled: bool,
    has_battery: bool,
}

impl Mbc2 {
    pub fn new(rom: Vec<u8>, rom_banks: usize) -> Result<Self, CartridgeError> {
        if rom.len() > rom_banks * BANK_LEN {
            return Err(CartridgeError::RomTooLarge { len: rom.len() });
        }

        Ok(Self {
            rom: rom.into_boxed_slice(),
            rom_bank: 0x01,
            ram: vec![0x00; RAM_LEN].into_boxed_slice(),
            ram_enabled: false,
            has_battery: false,
        })
    }

    #[must_use]
    pub fn with_battery(self) -> Self {
        Self {
            has_battery: true,
            ..self
        }
    }

    fn rom_offset(&self, addr: u16) -> usize {
        let addr = addr as usize;
        if addr < BANK_LEN {
            addr
        } else {
            let bank = if self.rom_bank == 0 {
                0x01
            } else {
                self.rom_bank
            };
            bank as usize * BANK_LEN + (addr - BANK_LEN)
        }
    }

    fn ram_offset(&self, addr: u16) -> usize {
        (addr as usize - *RAM_WINDOW.start() as usize) & (RAM_LEN - 1)
    }
}

impl Cartridge for Mbc2 {
    fn read(&self, addr: u16) -> u8 {
        if ROM_WINDOW.contains(&addr) {
            let offset = self.rom_offset(addr);
            self.rom.get(offset).copied().unwrap_or(OPEN_BUS)
        } else if RAM_WINDOW.contains(&addr) && self.ram_enabled {
            let offset = self.ram_offset(addr);
            self.ram.get(offset).copied().unwrap_or(OPEN_BUS)
        } else {
            OPEN_BUS
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        if ROM_WINDOW.contains(&addr) {
            if addr & 0x0100 == 0 {
                self.ram_enabled = (value & 0x0F) == 0x0A;
            } else {
                self.rom_bank = value & 0x0F;
            }
        } else if RAM_WINDOW.contains(&addr) && self.ram_enabled {
            let offset = self.ram_offset(addr);
            if let Some(slot) = self.ram.get_mut(offset) {
                *slot = value & 0x0F;
            }
        }
    }

    fn ram_data(&self) -> Option<&[u8]> {
        if self.has_battery {
            Some(&self.ram)
        } else {
            None
        }
    }

    fn load_ram(&mut self, data: &[u8]) {
        let len = data.len().min(self.ram.len());
        self.ram[..len].copy_from_slice(&data[..len]);
    }
}

impl fmt::Debug for Mbc2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Mbc2")
            .field("rom_len", &self.rom.len())
            .field("ram_len", &self.ram.len())
            .finish()
    }
}
