//! spec: `docs/reference/08-cartridges-mbc.md` § MBC3.
//! ROADMAP 5.2a–5.2b — ROM/RAM banking + RTC.

use std::fmt;
use std::ops::RangeInclusive;

use super::{Cartridge, CartridgeError, OPEN_BUS};

const ROM_WINDOW: RangeInclusive<u16> = 0x0000..=0x7FFF;
const RAM_WINDOW: RangeInclusive<u16> = 0xA000..=0xBFFF;
const BANK_LEN: usize = 0x4000;
const RAM_BANK_LEN: usize = 8 * 1024;

pub struct Mbc3 {
    rom: Box<[u8]>,
    rom_bank: u8,
    ram: Box<[u8]>,
    ram_enabled: bool,
    ram_rtc_select: u8,
    has_battery: bool,
    has_rtc: bool,
    rtc_registers: [u8; 5],
    latch_previous: u8,
}

impl Mbc3 {
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
            rom_bank: 0x01,
            ram,
            ram_enabled: false,
            ram_rtc_select: 0x00,
            has_battery: false,
            has_rtc: false,
            rtc_registers: [0; 5],
            latch_previous: 0xFF,
        })
    }

    #[must_use]
    pub fn with_battery(self) -> Self {
        Self {
            has_battery: true,
            ..self
        }
    }

    #[must_use]
    pub fn with_rtc(self) -> Self {
        Self {
            has_rtc: true,
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
        let bank = (self.ram_rtc_select & 0x07) as usize;
        let offset = (addr as usize) - (*RAM_WINDOW.start() as usize);
        let effective_bank = if self.ram.is_empty() {
            0
        } else {
            bank % (self.ram.len().max(1) / RAM_BANK_LEN)
        };
        effective_bank * RAM_BANK_LEN + offset % RAM_BANK_LEN
    }

    fn rtc_select_active(&self) -> bool {
        self.has_rtc && (0x08..=0x0C).contains(&self.ram_rtc_select)
    }
}

impl Cartridge for Mbc3 {
    fn read(&self, addr: u16) -> u8 {
        if ROM_WINDOW.contains(&addr) {
            let offset = self.rom_offset(addr);
            self.rom.get(offset).copied().unwrap_or(OPEN_BUS)
        } else if RAM_WINDOW.contains(&addr) && self.ram_enabled {
            if self.rtc_select_active() {
                self.rtc_registers[(self.ram_rtc_select - 8) as usize]
            } else if !self.ram.is_empty() {
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
            0x2000..=0x3FFF => {
                self.rom_bank = value & 0x7F;
            }
            0x4000..=0x5FFF => {
                self.ram_rtc_select = value & 0x0F;
            }
            0x6000..=0x7FFF => {
                self.latch_previous = value;
            }
            _ => {
                if RAM_WINDOW.contains(&addr) && self.ram_enabled {
                    if self.rtc_select_active() {
                        self.rtc_registers[(self.ram_rtc_select - 8) as usize] = value;
                    } else if !self.ram.is_empty() {
                        let offset = self.ram_offset(addr);
                        if let Some(slot) = self.ram.get_mut(offset) {
                            *slot = value;
                        }
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

impl fmt::Debug for Mbc3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Mbc3")
            .field("rom_len", &self.rom.len())
            .field("ram_len", &self.ram.len())
            .finish()
    }
}
