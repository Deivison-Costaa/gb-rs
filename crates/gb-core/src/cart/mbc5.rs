// spec: `docs/reference/08-cartridges-mbc.md` § MBC5.
// ROADMAP 5.3 — ROM banking (9 bits), RAM banking (4 bits).

use std::fmt;
use std::ops::RangeInclusive;

use super::{Cartridge, CartridgeError, OPEN_BUS};

const ROM_WINDOW: RangeInclusive<u16> = 0x0000..=0x7FFF;
const RAM_WINDOW: RangeInclusive<u16> = 0xA000..=0xBFFF;
const BANK_LEN: usize = 0x4000;
const RAM_BANK_LEN: usize = 8 * 1024;

pub struct Mbc5 {
    rom: Box<[u8]>,
    rom_bank_low: u8,
    rom_bank_high: u8,
    ram: Box<[u8]>,
    ram_bank: u8,
    ram_enabled: bool,
    has_battery: bool,
}

impl Mbc5 {
    pub fn new(rom: Vec<u8>, rom_banks: usize, ram_bytes: usize) -> Result<Self, CartridgeError> {
        if rom.len() > rom_banks * BANK_LEN {
            return Err(CartridgeError::RomTooLarge { len: rom.len() });
        }

        let ram = if ram_bytes > 0 {
            vec![0x00; ram_bytes].into_boxed_slice()
        } else {
            Box::new([])
        };

        Ok(Self {
            rom: rom.into_boxed_slice(),
            rom_bank_low: 0x01,
            rom_bank_high: 0x00,
            ram,
            ram_bank: 0x00,
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

    // O banco ROM no MBC5 é 9 bits, sem tradução 00→01.
    // ver docs/iterations/0071-mbc5.md — erros de primeira tentativa.
    fn effective_rom_bank(&self) -> usize {
        (self.rom_bank_low as usize) | ((self.rom_bank_high as usize & 0x01) << 8)
    }

    fn rom_offset(&self, addr: u16) -> usize {
        let addr = addr as usize;
        if addr < BANK_LEN {
            addr
        } else {
            self.effective_rom_bank() * BANK_LEN + (addr - BANK_LEN)
        }
    }

    fn ram_offset(&self, addr: u16) -> usize {
        let bank = self.ram_bank as usize;
        let offset = (addr as usize) - (*RAM_WINDOW.start() as usize);
        let effective_bank = if self.ram.is_empty() {
            0
        } else {
            bank % (self.ram.len().max(1) / RAM_BANK_LEN)
        };
        effective_bank * RAM_BANK_LEN + offset % RAM_BANK_LEN
    }
}

impl Cartridge for Mbc5 {
    fn read(&self, addr: u16) -> u8 {
        if ROM_WINDOW.contains(&addr) {
            let offset = self.rom_offset(addr);
            self.rom.get(offset).copied().unwrap_or(OPEN_BUS)
        } else if RAM_WINDOW.contains(&addr) && self.ram_enabled {
            if !self.ram.is_empty() {
                let offset = self.ram_offset(addr);
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
            0x2000..=0x2FFF => {
                self.rom_bank_low = value;
            }
            0x3000..=0x3FFF => {
                self.rom_bank_high = value;
            }
            0x4000..=0x5FFF => {
                self.ram_bank = value & 0x0F;
            }
            _ => {
                if RAM_WINDOW.contains(&addr) && self.ram_enabled && !self.ram.is_empty() {
                    let offset = self.ram_offset(addr);
                    if let Some(slot) = self.ram.get_mut(offset) {
                        *slot = value;
                    }
                }
            }
        }
    }

    fn ram_data(&self) -> Option<&[u8]> {
        if self.has_battery && !self.ram.is_empty() {
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

impl fmt::Debug for Mbc5 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Mbc5")
            .field("rom_len", &self.rom.len())
            .field("ram_len", &self.ram.len())
            .finish()
    }
}
