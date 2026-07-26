//! Banco de registradores do SM83 (ROADMAP 1.1). spec: `docs/reference/02-cpu.md` § CPU registers and flags.

use crate::cart::HeaderChecksum;

// Quatro flags, e só. Z80 tem S no bit 7 e P/V no bit 2 — foram removidos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Flag {
    Z,
    N,
    H,
    C,
}

impl Flag {
    const fn mask(self) -> u8 {
        match self {
            Self::Z => 0b1000_0000,
            Self::N => 0b0100_0000,
            Self::H => 0b0010_0000,
            Self::C => 0b0001_0000,
        }
    }
}

const fn pair(high: u8, low: u8) -> u16 {
    ((high as u16) << 8) | (low as u16)
}

const fn split(value: u16) -> (u8, u8) {
    ((value >> 8) as u8, value as u8)
}

// Campos públicos — banco de registradores, não invariante a proteger.
// Pares de 16 bits são métodos (há cálculo). Default é tudo zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Registers {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
}

impl Registers {
    // Hand-off da boot ROM do DMG. spec: 01-memory-map.md § Console state, coluna DMG.
    // F depende do checksum gravado em $014D (não do calculado).
    #[must_use]
    pub const fn after_boot_rom(checksum: HeaderChecksum) -> Self {
        let carries = checksum.stored() != 0;

        let mut regs = Self {
            a: 0x01,
            f: 0,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,
            sp: 0xFFFE,
            pc: 0x0100,
        };

        regs.set_flag(Flag::Z, true);
        regs.set_flag(Flag::N, false);
        regs.set_flag(Flag::H, carries);
        regs.set_flag(Flag::C, carries);

        regs
    }

    #[must_use]
    pub const fn af(&self) -> u16 {
        pair(self.a, self.f)
    }

    #[must_use]
    pub const fn bc(&self) -> u16 {
        pair(self.b, self.c)
    }

    #[must_use]
    pub const fn de(&self) -> u16 {
        pair(self.d, self.e)
    }

    #[must_use]
    pub const fn hl(&self) -> u16 {
        pair(self.h, self.l)
    }

    // Sem máscara no nibble baixo de F: o Pan Docs no commit fixado não descreve
    // os bits 3–0. Se for necessária, quem cobra é cpu_instrs/01-special no 1.13
    // (ver docs/iterations/0009).
    pub const fn set_af(&mut self, value: u16) {
        (self.a, self.f) = split(value);
    }

    pub const fn set_bc(&mut self, value: u16) {
        (self.b, self.c) = split(value);
    }

    pub const fn set_de(&mut self, value: u16) {
        (self.d, self.e) = split(value);
    }

    pub const fn set_hl(&mut self, value: u16) {
        (self.h, self.l) = split(value);
    }

    #[must_use]
    pub const fn flag(&self, flag: Flag) -> bool {
        self.f & flag.mask() != 0
    }

    pub const fn set_flag(&mut self, flag: Flag, on: bool) {
        let mask = flag.mask();
        if on {
            self.f |= mask;
        } else {
            self.f &= !mask;
        }
    }
}
