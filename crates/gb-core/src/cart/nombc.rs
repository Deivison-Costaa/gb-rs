//! O cartucho sem mapeador — o mais simples que existe.
//!
//! Spec: `docs/reference/08-cartridges-mbc.md` § No MBC (Pan Docs
//! `fe246067b695`), inteira:
//!
//! > Small games of not more than 32 KiB ROM do not require a MBC chip for
//! > ROM banking. The ROM is directly mapped to memory at $0000-7FFF.
//! > Optionally up to 8 KiB of RAM could be connected at $A000-BFFF, using
//! > a discrete logic decoder in place of a full MBC chip.
//!
//! **A RAM opcional não está implementada, e isso é escopo, não esquecimento.**
//! Os dois tipos de cartucho que a declaram são `$08` e `$09`, e são
//! exatamente os que a tabela de `$0147` marca com *"No licensed cartridge
//! makes use of this option. The exact behavior is unknown."* — não há
//! comportamento documentado para copiar. [`super::load`] os recusa em vez de
//! entregar um cartucho com RAM de mentira.

use std::fmt;
use std::ops::RangeInclusive;

use super::{Cartridge, CartridgeError, OPEN_BUS};

/// `$0000`–`$7FFF`. São as duas janelas do mapa de memória (banco fixo e banco
/// chaveado), contíguas aqui porque não existe mapeador para separá-las: num
/// cartucho sem MBC o endereço **é** o índice na ROM.
const ROM_WINDOW: RangeInclusive<u16> = 0x0000..=0x7FFF;

/// Cartucho ROM-only: um chip de ROM ligado direto ao barramento.
pub struct NoMbc {
    rom: Box<[u8]>,
}

impl NoMbc {
    /// 32 KiB — o que `$0000`–`$7FFF` alcança, e o teto da § No MBC.
    pub const MAX_ROM_LEN: usize = 32 * 1024;

    /// Monta o cartucho a partir dos bytes da ROM.
    ///
    /// Não olha o cabeçalho: quem lê `$0147` e escolhe o mapeador é
    /// [`super::load`]. Aqui só importa se a ROM cabe no barramento.
    ///
    /// # Errors
    ///
    /// [`CartridgeError::RomTooLarge`] se a ROM passar de [`Self::MAX_ROM_LEN`].
    /// Truncar em silêncio seria pior do que recusar: uma ROM de 64 KiB com
    /// `$0147` = `$00` é cartucho mal-identificado, e o sintoma — o jogo
    /// travando ao saltar para um banco que não existe — não apontaria para cá.
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
    /// O endereço é o índice na ROM, sem tradução nenhuma.
    ///
    /// Acima do fim de uma ROM menor que a janela, [`OPEN_BUS`]: a spec
    /// descreve o cartucho de 32 KiB e **não diz** o que responde um chip
    /// menor. Espelhar o começo (`addr % rom.len()`, que é o que meio mundo
    /// faz) seria inventar como o chip foi fiado.
    fn read(&self, addr: u16) -> u8 {
        if !ROM_WINDOW.contains(&addr) {
            return OPEN_BUS;
        }

        self.rom.get(addr as usize).copied().unwrap_or(OPEN_BUS)
    }

    /// Não faz nada, e é a implementação correta: sem MBC não há registrador
    /// escutando `$0000`–`$7FFF`, e sem RAM não há onde guardar `$A000`–`$BFFF`.
    fn write(&mut self, _addr: u16, _value: u8) {}
}

/// `derive(Debug)` despejaria 32 KiB no terminal de quem depurar o `Bus`. O que
/// interessa do cartucho é o tamanho.
impl fmt::Debug for NoMbc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NoMbc")
            .field("rom_len", &self.rom.len())
            .finish()
    }
}
