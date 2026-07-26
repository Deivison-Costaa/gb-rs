//! ROADMAP 1.9c — prefixo CB: `SLA` (`CB 20`–`CB 27`), `SRA` (`CB 28`–`CB 2F`),
//! `SWAP` (`CB 30`–`CB 37`), `SRL` (`CB 38`–`CB 3F`). 32 opcodes sobre o mecanismo
//! da 0033. `cb_fetch` decodifica `0b00100` (SLA), `0b00101` (SRA), `0b00110` (SWAP),
//! `0b00111` (SRL). `Z` calculado, `N=0`, `H=0`. `C` = bit deslocado para SLA/SRA/SRL;
//! SWAP zera `C` incondicionalmente.

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

// ── SLA B (CB 20) — shift left arithmetic ───────────────────────────────────

#[test]
fn sla_b_shifts_left_and_sets_bit_0_to_zero_carry_from_bit_7() {
    // SLA: bit 0 ← 0, carry ← bit 7 (não circular).
    let opcode_byte = 0x20;
    for (value, expected) in [
        (0x85, 0x0A), // 1000_0101 → 0000_1010, C=1
        (0x00, 0x00), // 0000_0000 → 0000_0000, C=0
        (0x80, 0x00), // 1000_0000 → 0000_0000, C=1
        (0xFF, 0xFE), // 1111_1111 → 1111_1110, C=1
    ] {
        let (mut cpu, mut bus) = machine(&[0xCB, opcode_byte]);
        cpu.registers.b = value;
        cpu.registers.a = 0xFF;

        cpu.step(&mut bus); // M1: fetch $CB
        cpu.step(&mut bus); // M2: fetch opcode + execução

        assert_eq!(
            cpu.registers.b, expected,
            "SLA B {value:#04X}: esperado={expected:#04X}, obtido={:#04X}",
            cpu.registers.b
        );
    }
}

#[test]
fn sla_b_takes_two_m_cycles() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x20, 0x00]);
    cpu.registers.b = 0x85;
    cpu.registers.a = 0x42;

    cpu.step(&mut bus); // M1: fetch $CB
    assert!(
        !cpu.is_between_instructions(),
        "depois do M1 a instrução não acabou"
    );
    assert_eq!(cpu.registers.pc, ENTRY as u16 + 1);

    cpu.step(&mut bus); // M2: fetch opcode e executa
    assert!(
        cpu.is_between_instructions(),
        "depois do M2 a instrução acabou"
    );
    assert_eq!(cpu.registers.pc, ENTRY as u16 + 2);
    assert_eq!(cpu.registers.a, 0x42, "A não foi tocado");
}

// ── SRA B (CB 28) — shift right arithmetic, bit 7 preservado ────────────────

#[test]
fn sra_b_shifts_right_and_preserves_bit_7_carry_from_bit_0() {
    let opcode_byte = 0x28;
    for (value, expected) in [
        (0x85, 0xC2), // 1000_0101 → 1100_0010, C=1
        (0x00, 0x00), // 0000_0000 → 0000_0000, C=0
        (0x01, 0x00), // 0000_0001 → 0000_0000, C=1
        (0x80, 0xC0), // 1000_0000 → 1100_0000, C=0
    ] {
        let (mut cpu, mut bus) = machine(&[0xCB, opcode_byte]);
        cpu.registers.b = value;
        cpu.registers.a = 0xFF;

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(
            cpu.registers.b, expected,
            "SRA B {value:#04X}: esperado={expected:#04X}, obtido={:#04X}",
            cpu.registers.b
        );
    }
}

// ── SWAP B (CB 30) — troca nibbles ──────────────────────────────────────────

#[test]
fn swap_b_swaps_high_and_low_nibbles() {
    let opcode_byte = 0x30;
    for (value, expected) in [
        (0x12, 0x21), // 0001_0010 → 0010_0001
        (0x00, 0x00), // 0000_0000 → 0000_0000
        (0xF0, 0x0F), // 1111_0000 → 0000_1111
        (0xFF, 0xFF), // 1111_1111 → 1111_1111
    ] {
        let (mut cpu, mut bus) = machine(&[0xCB, opcode_byte]);
        cpu.registers.b = value;
        cpu.registers.a = 0xFF;

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(
            cpu.registers.b, expected,
            "SWAP B {value:#04X}: esperado={expected:#04X}, obtido={:#04X}",
            cpu.registers.b
        );
    }
}

// ── SRL B (CB 38) — shift right logical ─────────────────────────────────────

#[test]
fn srl_b_shifts_right_and_sets_bit_7_to_zero_carry_from_bit_0() {
    let opcode_byte = 0x38;
    for (value, expected) in [
        (0x85, 0x42), // 1000_0101 → 0100_0010, C=1
        (0x00, 0x00), // 0000_0000 → 0000_0000, C=0
        (0x01, 0x00), // 0000_0001 → 0000_0000, C=1
        (0x80, 0x40), // 1000_0000 → 0100_0000, C=0
    ] {
        let (mut cpu, mut bus) = machine(&[0xCB, opcode_byte]);
        cpu.registers.b = value;
        cpu.registers.a = 0xFF;

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(
            cpu.registers.b, expected,
            "SRL B {value:#04X}: esperado={expected:#04X}, obtido={:#04X}",
            cpu.registers.b
        );
    }
}

// ── Flags — SLA ─────────────────────────────────────────────────────────────

fn flags_sla(value: u8) -> (bool, bool, bool, bool) {
    let result = value << 1;
    let c = (value & 0x80) != 0;
    (result == 0, false, false, c)
}

#[test]
fn sla_calculates_z_and_clears_n_h_and_sets_c_from_bit_7() {
    for value in [0x00, 0x01, 0x80, 0x85, 0xFF] {
        let (mut cpu, mut bus) = machine(&[0xCB, 0x20]); // SLA B
        cpu.registers.b = value;
        cpu.registers.a = 0xAA;

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        let (expected_z, _expected_n, _expected_h, expected_c) = flags_sla(value);

        assert_eq!(
            cpu.registers.flag(Flag::Z),
            expected_z,
            "SLA B={value:#04X}: Z esperado={expected_z}"
        );
        assert!(
            !cpu.registers.flag(Flag::N),
            "SLA B={value:#04X}: N deve ser 0"
        );
        assert!(
            !cpu.registers.flag(Flag::H),
            "SLA B={value:#04X}: H deve ser 0"
        );
        assert_eq!(
            cpu.registers.flag(Flag::C),
            expected_c,
            "SLA B={value:#04X}: C esperado={expected_c}"
        );
    }
}

// ── Flags — SRA ─────────────────────────────────────────────────────────────

fn flags_sra(value: u8) -> (bool, bool, bool, bool) {
    let result = (value >> 1) | (value & 0x80);
    let c = (value & 0x01) != 0;
    (result == 0, false, false, c)
}

#[test]
fn sra_calculates_z_and_clears_n_h_and_sets_c_from_bit_0() {
    for value in [0x00, 0x01, 0x80, 0x85, 0xFF] {
        let (mut cpu, mut bus) = machine(&[0xCB, 0x28]); // SRA B
        cpu.registers.b = value;
        cpu.registers.a = 0xAA;

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        let (expected_z, _expected_n, _expected_h, expected_c) = flags_sra(value);

        assert_eq!(
            cpu.registers.flag(Flag::Z),
            expected_z,
            "SRA B={value:#04X}: Z esperado={expected_z}"
        );
        assert!(
            !cpu.registers.flag(Flag::N),
            "SRA B={value:#04X}: N deve ser 0"
        );
        assert!(
            !cpu.registers.flag(Flag::H),
            "SRA B={value:#04X}: H deve ser 0"
        );
        assert_eq!(
            cpu.registers.flag(Flag::C),
            expected_c,
            "SRA B={value:#04X}: C esperado={expected_c}"
        );
    }
}

// ── Flags — SWAP ────────────────────────────────────────────────────────────

fn flags_swap(value: u8) -> (bool, bool, bool, bool) {
    let result = value.rotate_right(4);
    (result == 0, false, false, false)
}

#[test]
fn swap_calculates_z_and_clears_n_h_and_always_clears_c() {
    for value in [0x00, 0x01, 0x12, 0xF0, 0xFF] {
        let (mut cpu, mut bus) = machine(&[0xCB, 0x30]); // SWAP B
        cpu.registers.b = value;
        // suja C para garantir que é zerado
        cpu.registers.set_flag(Flag::C, true);
        cpu.registers.a = 0xAA;

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        let (expected_z, _expected_n, _expected_h, expected_c) = flags_swap(value);

        assert_eq!(
            cpu.registers.flag(Flag::Z),
            expected_z,
            "SWAP B={value:#04X}: Z esperado={expected_z}"
        );
        assert!(
            !cpu.registers.flag(Flag::N),
            "SWAP B={value:#04X}: N deve ser 0"
        );
        assert!(
            !cpu.registers.flag(Flag::H),
            "SWAP B={value:#04X}: H deve ser 0"
        );
        assert_eq!(
            cpu.registers.flag(Flag::C),
            expected_c,
            "SWAP B={value:#04X}: C deve ser sempre 0"
        );
    }
}

// ── Flags — SRL ─────────────────────────────────────────────────────────────

fn flags_srl(value: u8) -> (bool, bool, bool, bool) {
    let result = value >> 1;
    let c = (value & 0x01) != 0;
    (result == 0, false, false, c)
}

#[test]
fn srl_calculates_z_and_clears_n_h_and_sets_c_from_bit_0() {
    for value in [0x00, 0x01, 0x80, 0x85, 0xFF] {
        let (mut cpu, mut bus) = machine(&[0xCB, 0x38]); // SRL B
        cpu.registers.b = value;
        cpu.registers.a = 0xAA;

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        let (expected_z, _expected_n, _expected_h, expected_c) = flags_srl(value);

        assert_eq!(
            cpu.registers.flag(Flag::Z),
            expected_z,
            "SRL B={value:#04X}: Z esperado={expected_z}"
        );
        assert!(
            !cpu.registers.flag(Flag::N),
            "SRL B={value:#04X}: N deve ser 0"
        );
        assert!(
            !cpu.registers.flag(Flag::H),
            "SRL B={value:#04X}: H deve ser 0"
        );
        assert_eq!(
            cpu.registers.flag(Flag::C),
            expected_c,
            "SRL B={value:#04X}: C esperado={expected_c}"
        );
    }
}

// ── Isolamento — toca só o registrador destino ──────────────────────────────

#[test]
fn sla_b_touches_only_b_and_flags() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x20]);
    cpu.registers.b = 0x85;
    cpu.registers.a = 0x42;
    cpu.registers.c = 0x01;
    cpu.registers.d = 0x02;
    cpu.registers.e = 0x03;
    let hl_before = cpu.registers.hl();
    let sp_before = cpu.registers.sp;

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(cpu.registers.b, 0x0A);
    assert_eq!(cpu.registers.a, 0x42);
    assert_eq!(cpu.registers.c, 0x01);
    assert_eq!(cpu.registers.d, 0x02);
    assert_eq!(cpu.registers.e, 0x03);
    assert_eq!(cpu.registers.hl(), hl_before);
    assert_eq!(cpu.registers.sp, sp_before);
}

#[test]
fn swap_b_touches_only_b_and_flags() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x30]);
    cpu.registers.b = 0x12;
    cpu.registers.a = 0x42;
    cpu.registers.c = 0x01;
    cpu.registers.d = 0x02;
    cpu.registers.e = 0x03;
    let hl_before = cpu.registers.hl();
    let sp_before = cpu.registers.sp;

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(cpu.registers.b, 0x21);
    assert_eq!(cpu.registers.a, 0x42);
    assert_eq!(cpu.registers.c, 0x01);
    assert_eq!(cpu.registers.d, 0x02);
    assert_eq!(cpu.registers.e, 0x03);
    assert_eq!(cpu.registers.hl(), hl_before);
    assert_eq!(cpu.registers.sp, sp_before);
}

// ── (HL) — read-modify-write, 4 M-cycles ────────────────────────────────────

#[test]
fn sla_hl_reads_modifies_and_writes_in_four_m_cycles() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x26, 0x00, 0x00, 0x00]);
    cpu.registers.set_hl(0xC000);
    bus.write(0xC000, 0x85);

    cpu.step(&mut bus); // M1: fetch $CB
    assert_eq!(cpu.registers.pc, ENTRY as u16 + 1);

    cpu.step(&mut bus); // M2: fetch opcode
    assert_eq!(cpu.registers.pc, ENTRY as u16 + 2);

    cpu.step(&mut bus); // M3: read((HL))
    assert!(!cpu.is_between_instructions());

    cpu.step(&mut bus); // M4: write((HL))
    assert!(cpu.is_between_instructions());

    let result = bus.read(0xC000);
    assert_eq!(
        result, 0x0A,
        "SLA (HL) com valor 0x85 → 0x0A, obtido {result:#04X}"
    );
}

#[test]
fn sla_hl_does_not_change_hl_itself() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x26]);
    cpu.registers.set_hl(0xC000);
    bus.write(0xC000, 0x01);

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(cpu.registers.hl(), 0xC000, "SLA (HL) não muda HL");
}

#[test]
fn sra_hl_reads_modifies_and_writes_in_four_m_cycles() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x2E, 0x00, 0x00, 0x00]);
    cpu.registers.set_hl(0xC000);
    bus.write(0xC000, 0x85);

    cpu.step(&mut bus); // M1
    cpu.step(&mut bus); // M2
    cpu.step(&mut bus); // M3
    cpu.step(&mut bus); // M4

    let result = bus.read(0xC000);
    assert_eq!(
        result, 0xC2,
        "SRA (HL) com valor 0x85 → 0xC2, obtido {result:#04X}"
    );
}

#[test]
fn swap_hl_reads_modifies_and_writes_in_four_m_cycles() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x36, 0x00, 0x00, 0x00]);
    cpu.registers.set_hl(0xC000);
    bus.write(0xC000, 0x12);

    cpu.step(&mut bus); // M1
    cpu.step(&mut bus); // M2
    cpu.step(&mut bus); // M3
    cpu.step(&mut bus); // M4

    let result = bus.read(0xC000);
    assert_eq!(
        result, 0x21,
        "SWAP (HL) com valor 0x12 → 0x21, obtido {result:#04X}"
    );
}

#[test]
fn swap_hl_clears_carry_flag() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x36]);
    cpu.registers.set_hl(0xC000);
    bus.write(0xC000, 0x12);
    cpu.registers.set_flag(Flag::C, true);

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert!(!cpu.registers.flag(Flag::C), "SWAP (HL) deve zerar C");
}

#[test]
fn srl_hl_reads_modifies_and_writes_in_four_m_cycles() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x3E, 0x00, 0x00, 0x00]);
    cpu.registers.set_hl(0xC000);
    bus.write(0xC000, 0x85);

    cpu.step(&mut bus); // M1
    cpu.step(&mut bus); // M2
    cpu.step(&mut bus); // M3
    cpu.step(&mut bus); // M4

    let result = bus.read(0xC000);
    assert_eq!(
        result, 0x42,
        "SRL (HL) com valor 0x85 → 0x42, obtido {result:#04X}"
    );
}

// ── Flags do (HL) ───────────────────────────────────────────────────────────

#[test]
fn sla_hl_sets_z_flag_when_result_is_zero() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x26]);
    cpu.registers.set_hl(0xC000);
    bus.write(0xC000, 0x80); // 1000_0000 → SLA → 0000_0000

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert!(cpu.registers.flag(Flag::Z), "SLA (HL) 0x80 → Z=1");
    assert!(!cpu.registers.flag(Flag::N));
    assert!(!cpu.registers.flag(Flag::H));
    assert!(cpu.registers.flag(Flag::C));
}

// ── SWAP zera C mesmo com carry de entrada sujo ─────────────────────────────

#[test]
fn swap_clears_c_flag_even_when_carry_was_set() {
    for value in [0x12, 0x00, 0xFF] {
        let (mut cpu, mut bus) = machine(&[0xCB, 0x37]); // SWAP A
        cpu.registers.a = value;
        cpu.registers.set_flag(Flag::C, true);
        cpu.registers.set_flag(Flag::Z, value != 0x00);

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        let expected_result = value.rotate_right(4);
        assert_eq!(
            cpu.registers.a, expected_result,
            "SWAP A {value:#04X}: resultado"
        );
        assert_eq!(
            cpu.registers.flag(Flag::Z),
            expected_result == 0,
            "SWAP A {value:#04X}: Z"
        );
        assert!(!cpu.registers.flag(Flag::N), "SWAP A {value:#04X}: N");
        assert!(!cpu.registers.flag(Flag::H), "SWAP A {value:#04X}: H");
        assert!(
            !cpu.registers.flag(Flag::C),
            "SWAP A {value:#04X}: C zerado sempre"
        );
    }
}

// ── Multi-registro: confirma que todos os campos de r8 funcionam ───────────

fn set_b(cpu: &mut Cpu, v: u8) {
    cpu.registers.b = v;
}
fn set_c(cpu: &mut Cpu, v: u8) {
    cpu.registers.c = v;
}
fn set_d(cpu: &mut Cpu, v: u8) {
    cpu.registers.d = v;
}
fn set_e(cpu: &mut Cpu, v: u8) {
    cpu.registers.e = v;
}
fn set_h(cpu: &mut Cpu, v: u8) {
    cpu.registers.h = v;
}
fn set_l(cpu: &mut Cpu, v: u8) {
    cpu.registers.l = v;
}
fn set_a(cpu: &mut Cpu, v: u8) {
    cpu.registers.a = v;
}

fn get_b(cpu: &Cpu) -> u8 {
    cpu.registers.b
}
fn get_c(cpu: &Cpu) -> u8 {
    cpu.registers.c
}
fn get_d(cpu: &Cpu) -> u8 {
    cpu.registers.d
}
fn get_e(cpu: &Cpu) -> u8 {
    cpu.registers.e
}
fn get_h(cpu: &Cpu) -> u8 {
    cpu.registers.h
}
fn get_l(cpu: &Cpu) -> u8 {
    cpu.registers.l
}
fn get_a(cpu: &Cpu) -> u8 {
    cpu.registers.a
}

#[test]
#[allow(clippy::type_complexity)]
fn sla_all_registers_produce_correct_result() {
    let opcodes: [(u8, fn(&mut Cpu, u8), fn(&Cpu) -> u8, bool); 7] = [
        (0x20, set_b, get_b, false),
        (0x21, set_c, get_c, false),
        (0x22, set_d, get_d, false),
        (0x23, set_e, get_e, false),
        (0x24, set_h, get_h, false),
        (0x25, set_l, get_l, false),
        (0x27, set_a, get_a, true),
    ];
    // 0x85 → SLA → 0x0A
    for (opcode, set, get, is_a) in opcodes {
        let (mut cpu, mut bus) = machine(&[0xCB, opcode]);
        set(&mut cpu, 0x85);
        if !is_a {
            cpu.registers.a = 0xFF;
        }

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(get(&cpu), 0x0A, "SLA r8 opcode {opcode:#04X}: 0x85 → 0x0A");
        assert!(cpu.registers.flag(Flag::C), "SLA opcode {opcode:#04X}: C=1");
    }
}

#[test]
#[allow(clippy::type_complexity)]
fn swap_all_registers_produce_correct_result() {
    let opcodes: [(u8, fn(&mut Cpu, u8), fn(&Cpu) -> u8, bool); 7] = [
        (0x30, set_b, get_b, false),
        (0x31, set_c, get_c, false),
        (0x32, set_d, get_d, false),
        (0x33, set_e, get_e, false),
        (0x34, set_h, get_h, false),
        (0x35, set_l, get_l, false),
        (0x37, set_a, get_a, true),
    ];
    for (opcode, set, get, is_a) in opcodes {
        let (mut cpu, mut bus) = machine(&[0xCB, opcode]);
        set(&mut cpu, 0x12);
        if !is_a {
            cpu.registers.a = 0xFF;
        }
        cpu.registers.set_flag(Flag::C, true);

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(get(&cpu), 0x21, "SWAP r8 opcode {opcode:#04X}: 0x12 → 0x21");
        assert!(
            !cpu.registers.flag(Flag::C),
            "SWAP opcode {opcode:#04X}: C=0 sempre"
        );
    }
}

// ── Controle negativo — bytes do CB 20-3F ──────────────────────────────────

#[test]
fn cb_second_bytes_20_to_3f_known_ones_are_still_in_decoded_elsewhere() {
    // Dos 32 bytes (CB 20-3F), só alguns já estavam em decoded_elsewhere como
    // opcodes não-CB. Os demais (JR condicional, DAA, CPL, SCF, CCF) ainda não
    // estão implementados e portanto NÃO estão em decoded_elsewhere.
    let known: &[u8] = &[
        // SLA: 0x20 (JR NZ) e 0x27 (DAA) não estão
        0x21, 0x22, 0x23, 0x24, 0x25, 0x26, // SRA: 0x28 (JR Z) e 0x2F (CPL) não estão
        0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E,
        // SWAP: 0x30 (JR NC) e 0x37 (SCF) não estão
        0x31, 0x32, 0x33, 0x34, 0x35, 0x36, // SRL: 0x38 (JR C) e 0x3F (CCF) não estão
        0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E,
    ];
    for &opcode in known {
        assert!(
            support::decoded_elsewhere(opcode),
            "{opcode:#04X} já estava em decoded_elsewhere (como opcode não-CB)"
        );
    }
}
