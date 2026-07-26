//! ROADMAP 1.2b-i — os registradores da CPU no hand-off da boot ROM.

use gb_core::cart::CartridgeHeader;
use gb_core::cpu::{Flag, Registers};

const MIN_ROM_LEN: usize = 0x0150;
const CHECKSUMMED_FIRST: usize = 0x0134;
const CHECKSUMMED_LEN: u8 = 0x19;
const HEADER_CHECKSUM: usize = 0x014D;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Column {
    name: &'static str,
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
}

impl Column {
    fn of(regs: &Registers, name: &'static str) -> Self {
        Self {
            name,
            a: regs.a,
            b: regs.b,
            c: regs.c,
            d: regs.d,
            e: regs.e,
            h: regs.h,
            l: regs.l,
        }
    }

    fn same_values(self, other: Self) -> bool {
        (self.a, self.b, self.c, self.d, self.e, self.h, self.l)
            == (
                other.a, other.b, other.c, other.d, other.e, other.h, other.l,
            )
    }
}

const DMG: Column = Column {
    name: "DMG",
    a: 0x01,
    b: 0x00,
    c: 0x13,
    d: 0x00,
    e: 0xD8,
    h: 0x01,
    l: 0x4D,
};

const OTHER_MODELS: [Column; 4] = [
    Column {
        name: "DMG0",
        a: 0x01,
        b: 0xFF,
        c: 0x13,
        d: 0x00,
        e: 0xC1,
        h: 0x84,
        l: 0x03,
    },
    Column {
        name: "MGB",
        a: 0xFF,
        b: 0x00,
        c: 0x13,
        d: 0x00,
        e: 0xD8,
        h: 0x01,
        l: 0x4D,
    },
    Column {
        name: "SGB",
        a: 0x01,
        b: 0x00,
        c: 0x14,
        d: 0x00,
        e: 0x00,
        h: 0xC0,
        l: 0x60,
    },
    Column {
        name: "SGB2",
        a: 0xFF,
        b: 0x00,
        c: 0x14,
        d: 0x00,
        e: 0x00,
        h: 0xC0,
        l: 0x60,
    },
];

fn rom_with_checksums(stored: u8, computed: u8) -> Vec<u8> {
    let mut rom = vec![0u8; MIN_ROM_LEN];
    rom[CHECKSUMMED_FIRST] = 0u8.wrapping_sub(computed).wrapping_sub(CHECKSUMMED_LEN);
    rom[HEADER_CHECKSUM] = stored;

    let checksum = CartridgeHeader::parse(&rom)
        .expect("ROM do fixture tem o cabeçalho inteiro")
        .checksum();
    assert_eq!(checksum.stored(), stored, "fixture: $014D saiu errado");
    assert_eq!(
        checksum.computed(),
        computed,
        "fixture: o checksum calculado saiu errado"
    );

    rom
}

fn after_boot(stored: u8, computed: u8) -> Registers {
    let rom = rom_with_checksums(stored, computed);
    let header = CartridgeHeader::parse(&rom).expect("ROM do fixture tem o cabeçalho inteiro");

    Registers::after_boot_rom(header.checksum())
}

fn after_boot_valid(checksum: u8) -> Registers {
    after_boot(checksum, checksum)
}

#[test]
fn the_eight_bit_registers_come_from_the_dmg_column() {
    let regs = after_boot_valid(0x42);

    assert_eq!(regs.a, 0x01, "A");
    assert_eq!(regs.b, 0x00, "B");
    assert_eq!(regs.c, 0x13, "C");
    assert_eq!(regs.d, 0x00, "D");
    assert_eq!(regs.e, 0xD8, "E");
    assert_eq!(regs.h, 0x01, "H");
    assert_eq!(regs.l, 0x4D, "L");
}

#[test]
fn the_pairs_compose_the_values_the_folklore_quotes() {
    let regs = after_boot_valid(0x42);

    assert_eq!(regs.bc(), 0x0013, "BC");
    assert_eq!(regs.de(), 0x00D8, "DE");
    assert_eq!(regs.hl(), 0x014D, "HL");
}

#[test]
fn sp_and_pc_hand_off_at_the_cartridge_entry_point() {
    let regs = after_boot_valid(0x42);

    assert_eq!(
        regs.pc, 0x0100,
        "PC — é aqui que a boot ROM entrega o controle"
    );
    assert_eq!(
        regs.sp, 0xFFFE,
        "SP — topo da HRAM, e a pilha cresce para baixo"
    );
}

#[test]
fn this_is_the_dmg_column_and_not_one_of_the_other_four() {
    let regs = after_boot_valid(0x42);
    let actual = Column::of(&regs, "implementado");

    assert!(
        actual.same_values(DMG),
        "os registradores não são a coluna DMG: {actual:?}"
    );

    for other in OTHER_MODELS {
        assert!(
            !actual.same_values(other),
            "os registradores saíram iguais à coluna {}, que é outro console",
            other.name
        );
    }
}

#[test]
fn z_is_set_and_n_is_clear_whatever_the_checksum() {
    for checksum in [0x00, 0x01, 0x7F, 0x80, 0xFF] {
        let regs = after_boot_valid(checksum);

        assert!(regs.flag(Flag::Z), "Z com checksum ${checksum:02X}");
        assert!(!regs.flag(Flag::N), "N com checksum ${checksum:02X}");
    }
}

#[test]
fn half_carry_and_carry_are_clear_when_the_header_checksum_is_zero() {
    let regs = after_boot_valid(0x00);

    assert!(!regs.flag(Flag::H), "H com checksum $00");
    assert!(!regs.flag(Flag::C), "C com checksum $00");
}

#[test]
fn half_carry_and_carry_are_set_for_every_other_header_checksum() {
    for checksum in 0x01..=0xFF {
        let regs = after_boot_valid(checksum);

        assert!(regs.flag(Flag::H), "H com checksum ${checksum:02X}");
        assert!(regs.flag(Flag::C), "C com checksum ${checksum:02X}");
    }
}

#[test]
fn the_flags_follow_the_stored_byte_and_not_the_computed_one() {
    let stored_zero = after_boot(0x00, 0xE7);
    assert!(
        !stored_zero.flag(Flag::H) && !stored_zero.flag(Flag::C),
        "com $014D = $00, H e C são limpos mesmo que o calculado não seja zero"
    );

    let computed_zero = after_boot(0x5A, 0x00);
    assert!(
        computed_zero.flag(Flag::H) && computed_zero.flag(Flag::C),
        "com $014D = $5A, H e C são setados mesmo que o calculado seja zero"
    );
}

#[test]
fn f_is_b0_on_a_valid_cartridge_and_80_on_the_zero_checksum() {
    assert_eq!(after_boot_valid(0x42).f, 0b1011_0000, "Z=1 N=0 H=1 C=1");
    assert_eq!(after_boot_valid(0x00).f, 0b1000_0000, "Z=1 N=0 H=0 C=0");

    assert_eq!(after_boot_valid(0x42).af(), 0x01B0);
    assert_eq!(after_boot_valid(0x00).af(), 0x0180);
}

#[test]
fn the_low_nibble_of_f_has_no_value_in_the_spec_and_comes_out_zero() {
    for checksum in 0x00..=0xFF {
        let regs = after_boot_valid(checksum);

        assert_eq!(
            regs.f & 0b0000_1111,
            0,
            "nibble baixo de F com checksum ${checksum:02X}"
        );
    }
}

#[test]
fn the_boot_state_is_a_constructor_and_not_the_default() {
    let regs = after_boot_valid(0x42);

    assert_ne!(regs, Registers::default());
    assert_eq!(
        Registers::default().pc,
        0x0000,
        "o default não pula a boot ROM"
    );
}
