//! O cabeçalho do cartucho, em `$0100`–`$014F`.
//!
//! Spec: `docs/reference/08-cartridges-mbc.md` § The Cartridge Header
//! (Pan Docs `fe246067b695`). **Leia antes de mexer** — a R1 existe porque
//! duas tabelas daqui não seguem fórmula nenhuma e a memória as "corrige"
//! sozinha (ver `RamSize::bytes`).
//!
//! Escopo do ROADMAP 0.3a: título, tipo de cartucho, tamanho de ROM/RAM e o
//! checksum do cabeçalho. O que ficou de fora, de propósito:
//!
//! - **Checksum global (`$014E`–`$014F`).** Precisa da ROM inteira e nenhum
//!   hardware o verifica — só o emulador de GB Tower do Pokémon Stadium.
//! - **CGB flag, SGB flag, licenciante, região, versão.** Este emulador é DMG
//!   e o 0.3b não vai imprimi-los. Quando algum item pedir, o campo entra aqui.
//! - **Despacho de mapeador.** É o 0.4. Aqui o tipo é um código com nome.

use std::fmt;
use std::ops::{Range, RangeInclusive};

/// `$0134`–`$0143`: o título, em ASCII maiúsculo, preenchido com `$00`.
const TITLE: Range<usize> = 0x0134..0x0144;
/// `$0147`: que hardware o cartucho tem — antes de tudo, qual mapeador.
const CARTRIDGE_TYPE: usize = 0x0147;
/// `$0148`: tamanho da ROM.
const ROM_SIZE: usize = 0x0148;
/// `$0149`: tamanho da RAM do cartucho, se houver.
const RAM_SIZE: usize = 0x0149;
/// `$0134`–`$014C`: exatamente o que entra na soma do checksum. Nem o logo
/// (`$0104`–`$0133`), nem o próprio `$014D`, nem o checksum global.
const CHECKSUMMED: RangeInclusive<usize> = 0x0134..=0x014C;
/// `$014D`: o checksum que o boot ROM confere antes de deixar o jogo rodar.
const CHECKSUM: usize = 0x014D;

/// Menor ROM que contém o cabeçalho inteiro — `$014F` é o último byte dele.
pub const MIN_ROM_LEN: usize = 0x0150;

/// Um banco de ROM. O cabeçalho conta bancos; o mapa de memória os endereça em
/// `$0000`–`$3FFF` e `$4000`–`$7FFF`.
const ROM_BANK_SIZE: u32 = 16 * 1024;

/// O que impede uma fatia de bytes de ser um cabeçalho.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderError {
    /// A ROM acaba antes de `$014F`. `len` é o tamanho que ela tem.
    TooShort {
        /// Tamanho da fatia recebida, em bytes.
        len: usize,
    },
}

impl fmt::Display for HeaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooShort { len } => write!(
                f,
                "ROM tem {len} bytes e o cabeçalho vai até $014F: \
                 são necessários pelo menos {MIN_ROM_LEN}"
            ),
        }
    }
}

impl std::error::Error for HeaderError {}

/// `$0147` — o tipo do cartucho.
///
/// Guarda o código cru e sabe nomeá-lo quando ele está na tabela do Pan Docs.
/// Código fora da tabela **não** é erro de parse: um `info` que se recusa a
/// falar sobre a ROM esquisita é exatamente o que não serve para diagnosticar
/// ROM esquisita.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CartridgeType(u8);

impl CartridgeType {
    /// O byte de `$0147`, como está na ROM.
    #[must_use]
    pub const fn code(self) -> u8 {
        self.0
    }

    /// O nome da tabela, ou `None` se o código não estiver nela.
    #[must_use]
    pub const fn name(self) -> Option<&'static str> {
        Some(match self.0 {
            0x00 => "ROM ONLY",
            0x01 => "MBC1",
            0x02 => "MBC1+RAM",
            0x03 => "MBC1+RAM+BATTERY",
            0x05 => "MBC2",
            0x06 => "MBC2+BATTERY",
            0x08 => "ROM+RAM",
            0x09 => "ROM+RAM+BATTERY",
            0x0B => "MMM01",
            0x0C => "MMM01+RAM",
            0x0D => "MMM01+RAM+BATTERY",
            0x0F => "MBC3+TIMER+BATTERY",
            0x10 => "MBC3+TIMER+RAM+BATTERY",
            0x11 => "MBC3",
            0x12 => "MBC3+RAM",
            0x13 => "MBC3+RAM+BATTERY",
            0x19 => "MBC5",
            0x1A => "MBC5+RAM",
            0x1B => "MBC5+RAM+BATTERY",
            0x1C => "MBC5+RUMBLE",
            0x1D => "MBC5+RUMBLE+RAM",
            0x1E => "MBC5+RUMBLE+RAM+BATTERY",
            0x20 => "MBC6",
            0x22 => "MBC7+SENSOR+RUMBLE+RAM+BATTERY",
            0xFC => "POCKET CAMERA",
            0xFD => "BANDAI TAMA5",
            0xFE => "HuC3",
            0xFF => "HuC1+RAM+BATTERY",
            _ => return None,
        })
    }
}

/// `$0148` — o tamanho da ROM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RomSize(u8);

impl RomSize {
    /// O byte de `$0148`, como está na ROM.
    #[must_use]
    pub const fn code(self) -> u8 {
        self.0
    }

    /// `32 KiB × (1 << code)`, ou `None` para código fora da tabela.
    ///
    /// `$52`, `$53` e `$54` (1.1, 1.2 e 1.5 MiB) ficam de fora **de propósito**:
    /// a spec diz que aparecem só em documentação não oficial, que nenhum
    /// cartucho ou arquivo conhecido os usa e que, sendo todos os outros
    /// potências de 2, provavelmente estão errados. Mapeá-los seria inventar um
    /// número com cara de medição no campo que o 0.3b imprime.
    #[must_use]
    pub const fn bytes(self) -> Option<u32> {
        match self.0 {
            code @ 0x00..=0x08 => Some((32 * 1024) << code),
            _ => None,
        }
    }

    /// Quantos bancos de 16 KiB o tamanho dá. `None` pelo mesmo motivo acima.
    #[must_use]
    pub const fn banks(self) -> Option<u16> {
        match self.bytes() {
            // O maior caso é 8 MiB = 512 bancos, que cabe folgado em u16.
            Some(bytes) => Some((bytes / ROM_BANK_SIZE) as u16),
            None => None,
        }
    }
}

/// `$0149` — o tamanho da RAM do cartucho.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RamSize(u8);

impl RamSize {
    /// O byte de `$0149`, como está na ROM.
    #[must_use]
    pub const fn code(self) -> u8 {
        self.0
    }

    /// O tamanho da tabela, ou `None` para código sem tamanho conhecido.
    ///
    /// **Isto não é uma fórmula e não tente transformar em uma.** `$04` são
    /// 128 KiB e `$05` são 64 KiB — a tabela não é monotônica. E `$01`, que
    /// meio mundo documenta como 2 KiB, é `None`: o Pan Docs marca "Unused",
    /// registra que chip de 2 KiB nunca foi usado em cartucho e que a origem
    /// do valor é desconhecida. ROMs homebrew antigas gravam `$01` por engano
    /// e em geral nem usam RAM.
    #[must_use]
    pub const fn bytes(self) -> Option<u32> {
        match self.0 {
            0x00 => Some(0),
            0x02 => Some(8 * 1024),
            0x03 => Some(32 * 1024),
            0x04 => Some(128 * 1024),
            0x05 => Some(64 * 1024),
            _ => None,
        }
    }
}

/// `$014D` — o checksum do cabeçalho, dos dois lados.
///
/// O boot ROM **trava a máquina** quando os dois não batem, então guardar só o
/// veredito perderia a informação de que o cartucho está corrompido e como.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeaderChecksum {
    stored: u8,
    computed: u8,
}

impl HeaderChecksum {
    /// O byte que está gravado em `$014D`.
    #[must_use]
    pub const fn stored(self) -> u8 {
        self.stored
    }

    /// O byte que o boot ROM calcularia a partir de `$0134`–`$014C`.
    #[must_use]
    pub const fn computed(self) -> u8 {
        self.computed
    }

    /// `true` quando o boot ROM deixaria o cartucho rodar.
    #[must_use]
    pub const fn is_valid(self) -> bool {
        self.stored == self.computed
    }
}

/// O cabeçalho do cartucho, já interpretado.
///
/// Construído por [`CartridgeHeader::parse`] a partir dos bytes da ROM; os
/// campos não mudam depois disso.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CartridgeHeader {
    title: String,
    cartridge_type: CartridgeType,
    rom_size: RomSize,
    ram_size: RamSize,
    checksum: HeaderChecksum,
}

impl CartridgeHeader {
    /// Lê o cabeçalho dos bytes de uma ROM.
    ///
    /// Só falha quando a fatia acaba antes de `$014F`. Campo com código
    /// desconhecido não é erro: o código cru fica acessível e o significado
    /// vira `None`.
    ///
    /// # Errors
    ///
    /// [`HeaderError::TooShort`] se `rom` tiver menos de [`MIN_ROM_LEN`] bytes.
    pub fn parse(rom: &[u8]) -> Result<Self, HeaderError> {
        if rom.len() < MIN_ROM_LEN {
            return Err(HeaderError::TooShort { len: rom.len() });
        }

        Ok(Self {
            title: parse_title(&rom[TITLE]),
            cartridge_type: CartridgeType(rom[CARTRIDGE_TYPE]),
            rom_size: RomSize(rom[ROM_SIZE]),
            ram_size: RamSize(rom[RAM_SIZE]),
            checksum: HeaderChecksum {
                stored: rom[CHECKSUM],
                computed: compute_checksum(rom),
            },
        })
    }

    /// O título, sem o preenchimento e sem os campos que invadiram o fim dele.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// `$0147` — o tipo do cartucho.
    #[must_use]
    pub const fn cartridge_type(&self) -> CartridgeType {
        self.cartridge_type
    }

    /// `$0148` — o tamanho da ROM.
    #[must_use]
    pub const fn rom_size(&self) -> RomSize {
        self.rom_size
    }

    /// `$0149` — o tamanho da RAM do cartucho.
    #[must_use]
    pub const fn ram_size(&self) -> RamSize {
        self.ram_size
    }

    /// `$014D` — o checksum do cabeçalho, armazenado e calculado.
    #[must_use]
    pub const fn checksum(&self) -> HeaderChecksum {
        self.checksum
    }
}

/// Extrai o título dos 16 bytes de `$0134`–`$0143`.
///
/// A spec manda preencher o que sobra com `$00`, mas o fim do campo foi sendo
/// tomado por outros usos: `$013F`–`$0142` viraram código do fabricante e
/// `$0143` virou o CGB flag (`$80`/`$C0`). Nada no cabeçalho diz se o cartucho
/// é antigo ou novo, então a regra aqui é o que dá para decidir olhando os
/// bytes: vale o trecho **inicial** de ASCII imprimível, e o resto se ignora.
/// Sem isso um título de 15 caracteres arrastaria o `$80` do CGB flag junto.
fn parse_title(field: &[u8]) -> String {
    let printable: String = field
        .iter()
        .copied()
        .take_while(|byte| byte.is_ascii_graphic() || *byte == b' ')
        .map(char::from)
        .collect();

    printable.trim_end().to_string()
}

/// O checksum de `$0134`–`$014C`, como o boot ROM o calcula:
///
/// ```c
/// uint8_t checksum = 0;
/// for (uint16_t address = 0x0134; address <= 0x014C; address++) {
///     checksum = checksum - rom[address] - 1;
/// }
/// ```
///
/// O `- 1` por byte é o que distingue isto de uma soma comum: sem ele, um
/// cabeçalho todo zerado — que é o caso mais provável de ROM truncada ou lixo —
/// bateria com o `$00` gravado em `$014D`.
fn compute_checksum(rom: &[u8]) -> u8 {
    rom[CHECKSUMMED]
        .iter()
        .fold(0u8, |acc, &byte| acc.wrapping_sub(byte).wrapping_sub(1))
}
