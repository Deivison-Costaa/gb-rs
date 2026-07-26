//! spec: `docs/reference/01-memory-map.md`. Bus é struct, não trait — vtable
//! no caminho mais quente do emulador não compra nada (ver docs/iterations/0010).

mod boot;

use crate::cart::{Cartridge, OPEN_BUS};

const WRAM_LEN: usize = 8 * 1024;

// Só os 13 bits baixos do endereço chegam à WRAM (§ Echo RAM descrevendo a fiação).
const WRAM_ADDRESS_MASK: usize = 0x1FFF;

// $FF80–$FFFE: 127 bytes, não 128. $FFFF é o IE.
const HRAM_LEN: usize = 0x7F;

const HRAM_BASE: usize = 0xFF80;

// $FEA0–$FEFF: $00 no DMG sem bloqueio de OAM (sem PPU até M3). $FF só com OAM bloqueada.
const NOT_USABLE_READ: u8 = 0x00;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Region {
    CartridgeRom,
    VideoRam,
    ExternalRam,
    WorkRam,
    EchoRam,
    ObjectAttributeMemory,
    NotUsable,
    IoRegisters,
    HighRam,
    InterruptEnable,
}

impl Region {
    #[must_use]
    pub const fn of(addr: u16) -> Self {
        match addr {
            0x0000..=0x7FFF => Self::CartridgeRom,
            0x8000..=0x9FFF => Self::VideoRam,
            0xA000..=0xBFFF => Self::ExternalRam,
            0xC000..=0xDFFF => Self::WorkRam,
            0xE000..=0xFDFF => Self::EchoRam,
            0xFE00..=0xFE9F => Self::ObjectAttributeMemory,
            0xFEA0..=0xFEFF => Self::NotUsable,
            0xFF00..=0xFF7F => Self::IoRegisters,
            0xFF80..=0xFFFE => Self::HighRam,
            0xFFFF => Self::InterruptEnable,
        }
    }
}

pub struct Bus {
    cartridge: Box<dyn Cartridge>,
    wram: [u8; WRAM_LEN],
    hram: [u8; HRAM_LEN],
    io: [u8; boot::IO_LEN],
    ie: u8,
}

impl Bus {
    #[must_use]
    pub fn new(cartridge: Box<dyn Cartridge>) -> Self {
        Self {
            cartridge,
            wram: [0x00; WRAM_LEN],
            hram: [0x00; HRAM_LEN],
            io: boot::IO,
            ie: boot::INTERRUPT_ENABLE,
        }
    }

    #[must_use]
    pub fn read(&self, addr: u16) -> u8 {
        match Region::of(addr) {
            Region::CartridgeRom | Region::ExternalRam => self.cartridge.read(addr),
            Region::WorkRam | Region::EchoRam => self.wram[wram_index(addr)],
            Region::HighRam => self.hram[hram_index(addr)],
            Region::NotUsable => NOT_USABLE_READ,
            Region::IoRegisters => {
                let index = io_index(addr);
                if boot::IO_HAS_OWNER[index] {
                    self.io[index]
                } else {
                    OPEN_BUS
                }
            }
            Region::InterruptEnable => self.ie,
            Region::VideoRam | Region::ObjectAttributeMemory => OPEN_BUS,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match Region::of(addr) {
            Region::CartridgeRom | Region::ExternalRam => self.cartridge.write(addr, value),
            Region::WorkRam | Region::EchoRam => self.wram[wram_index(addr)] = value,
            Region::HighRam => self.hram[hram_index(addr)] = value,
            Region::IoRegisters => {
                let index = io_index(addr);
                if boot::IO_HAS_OWNER[index] {
                    self.io[index] = value;
                }
            }
            Region::InterruptEnable => self.ie = value,
            Region::NotUsable | Region::VideoRam | Region::ObjectAttributeMemory => {}
        }
    }
}

const fn wram_index(addr: u16) -> usize {
    (addr as usize) & WRAM_ADDRESS_MASK
}

const fn hram_index(addr: u16) -> usize {
    (addr as usize) - HRAM_BASE
}

const fn io_index(addr: u16) -> usize {
    (addr as usize) - boot::IO_BASE
}

impl std::fmt::Debug for Bus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bus")
            .field("wram_len", &self.wram.len())
            .field("hram_len", &self.hram.len())
            .finish_non_exhaustive()
    }
}
