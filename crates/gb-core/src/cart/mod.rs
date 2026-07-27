//! Cartucho: cabeçalho, mapeadores e despacho. spec: `docs/reference/08-cartridges-mbc.md`.

mod header;
mod mbc1;
mod mbc2;
mod mbc3;
mod nombc;

pub use header::{
    CartridgeHeader, CartridgeType, HeaderChecksum, HeaderError, MIN_ROM_LEN, RamSize, RomSize,
};
pub use mbc1::Mbc1;
pub use mbc2::Mbc2;
pub use mbc3::Mbc3;
pub use nombc::NoMbc;

use std::fmt;

// Open bus: "often $FF, but not guaranteed" (§ MBC1). Valor típico de linha solta.
pub const OPEN_BUS: u8 = 0xFF;

const ROM_ONLY: u8 = 0x00;
const MBC1: u8 = 0x01;
const MBC1_RAM: u8 = 0x02;
const MBC1_RAM_BATTERY: u8 = 0x03;
const MBC2: u8 = 0x05;
const MBC2_BATTERY: u8 = 0x06;
const MBC3_TIMER_BATTERY: u8 = 0x0F;
const MBC3_TIMER_RAM_BATTERY: u8 = 0x10;
const MBC3: u8 = 0x11;
const MBC3_RAM: u8 = 0x12;
const MBC3_RAM_BATTERY: u8 = 0x13;

// O cartucho visto pelo barramento: dois endereços, read e write.
// write existe mesmo em ROM ONLY (no-op) — quem chama é o Bus, que não sabe o MBC.
pub trait Cartridge {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);

    fn ram_data(&self) -> Option<&[u8]> {
        None
    }

    fn load_ram(&mut self, _data: &[u8]) {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CartridgeError {
    Header(HeaderError),
    UnsupportedType { cartridge_type: CartridgeType },
    RomTooLarge { len: usize },
}

impl fmt::Display for CartridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Header(error) => write!(f, "{error}"),
            Self::UnsupportedType { cartridge_type } => {
                let code = cartridge_type.code();
                match cartridge_type.name() {
                    Some(name) => write!(
                        f,
                        "cartucho tipo ${code:02X} ({name}) ainda não tem mapeador \
                         implementado: por ora só ROM ONLY ($00)"
                    ),
                    None => write!(
                        f,
                        "cartucho tipo ${code:02X}, que não está na tabela do Pan Docs: \
                         sem saber que hardware é, não há como emulá-lo"
                    ),
                }
            }
            Self::RomTooLarge { len } => write!(
                f,
                "ROM de {len} bytes excede a capacidade declarada no cabeçalho"
            ),
        }
    }
}

impl std::error::Error for CartridgeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Header(error) => Some(error),
            Self::UnsupportedType { .. } | Self::RomTooLarge { .. } => None,
        }
    }
}

impl From<HeaderError> for CartridgeError {
    fn from(error: HeaderError) -> Self {
        Self::Header(error)
    }
}

// Monta o cartucho que $0147 descreve. Não julga o cabeçalho (checksum errado
// monta normalmente — quem trava por checksum é a boot ROM, que este emulador pula).
// $08/$09 são recusados: RAM opcional sem comportamento documentado (ver docs/iterations/0007).
pub fn load(rom: Vec<u8>) -> Result<Box<dyn Cartridge>, CartridgeError> {
    let header = CartridgeHeader::parse(&rom)?;
    let cartridge_type = header.cartridge_type();
    let rom_banks = rom_bank_count(header.rom_size(), rom.len());

    let ram_bytes = mbc_ram_bytes(cartridge_type.code(), header.ram_size());

    match cartridge_type.code() {
        ROM_ONLY => Ok(Box::new(NoMbc::new(rom)?)),
        MBC1 | MBC1_RAM | MBC1_RAM_BATTERY => {
            let mut mbc = Mbc1::new(rom, rom_banks, ram_bytes)?;
            if cartridge_type.code() == MBC1_RAM_BATTERY {
                mbc = mbc.with_battery();
            }
            Ok(Box::new(mbc))
        }
        MBC2 | MBC2_BATTERY => {
            let mut mbc = Mbc2::new(rom, rom_banks)?;
            if cartridge_type.code() == MBC2_BATTERY {
                mbc = mbc.with_battery();
            }
            Ok(Box::new(mbc))
        }
        MBC3 | MBC3_RAM | MBC3_RAM_BATTERY | MBC3_TIMER_BATTERY | MBC3_TIMER_RAM_BATTERY => {
            let mut mbc = Mbc3::new(rom, rom_banks, ram_bytes)?;
            if cartridge_type.code() == MBC3_RAM_BATTERY
                || cartridge_type.code() == MBC3_TIMER_BATTERY
                || cartridge_type.code() == MBC3_TIMER_RAM_BATTERY
            {
                mbc = mbc.with_battery();
            }
            if cartridge_type.code() == MBC3_TIMER_BATTERY
                || cartridge_type.code() == MBC3_TIMER_RAM_BATTERY
            {
                mbc = mbc.with_rtc();
            }
            Ok(Box::new(mbc))
        }
        _ => Err(CartridgeError::UnsupportedType { cartridge_type }),
    }
}

const RAM_BANK_LEN: u32 = 8 * 1024;

fn mbc_ram_bytes(cart_type: u8, ram_size: RamSize) -> usize {
    if cart_type != MBC1_RAM
        && cart_type != MBC1_RAM_BATTERY
        && cart_type != MBC3_RAM
        && cart_type != MBC3_RAM_BATTERY
        && cart_type != MBC3_TIMER_RAM_BATTERY
    {
        return 0;
    }
    match ram_size.code() {
        0x00 => RAM_BANK_LEN as usize,
        _ => ram_size.bytes().unwrap_or(0) as usize,
    }
}

fn rom_bank_count(rom_size: RomSize, rom_len: usize) -> usize {
    rom_size.banks().map(|b| b as usize).unwrap_or_else(|| {
        let banks = rom_len / 0x4000;
        if banks == 0 { 2 } else { banks }
    })
}
