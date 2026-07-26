//! O cartucho: o cabeçalho, os mapeadores e o despacho entre eles.
//!
//! Spec: `docs/reference/08-cartridges-mbc.md` (Pan Docs `fe246067b695`).
//!
//! O `Bus` (ROADMAP 1.2) vai rotear para cá duas janelas do mapa de memória, e
//! **só** elas:
//!
//! | Janela | Conteúdo |
//! |---|---|
//! | `$0000`–`$7FFF` | ROM do cartucho — banco fixo e banco chaveado pelo mapeador, se houver |
//! | `$A000`–`$BFFF` | RAM externa, se o cartucho tiver |
//!
//! Hoje o único mapeador é o [`NoMbc`] (0.4). MBC1 chega no 4.2 e MBC2/3/5 no
//! marco M5 — cada um em seu módulo, todos lendo a spec antes (R1).

mod header;
mod nombc;

pub use header::{
    CartridgeHeader, CartridgeType, HeaderChecksum, HeaderError, MIN_ROM_LEN, RamSize, RomSize,
};
pub use nombc::NoMbc;

use std::fmt;

/// O que o barramento entrega quando ninguém do outro lado responde.
///
/// O Pan Docs descreve o valor em § MBC1, `$A000`–`$BFFF`: *"reads return open
/// bus values (often $FF, but not guaranteed)"*. O "not guaranteed" é o ponto —
/// isto é o valor típico de uma linha solta, não um número que algum chip
/// produz. Nenhum jogo pode depender dele, e nenhum teste do projeto deve
/// afirmar mais do que "não veio dado de lugar nenhum".
pub const OPEN_BUS: u8 = 0xFF;

/// `$0147` = `$00`, ROM ONLY — o único cartucho que o 0.4 sabe montar.
const ROM_ONLY: u8 = 0x00;

/// O cartucho visto pelo barramento: dois endereços e mais nada.
///
/// A abstração inteira do mapeador cabe aqui porque é literalmente isto que o
/// hardware expõe — o MBC fica entre o barramento e os chips de memória, e o
/// Game Boy não tem como perguntar nada a ele além de ler e escrever endereço.
/// Banco selecionado, RAM habilitada e modo de banking são estado **interno**
/// do mapeador, escrito pelas mesmas escritas em `$0000`–`$7FFF` que num
/// cartucho sem MBC não fazem nada.
///
/// Por isso [`write`](Cartridge::write) existe mesmo em ROM ONLY, onde é
/// no-op: quem chama é o `Bus`, que não sabe (nem deve saber) qual mapeador
/// está do outro lado.
pub trait Cartridge {
    /// Lê um byte. Fora das janelas do cartucho, [`OPEN_BUS`].
    fn read(&self, addr: u16) -> u8;

    /// Escreve um byte — na RAM externa, ou num registrador do mapeador.
    ///
    /// Escrita em ROM não altera ROM em hardware nenhum: quando pega em alguma
    /// coisa, é porque o mapeador está ouvindo aquele endereço.
    fn write(&mut self, addr: u16, value: u8);
}

/// O que impede uma ROM de virar cartucho.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CartridgeError {
    /// O cabeçalho não pôde ser lido — sem ele não há `$0147` para despachar.
    Header(HeaderError),
    /// `$0147` pede um mapeador que este emulador ainda não tem, ou nomeia
    /// hardware que o Pan Docs não descreve.
    UnsupportedType {
        /// O tipo lido de `$0147`, com o código cru e o nome, se houver.
        cartridge_type: CartridgeType,
    },
    /// A ROM não cabe no mapeador escolhido.
    RomTooLarge {
        /// Tamanho da ROM recebida, em bytes.
        len: usize,
    },
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
                "ROM de {len} bytes num cartucho sem MBC: $0000-$7FFF endereça \
                 {max} bytes e não há mapeador para alcançar o resto",
                max = NoMbc::MAX_ROM_LEN
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

/// Monta o cartucho que o `$0147` da ROM descreve.
///
/// Recebe os bytes já em memória: quem toca em arquivo é `gb-cli` ou
/// `gb-desktop` (R3). A `Vec` é consumida porque o cartucho passa a ser dono
/// dela — a ROM vive enquanto a máquina viver.
///
/// **O cabeçalho não é julgado aqui.** Checksum errado, título ilegível ou
/// tamanho declarado que não bate com o arquivo não impedem a montagem: quem
/// trava a máquina por checksum é o boot ROM, que este emulador pula. O
/// diagnóstico do cabeçalho é trabalho do `gb-cli info` (0.3b), e a ROM que
/// trava o boot ROM é justamente a que alguém quer investigar.
///
/// # Errors
///
/// - [`CartridgeError::Header`] se a ROM acabar antes de `$014F`.
/// - [`CartridgeError::UnsupportedType`] para qualquer `$0147` != `$00`.
///   Inclui `$08`/`$09` (ROM+RAM), que **são** cartucho sem MBC: eles trazem a
///   RAM opcional da § No MBC, e o Pan Docs anota na tabela de `$0147` que
///   nenhum cartucho licenciado os usa e que *"the exact behavior is unknown"*.
///   Montá-los como ROM ONLY entregaria um cartucho cuja RAM lê [`OPEN_BUS`] e
///   engole escrita — o jogo perderia o save sem um erro sequer.
/// - [`CartridgeError::RomTooLarge`] se a ROM não couber no mapeador.
pub fn load(rom: Vec<u8>) -> Result<Box<dyn Cartridge>, CartridgeError> {
    let cartridge_type = CartridgeHeader::parse(&rom)?.cartridge_type();

    match cartridge_type.code() {
        ROM_ONLY => Ok(Box::new(NoMbc::new(rom)?)),
        _ => Err(CartridgeError::UnsupportedType { cartridge_type }),
    }
}
