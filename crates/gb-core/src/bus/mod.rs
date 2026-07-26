//! O barramento: quem responde a cada um dos 65536 endereços (ROADMAP 1.2a).
//!
//! Spec: `docs/reference/01-memory-map.md` § Memory Map, § Echo RAM e
//! § FEA0–FEFF range (Pan Docs `fe246067b695`).
//!
//! O [`Bus`] é o dono do estado da máquina (`CLAUDE.md` § Arquitetura): a RAM
//! interna mora aqui, o cartucho é dele, e a PPU, o timer e a APU chegam aqui
//! nos marcos seguintes. Por isso ele é `struct` e não `trait` — o item do
//! ROADMAP dizia "trait", mas um trait com um único implementador poria
//! despacho dinâmico no caminho mais quente que existe num emulador sem
//! comprar nada em troca. Se o 1.3 quiser uma memória plana para testar
//! opcodes sem cartucho, extrair a interface então é mudança local.
//!
//! **[`Bus::read`] e [`Bus::write`] não avançam o tempo.** A R2 (CPU
//! cycle-stepped) vive no laço do 1.3, que os chamará uma vez por M-cycle;
//! escondê-la aqui dentro faria cada acesso tiquetaquear por conta própria e
//! tornaria impossível posicionar o acesso *dentro* da instrução.

mod boot;

use crate::cart::{Cartridge, OPEN_BUS};

/// Os 8 KiB de Work RAM. No DMG são contíguos: o banco chaveado de
/// `$D000`–`$DFFF` é do CGB, e este emulador não é CGB.
const WRAM_LEN: usize = 8 * 1024;

/// Só os 13 bits baixos do endereço chegam à WRAM.
///
/// Isto não é um truque de implementação, é a § Echo RAM descrevendo a fiação:
///
/// > The range E000-FDFF is mapped to WRAM, but only the lower 13 bits of the
/// > address lines are connected […] This causes the address to effectively
/// > wrap around.
///
/// A mesma máscara serve `$C000`–`$DFFF` e `$E000`–`$FDFF`, e é por isso que
/// não existe "cópia" a sincronizar: é a mesma célula, alcançada por dois
/// endereços.
const WRAM_ADDRESS_MASK: usize = 0x1FFF;

/// `$FF80`–`$FFFE` — **127** bytes, não 128. `$FFFF` é o `IE`, e a diferença
/// de um byte é um registrador de interrupções inteiro.
const HRAM_LEN: usize = 0x7F;

/// Primeiro endereço da HRAM.
const HRAM_BASE: usize = 0xFF80;

/// O que `$FEA0`–`$FEFF` devolve neste emulador, e por quê.
///
/// A § FEA0–FEFF range, no que cabe ao DMG:
///
/// > This area returns $FF when OAM is blocked, and otherwise the behavior
/// > depends on the hardware revision.
/// > - On DMG, MGB, SGB, and SGB2, reads during OAM block trigger OAM
/// >   corruption. Reads otherwise return $00.
///
/// O `$FF` está condicionado a "OAM bloqueada", que é estado da PPU (modos 2 e
/// 3). Não há PPU até o M3, logo a OAM nunca está bloqueada, logo a leitura é
/// `$00`. Ler "não usável" e responder `$FF` — o reflexo — seria afirmar um
/// bloqueio que este emulador ainda não tem como ter.
///
/// Quando o 3.6 trouxer o bloqueio por modo, esta constante vira o ramo "fora
/// de bloqueio" de uma decisão de dois. A corrupção de OAM que a spec cita na
/// mesma frase é o 7.2 (`oam_bug`).
const NOT_USABLE_READ: u8 = 0x00;

/// Uma faixa do mapa de memória, com o dono que a § Memory Map lhe dá.
///
/// Existe separada do [`Bus`] porque o mapa é um fato sobre o hardware e não
/// sobre o que já está implementado: decodificar `$8000` como VRAM é verdade
/// hoje, mesmo sem PPU para atender. Isso deixa a tabela testável contra a
/// spec, endereço por endereço, sem depender de qual componente existe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Region {
    /// `$0000`–`$7FFF` — ROM do cartucho: banco fixo `00` e banco `01`–`NN`,
    /// que o mapeador chaveia. Sem MBC, o endereço é o índice na ROM.
    CartridgeRom,
    /// `$8000`–`$9FFF` — Video RAM. Dona: a PPU (ROADMAP 3.1).
    VideoRam,
    /// `$A000`–`$BFFF` — RAM externa, no cartucho, se houver.
    ExternalRam,
    /// `$C000`–`$DFFF` — Work RAM, 8 KiB.
    WorkRam,
    /// `$E000`–`$FDFF` — echo da WRAM. Espelha `$C000`–`$DDFF`, e **só** até
    /// ali: a janela é 512 bytes mais curta que a WRAM.
    EchoRam,
    /// `$FE00`–`$FE9F` — Object Attribute Memory. Dona: a PPU (ROADMAP 3.5).
    ObjectAttributeMemory,
    /// `$FEA0`–`$FEFF` — a região que a Nintendo proíbe. Ver [`NOT_USABLE_READ`].
    NotUsable,
    /// `$FF00`–`$FF7F` — registradores de I/O: joypad, serial, timer, som, LCD.
    ///
    /// Faixa de dono repartido: cada endereço pertence a um componente
    /// diferente, e 72 dos 128 não aparecem sequer na tabela do hand-off. Ver
    /// [`boot`].
    IoRegisters,
    /// `$FF80`–`$FFFE` — High RAM, 127 bytes dentro da própria CPU.
    HighRam,
    /// `$FFFF` — o registrador `IE`. Um endereço, faixa própria. O valor
    /// inicial é do 1.2b-ii; o despacho de interrupções é o 2.2.
    InterruptEnable,
}

impl Region {
    /// A que faixa do mapa pertence este endereço.
    ///
    /// Total por construção: os braços cobrem os 16 bits sem `_ =>`, para que
    /// acrescentar uma faixa nova quebre o `match` em vez de cair num ramo
    /// genérico que ninguém escreveu de propósito.
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

/// O barramento de memória do Game Boy.
///
/// Hoje ele liga o cartucho, a WRAM, a HRAM, os registradores de hardware que a
/// tabela do hand-off nomeia (1.2b-ii) e o `IE`. VRAM, OAM e os 72 endereços de
/// I/O sobre os quais a spec se cala continuam sem componente — neles a leitura
/// é [`OPEN_BUS`] e a escrita se perde. Isso **não** é uma afirmação sobre o
/// hardware; é a ausência de quem responda, fixada por teste para ser uma
/// decisão visível em vez de uma lacuna a descobrir depurando.
///
/// Entrar em pânico ali seria pior: `read` de emulador é o último lugar onde se
/// quer descobrir erro de roteamento — a mesma escolha que o `NoMbc` fez no 0.4.
pub struct Bus {
    cartridge: Box<dyn Cartridge>,
    wram: [u8; WRAM_LEN],
    hram: [u8; HRAM_LEN],
    /// `$FF00`–`$FF7F`. Byte cru: só os endereços de [`boot::IO_HAS_OWNER`]
    /// são alcançáveis, e nenhum tem máscara ou efeito colateral ainda.
    io: [u8; boot::IO_LEN],
    /// `$FFFF`. Campo próprio porque é região própria.
    ie: u8,
}

impl Bus {
    /// Liga um cartucho ao barramento, no estado em que a boot ROM do **DMG**
    /// entrega o controle ao cartucho.
    ///
    /// Não há um `Bus::after_boot_rom` separado, e a assimetria com
    /// [`crate::cpu::Registers::after_boot_rom`] é deliberada: lá o estado
    /// depende do checksum da ROM, então havia o que um construtor carregar;
    /// aqui a coluna DMG / MGB é literal, e este emulador não tem outro estado
    /// em que estar — ele nunca roda a boot ROM.
    ///
    /// O que sai daqui mistura duas coisas, e vale saber qual é qual:
    ///
    /// - **Spec:** os registradores de hardware, do módulo [`boot`].
    /// - **Escolha deste emulador:** a RAM interna zerada. A § Console state
    ///   after boot ROM hand-off diz que "the console's WRAM and HRAM are random
    ///   on power-up" e que os emuladores divergem — uns preenchem com uma
    ///   constante (`$00` ou `$FF`), outros sorteiam. Constante é o que dá teste
    ///   reprodutível, e a própria spec desaconselha que um jogo dependa disso.
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

    /// Lê um byte de qualquer endereço. Não avança o tempo.
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

    /// Escreve um byte em qualquer endereço. Não avança o tempo.
    ///
    /// A escrita em `$0000`–`$7FFF` **é** entregue ao cartucho, ainda que ROM
    /// não se escreva: é por ela que se fala com o mapeador (4.2). Num cartucho
    /// sem MBC ela não faz nada, e quem decide isso é o cartucho, não o `Bus`.
    ///
    /// Em `$FEA0`–`$FEFF` a escrita se perde. A spec descreve a leitura daquela
    /// faixa como constante, o que já implica que não há célula a escrever —
    /// guardar o byte ali contradiria [`NOT_USABLE_READ`].
    ///
    /// Nos registradores de I/O a escrita é **crua**: o byte entra inteiro, sem
    /// máscara, sem bit read-only e sem efeito colateral. Escrever em `DIV`
    /// devia zerá-lo, escrever em `LY` não devia fazer nada, e os dois aqui
    /// guardam o byte. Isso é divergência conhecida e delimitada: o 1.2b-ii
    /// entrega valor inicial, e a semântica vem com o componente dono
    /// (timer 2.1, interrupções 2.2, PPU 3.1, APU M6).
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

/// Índice na WRAM, para um endereço que já se sabe ser [`Region::WorkRam`] ou
/// [`Region::EchoRam`].
///
/// A máscara nunca produz índice fora dos 8 KiB, então não há caso de erro a
/// tratar. E ela é o que dá de graça o fato que a intuição erra: o echo termina
/// em `$FDFF`, que mascarado dá `$1DFF` — os 512 bytes finais da WRAM
/// (`$DE00`–`$DFFF`) simplesmente não têm endereço de echo que os alcance.
const fn wram_index(addr: u16) -> usize {
    (addr as usize) & WRAM_ADDRESS_MASK
}

/// Índice na HRAM, para um endereço que já se sabe ser [`Region::HighRam`].
const fn hram_index(addr: u16) -> usize {
    (addr as usize) - HRAM_BASE
}

/// Índice na faixa de I/O, para um endereço que já se sabe ser
/// [`Region::IoRegisters`]. Ter índice não é ter dono: quem decide isso é
/// [`boot::IO_HAS_OWNER`].
const fn io_index(addr: u16) -> usize {
    (addr as usize) - boot::IO_BASE
}

/// `derive(Debug)` despejaria 8 KiB de WRAM no terminal, e o `Box<dyn
/// Cartridge>` nem o tem — o trait do 0.4 não exige `Debug`, e exigi-lo agora
/// seria mexer no contrato do cartucho por conveniência de impressão.
impl std::fmt::Debug for Bus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bus")
            .field("wram_len", &self.wram.len())
            .field("hram_len", &self.hram.len())
            .finish_non_exhaustive()
    }
}
