//! O cartucho: cabeçalho e, a partir do ROADMAP 0.4, os mapeadores.
//!
//! Hoje só existe o cabeçalho (0.3a). `NoMbc` chega no 0.4, MBC1 no 4.2 e o
//! resto no marco M5 — cada um em seu módulo, todos lendo
//! `docs/reference/08-cartridges-mbc.md` antes (R1).

mod header;

pub use header::{
    CartridgeHeader, CartridgeType, HeaderChecksum, HeaderError, MIN_ROM_LEN, RamSize, RomSize,
};
