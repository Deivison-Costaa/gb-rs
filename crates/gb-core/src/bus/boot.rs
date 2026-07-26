//! Registradores de hardware no hand-off da boot ROM (ROADMAP 1.2b-ii).
//! spec: `docs/reference/01-memory-map.md` § Console state after boot ROM hand-off,
//! coluna DMG / MGB.

pub(super) const IO_BASE: usize = 0xFF00;
pub(super) const IO_LEN: usize = 0x80;

// OBP0/OBP1: spec não dá valor ($00 por escolha, teste reprodutível).
const UNINITIALIZED: u8 = 0x00;

// Coluna DMG / MGB da tabela § Hardware registers. None = ?? na spec.
const HARDWARE_REGISTERS: &[(u16, Option<u8>)] = &[
    (0xFF00, Some(0xCF)), // P1
    (0xFF01, Some(0x00)), // SB
    (0xFF02, Some(0x7E)), // SC
    (0xFF04, Some(0xAB)), // DIV
    (0xFF05, Some(0x00)), // TIMA
    (0xFF06, Some(0x00)), // TMA
    (0xFF07, Some(0xF8)), // TAC
    (0xFF0F, Some(0xE1)), // IF
    (0xFF10, Some(0x80)), // NR10
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
    (0xFF26, Some(0xF1)), // NR52
    (0xFF40, Some(0x91)), // LCDC
    (0xFF41, Some(0x85)), // STAT
    (0xFF42, Some(0x00)), // SCY
    (0xFF43, Some(0x00)), // SCX
    (0xFF44, Some(0x00)), // LY
    (0xFF45, Some(0x00)), // LYC
    (0xFF46, Some(0xFF)), // DMA — $00 é CGB/AGB (erro #1 da 0012)
    (0xFF47, Some(0xFC)), // BGP
    (0xFF48, None),       // OBP0 — ?? na spec
    (0xFF49, None),       // OBP1 — ?? na spec
    (0xFF4A, Some(0x00)), // WY
    (0xFF4B, Some(0x00)), // WX
];

pub(super) const INTERRUPT_ENABLE: u8 = 0x00;

pub(super) const IO: [u8; IO_LEN] = {
    let mut io = [UNINITIALIZED; IO_LEN];
    let mut i = 0;

    while i < HARDWARE_REGISTERS.len() {
        let index = (HARDWARE_REGISTERS[i].0 as usize) - IO_BASE;
        if let Some(value) = HARDWARE_REGISTERS[i].1 {
            io[index] = value;
        }
        i += 1;
    }

    io
};

pub(super) const IO_HAS_OWNER: [bool; IO_LEN] = {
    let mut owned = [false; IO_LEN];
    let mut i = 0;

    while i < HARDWARE_REGISTERS.len() {
        owned[(HARDWARE_REGISTERS[i].0 as usize) - IO_BASE] = true;
        i += 1;
    }

    owned
};
