//! ROADMAP 1.9d — `BIT` (`CB 40`–`CB 7F`).
//! Dois M-cycles para registrador (8 T-cycles), três para `(HL)` (12 T-cycles).
//! `Z` = bit testado é zero, `N=0`, `H=1`, `C` intocado.
//! `(HL)` lê sem escrever — ao contrário das rotações, não é read-modify-write.

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::{Cpu, Flag};

mod support;

const ENTRY: usize = 0x0100;

fn machine(program: &[u8]) -> (Cpu, Bus) {
    let mut rom = vec![0x00; NoMbc::MAX_ROM_LEN];
    rom[ENTRY..ENTRY + program.len()].copy_from_slice(program);

    let checksum = CartridgeHeader::parse(&rom)
        .expect("32 KiB de ROM têm cabeçalho de sobra")
        .checksum();
    let cartridge = NoMbc::new(rom).expect("32 KiB cabem num cartucho sem MBC");

    (Cpu::after_boot_rom(checksum), Bus::new(Box::new(cartridge)))
}

fn step_bit_reg(opcode: u8, value: u8, set_fn: fn(&mut Cpu, u8)) -> (bool, bool, bool, bool) {
    let (mut cpu, mut bus) = machine(&[0xCB, opcode]);

    set_fn(&mut cpu, value);
    cpu.registers.set_flag(Flag::C, false);
    cpu.registers.set_flag(Flag::H, false);

    cpu.step(&mut bus); // M1: fetch $CB
    cpu.step(&mut bus); // M2: fetch opcode + execução

    let z = cpu.registers.flag(Flag::Z);
    let n = cpu.registers.flag(Flag::N);
    let h = cpu.registers.flag(Flag::H);
    let c = cpu.registers.flag(Flag::C);
    (z, n, h, c)
}

// ── BIT 0,B ($CB 40) ───────────────────────────────────────────────────────

#[test]
fn bit_0_of_b_sets_z_when_bit_is_zero_and_clears_z_when_bit_is_one() {
    // bit 0 = 0 → Z = 1
    let (z, _, _, _) = step_bit_reg(0x40, 0xFE, |cpu, v| cpu.registers.b = v);
    assert!(z, "BIT 0,B com valor 0xFE (bit 0 = 0): Z deve ser 1");

    // bit 0 = 1 → Z = 0
    let (z, _, _, _) = step_bit_reg(0x40, 0x01, |cpu, v| cpu.registers.b = v);
    assert!(!z, "BIT 0,B com valor 0x01 (bit 0 = 1): Z deve ser 0");
}

#[test]
fn bit_sets_n_to_zero_h_to_one_and_preserves_carry() {
    let (_, n, h, c) = step_bit_reg(0x40, 0x00, |cpu, v| cpu.registers.b = v);

    assert!(!n, "BIT: N deve ser 0");
    assert!(h, "BIT: H deve ser 1");
    assert!(!c, "BIT: C intocado (false antes)");
}

#[test]
fn bit_preserves_carry_unchanged() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x40]);
    cpu.registers.b = 0x00;
    cpu.registers.set_flag(Flag::C, true); // C=1 antes do BIT

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert!(
        cpu.registers.flag(Flag::C),
        "BIT: C deve continuar 1 (intocado)"
    );
}

// ── BIT 3,C ($CB 59) — meio da tabela: bit 3, registrador C ────────────────

#[test]
fn bit_3_of_c_sets_z_when_bit_is_zero_and_clears_z_when_bit_is_one() {
    let (z, _, _, _) = step_bit_reg(0x59, 0b1111_0111, |cpu, v| cpu.registers.c = v);
    assert!(z, "BIT 3,C com valor 0b1111_0111 (bit 3 = 0): Z deve ser 1");

    let (z, _, _, _) = step_bit_reg(0x59, 0b0000_1000, |cpu, v| cpu.registers.c = v);
    assert!(
        !z,
        "BIT 3,C com valor 0b0000_1000 (bit 3 = 1): Z deve ser 0"
    );
}

// ── BIT 7,A ($CB 7F) — último opcode da faixa: bit 7, registrador A ────────

#[test]
fn bit_7_of_a_sets_z_when_bit_is_zero() {
    let (z, _, _, _) = step_bit_reg(0x7F, 0x7F, |cpu, v| cpu.registers.a = v);
    assert!(z, "BIT 7,A com valor 0x7F (bit 7 = 0): Z deve ser 1");
}

#[test]
fn bit_7_of_a_clears_z_when_bit_is_one() {
    let (z, _, _, _) = step_bit_reg(0x7F, 0x80, |cpu, v| cpu.registers.a = v);
    assert!(!z, "BIT 7,A com valor 0x80 (bit 7 = 1): Z deve ser 0");
}

#[test]
fn bit_does_not_modify_any_register() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x40]); // BIT 0,B
    cpu.registers.a = 0x12;
    cpu.registers.b = 0x34;
    cpu.registers.c = 0x56;
    cpu.registers.d = 0x78;
    cpu.registers.e = 0x9A;
    cpu.registers.set_hl(0xBCDE);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(cpu.registers.a, 0x12, "BIT não modifica A");
    assert_eq!(cpu.registers.b, 0x34, "BIT não modifica B");
    assert_eq!(cpu.registers.c, 0x56, "BIT não modifica C");
    assert_eq!(cpu.registers.d, 0x78, "BIT não modifica D");
    assert_eq!(cpu.registers.e, 0x9A, "BIT não modifica E");
    assert_eq!(cpu.registers.hl(), 0xBCDE, "BIT não modifica HL");
}

// ── BIT (HL) ($CB 46, $CB 4E, $CB 5E, ...) — 3 M-cycles ────────────────────

#[test]
fn bit_hl_reads_without_writing_in_three_m_cycles() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x46, 0x00, 0x00, 0x00]); // BIT 0,(HL)

    cpu.registers.set_hl(0xC000);
    bus.write(0xC000, 0x00); // bit 0 = 0

    cpu.step(&mut bus); // M1: fetch $CB
    assert_eq!(cpu.registers.pc, ENTRY as u16 + 1);

    cpu.step(&mut bus); // M2: fetch opcode
    assert_eq!(cpu.registers.pc, ENTRY as u16 + 2);
    assert!(
        !cpu.is_between_instructions(),
        "depois do M2 (BIT (HL)) a instrução não acabou"
    );

    cpu.step(&mut bus); // M3: read((HL)) e aplica BIT
    assert!(
        cpu.is_between_instructions(),
        "depois do M3 a instrução BIT (HL) acabou"
    );
}

#[test]
fn bit_hl_detects_bit_correctly() {
    for (opcode, bit_idx, value) in [
        (0x46u8, 0u8, 0xFEu8), // BIT 0,(HL), bit 0 = 0
        (0x4E, 1, 0xFD),       // BIT 1,(HL), bit 1 = 0
        (0x56, 2, 0xFB),       // BIT 2,(HL), bit 2 = 0
        (0x5E, 3, 0xF7),       // BIT 3,(HL), bit 3 = 0
        (0x66, 4, 0xEF),       // BIT 4,(HL), bit 4 = 0
        (0x6E, 5, 0xDF),       // BIT 5,(HL), bit 5 = 0
        (0x76, 6, 0xBF),       // BIT 6,(HL), bit 6 = 0
        (0x7E, 7, 0x7F),       // BIT 7,(HL), bit 7 = 0
    ] {
        let (mut cpu, mut bus) = machine(&[0xCB, opcode]);
        cpu.registers.set_hl(0xC000);
        bus.write(0xC000, value);

        cpu.step(&mut bus);
        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert!(
            cpu.registers.flag(Flag::Z),
            "BIT {},{opcode:#04X} com bit {bit_idx}=0: Z deve ser 1",
            bit_idx
        );
    }
}

#[test]
fn bit_hl_does_not_modify_memory() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x46]); // BIT 0,(HL)
    cpu.registers.set_hl(0xC000);
    bus.write(0xC000, 0xAA);

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        bus.read(0xC000),
        0xAA,
        "BIT (HL) não escreve — memória continua 0xAA"
    );
}

#[test]
fn bit_hl_flags_z_from_bit_n0_h1_c_unchanged() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x4E]); // BIT 1,(HL)
    cpu.registers.set_hl(0xC000);
    bus.write(0xC000, 0x02); // bit 1 = 1 → Z = 0
    cpu.registers.set_flag(Flag::C, true);

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert!(!cpu.registers.flag(Flag::Z), "BIT 1,(HL) com bit=1: Z=0");
    assert!(!cpu.registers.flag(Flag::N), "BIT: N=0");
    assert!(cpu.registers.flag(Flag::H), "BIT: H=1");
    assert!(cpu.registers.flag(Flag::C), "BIT: C intocado (true)");
}

// ── Controle negativo ──────────────────────────────────────────────────────

#[test]
fn bit_second_bytes_40_to_7f_known_ones_are_still_in_decoded_elsewhere() {
    let known: &[u8] = &[
        0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, // BIT 0
        0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F, // BIT 1
        0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, // BIT 2
        0x58, 0x59, 0x5A, 0x5B, 0x5C, 0x5D, 0x5E, 0x5F, // BIT 3
        0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, // BIT 4
        0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F, // BIT 5
        // 0x76 é HALT e não está em decoded_elsewhere — CB 76 é BIT 6,(HL)
        0x70, 0x71, 0x72, 0x73, 0x74, 0x75, /* 0x76 */ 0x77, // BIT 6
        0x78, 0x79, 0x7A, 0x7B, 0x7C, 0x7D, 0x7E, 0x7F, // BIT 7
    ];
    for &opcode in known {
        assert!(
            support::decoded_elsewhere(opcode),
            "{opcode:#04X} já estava em decoded_elsewhere (como opcode não-CB)"
        );
    }
}
