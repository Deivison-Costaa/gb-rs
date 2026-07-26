//! Os registradores de hardware no hand-off da boot ROM (ROADMAP 1.2b-ii).
//!
//! Spec: `docs/reference/01-memory-map.md` § Console state after boot ROM
//! hand-off → *Hardware registers* (Pan Docs `fe246067b695`), coluna
//! **DMG / MGB**. Este emulador não roda a boot ROM: ele começa com os
//! registradores que ela teria deixado, e a spec diz de onde ela os tira —
//! *"As far as timing-sensitive values are concerned, these values are recorded
//! at PC = $0100."*
//!
//! O que esta tabela **é**: um retrato dos valores lidos naquele instante.
//!
//! O que ela **não é**: a semântica de cada registrador. Que `LY` seja
//! read-only, que escrever em `DIV` o zere, que os 5 bits altos de `TAC` leiam
//! 1 — nada disso está aqui, e nada disso é desta iteração. Quem traz isso é o
//! componente dono: timer (2.1), interrupções (2.2), PPU (3.1), APU (M6). Um
//! byte guardado não é um contador emulado, e o `DIV` daqui vai valer `$AB` para
//! sempre até o 2.1 chegar.
//!
//! **A spec avisa que a seção é frágil:** *"Some of the information below is
//! highly volatile […] thus, some of it may contain errors."* Quem cobra de
//! verdade é a Mooneye `acceptance/boot_hwio-dmgABCmgb`, no ROADMAP 7.1 — a
//! própria tabela diz ter saído dela.

/// Primeiro endereço da faixa de I/O.
pub(super) const IO_BASE: usize = 0xFF00;

/// `$FF00`–`$FF7F` — 128 endereços. A tabela nomeia 41 e cala sobre 72.
pub(super) const IO_LEN: usize = 0x80;

/// O que este emulador põe nos registradores que a tabela marca como **não
/// inicializados** (`OBP0` e `OBP1`).
///
/// A nota de rodapé das duas linhas:
///
/// > These registers are left entirely uninitialized. Their value tends to be
/// > most often $00 or $FF, but the value is especially not reliable if your
/// > software runs after e.g. a flashcart or multicart selection menu.
///
/// A spec **não dá** valor, e avisa que os dois candidatos óbvios são tendência
/// e não regra. `$00` aqui é a mesma escolha que o 1.2a fez para a WRAM e a
/// HRAM: constante é o que dá teste reprodutível. Escrever `$FF` "porque paleta
/// não inicializada costuma ser branca" seria transformar uma tendência
/// observada em fiação — exatamente o que a R1 proíbe.
///
/// Não confundir com [`crate::cart::OPEN_BUS`], que também vale `$FF` e diz
/// outra coisa: lá é "ninguém respondeu", aqui é "há registrador, e o conteúdo
/// dele não está especificado".
const UNINITIALIZED: u8 = 0x00;

/// A coluna **DMG / MGB** da tabela § Hardware registers, linha a linha.
///
/// `None` transcreve o `??` da spec: a linha existe, o registrador existe, e o
/// valor não é dado — ver [`UNINITIALIZED`].
///
/// As linhas `---` **não estão aqui**, e a ausência é o ponto: `KEY0`, `KEY1`,
/// `VBK`, `HDMA1`–`HDMA5`, `RP`, `BGPI`/`BGPD`, `OGPI`/`OGPD` e `SVBK` são
/// registradores de CGB, e este console não os tem. `BANK` (`$FF50`) é `---` nas
/// quatro colunas — ele desmapeia a boot ROM, e a tabela é tirada depois disso
/// já ter acontecido. `---` não é `$00`: um registrador que não existe e um que
/// existe zerado se distinguem na primeira leitura que um jogo fizer.
///
/// A ordem é a da spec, e as lacunas de endereço também: `$FF03`, `$FF08`–`$FF0E`,
/// `$FF15`, `$FF1F`, `$FF27`–`$FF2F`, `$FF30`–`$FF3F` (wave RAM) e o resto acima
/// de `$FF4B` não aparecem na tabela. Sobre eles a seção se cala, e silêncio de
/// spec não vira valor — esses endereços continuam sem dono no [`super::Bus`],
/// como VRAM e OAM.
const HARDWARE_REGISTERS: &[(u16, Option<u8>)] = &[
    (0xFF00, Some(0xCF)), // P1   — joypad (4.1)
    (0xFF01, Some(0x00)), // SB   — serial (1.12)
    (0xFF02, Some(0x7E)), // SC
    (0xFF04, Some(0xAB)), // DIV  — timer (2.1). $18 é a coluna DMG0.
    (0xFF05, Some(0x00)), // TIMA
    (0xFF06, Some(0x00)), // TMA
    (0xFF07, Some(0xF8)), // TAC
    (0xFF0F, Some(0xE1)), // IF   — interrupções (2.2)
    (0xFF10, Some(0x80)), // NR10 — APU (M6), daqui até $FF26
    (0xFF11, Some(0xBF)), // NR11
    (0xFF12, Some(0xF3)), // NR12
    (0xFF13, Some(0xFF)), // NR13
    (0xFF14, Some(0xBF)), // NR14
    (0xFF16, Some(0x3F)), // NR21
    (0xFF17, Some(0x00)), // NR22
    (0xFF18, Some(0xFF)), // NR23
    (0xFF19, Some(0xBF)), // NR24
    (0xFF1A, Some(0x7F)), // NR30
    (0xFF1B, Some(0xFF)), // NR31
    (0xFF1C, Some(0x9F)), // NR32
    (0xFF1D, Some(0xFF)), // NR33
    (0xFF1E, Some(0xBF)), // NR34
    (0xFF20, Some(0xFF)), // NR41
    (0xFF21, Some(0x00)), // NR42
    (0xFF22, Some(0x00)), // NR43
    (0xFF23, Some(0xBF)), // NR44
    (0xFF24, Some(0x77)), // NR50
    (0xFF25, Some(0xF3)), // NR51
    (0xFF26, Some(0xF1)), // NR52 — $F0 é a coluna SGB / SGB2
    (0xFF40, Some(0x91)), // LCDC — PPU (3.1), daqui até $FF4B
    (0xFF41, Some(0x85)), // STAT — $81 é a coluna DMG0
    (0xFF42, Some(0x00)), // SCY
    (0xFF43, Some(0x00)), // SCX
    (0xFF44, Some(0x00)), // LY   — $91 é a coluna DMG0
    (0xFF45, Some(0x00)), // LYC
    (0xFF46, Some(0xFF)), // DMA  — $00 é a coluna CGB / AGB
    (0xFF47, Some(0xFC)), // BGP
    (0xFF48, None),       // OBP0 — `??` na spec
    (0xFF49, None),       // OBP1 — `??` na spec
    (0xFF4A, Some(0x00)), // WY
    (0xFF4B, Some(0x00)), // WX
];

/// A última linha da mesma tabela. Mora à parte porque `$FFFF` é
/// [`super::Region::InterruptEnable`], e não [`super::Region::IoRegisters`] —
/// um endereço, uma região, um campo.
pub(super) const INTERRUPT_ENABLE: u8 = 0x00;

/// O conteúdo de `$FF00`–`$FF7F` no hand-off.
///
/// Os endereços sem linha na tabela recebem `UNINITIALIZED` e **nunca são
/// lidos**: [`IO_HAS_OWNER`] os deixa fora, e o [`super::Bus`] responde
/// `OPEN_BUS` para eles. O valor aqui é só o que preenche o buraco do array.
pub(super) const IO: [u8; IO_LEN] = {
    let mut io = [UNINITIALIZED; IO_LEN];
    let mut i = 0;

    while i < HARDWARE_REGISTERS.len() {
        // Índice fora de $FF00–$FF7F estoura em tempo de compilação, e é a
        // guarda que se quer: linha nova fora da faixa não compila.
        let index = (HARDWARE_REGISTERS[i].0 as usize) - IO_BASE;
        if let Some(value) = HARDWARE_REGISTERS[i].1 {
            io[index] = value;
        }
        i += 1;
    }

    io
};

/// Quais dos 128 endereços de I/O têm alguém do outro lado.
///
/// Só os que a tabela nomeia — inclusive os dois `??`, que não têm valor
/// especificado mas **têm** registrador (são as paletas de objeto da PPU). Os
/// outros 72 não estão na tabela, e nada em `docs/reference/` diz o que eles
/// fazem no DMG; ficam sem dono, como VRAM e OAM ficaram no 1.2a.
pub(super) const IO_HAS_OWNER: [bool; IO_LEN] = {
    let mut owned = [false; IO_LEN];
    let mut i = 0;

    while i < HARDWARE_REGISTERS.len() {
        owned[(HARDWARE_REGISTERS[i].0 as usize) - IO_BASE] = true;
        i += 1;
    }

    owned
};
