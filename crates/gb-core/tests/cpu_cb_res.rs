//! ROADMAP 1.9e — `RES` (`CB 80`–`CB BF`).
//! Dois M-cycles para registrador (8 T-cycles), quatro para `(HL)` (16 T-cycles).
//! Sem flags alteradas (`Z N H C` intocados). `(HL)` é read-modify-write.

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

fn step_res_reg(
    opcode: u8,
    value: u8,
    set_fn: fn(&mut Cpu, u8),
    read_fn: fn(&Cpu) -> u8,
) -> (u8, Cpu) {
    let (mut cpu, mut bus) = machine(&[0xCB, opcode]);

    set_fn(&mut cpu, value);

    cpu.step(&mut bus); // M1: fetch $CB
    cpu.step(&mut bus); // M2: fetch opcode + execução

    let result = read_fn(&cpu);
    (result, cpu)
}

// ── RES 0,B ($CB 80) — zera o bit 0 de B ──────────────────────────────────

#[test]
fn res_0_of_b_clears_bit_0() {
    let (result, cpu) = step_res_reg(
        0x80,
        0xFF,
        |cpu, v| cpu.registers.b = v,
        |cpu| cpu.registers.b,
    );
    assert!(
        cpu.is_between_instructions(),
        "RES 0,B deve completar sem lockup"
    );
    assert_eq!(result, 0xFE, "RES 0,B com 0xFF: bit 0 deve ser zero → 0xFE");
}

#[test]
fn res_0_of_b_leaves_other_bits_untouched() {
    let (result, cpu) = step_res_reg(
        0x80,
        0x05,
        |cpu, v| cpu.registers.b = v,
        |cpu| cpu.registers.b,
    );
    assert!(
        cpu.is_between_instructions(),
        "RES 0,B deve completar sem lockup"
    );
    assert_eq!(
        result, 0x04,
        "RES 0,B com 0x05: só o bit 0 deve ser zerado → 0x04"
    );
}

#[test]
fn res_0_of_b_does_not_change_already_zero_bit() {
    let (result, cpu) = step_res_reg(
        0x80,
        0xFE,
        |cpu, v| cpu.registers.b = v,
        |cpu| cpu.registers.b,
    );
    assert!(
        cpu.is_between_instructions(),
        "RES 0,B deve completar sem lockup"
    );
    assert_eq!(
        result, 0xFE,
        "RES 0,B com 0xFE (bit 0 já é 0): valor inalterado"
    );
}

// ── RES 7,A ($CB BF) — último da faixa, bit 7 de A ────────────────────────

#[test]
fn res_7_of_a_clears_bit_7() {
    let (result, cpu) = step_res_reg(
        0xBF,
        0xFF,
        |cpu, v| cpu.registers.a = v,
        |cpu| cpu.registers.a,
    );
    assert!(
        cpu.is_between_instructions(),
        "RES 7,A deve completar sem lockup"
    );
    assert_eq!(result, 0x7F, "RES 7,A com 0xFF: bit 7 deve ser zero → 0x7F");
}

#[test]
fn res_7_of_a_leaves_low_bits_untouched() {
    let (result, cpu) = step_res_reg(
        0xBF,
        0x85,
        |cpu, v| cpu.registers.a = v,
        |cpu| cpu.registers.a,
    );
    assert!(
        cpu.is_between_instructions(),
        "RES 7,A deve completar sem lockup"
    );
    assert_eq!(
        result, 0x05,
        "RES 7,A com 0x85: bit 7 zerado, bits 2 e 0 preservados → 0x05"
    );
}

// ── RES 5,E ($CB AB) — meio da tabela: bit 5, registrador E ───────────────

#[test]
fn res_5_of_e_clears_bit_5() {
    let (result, cpu) = step_res_reg(
        0xAB,
        0xFF,
        |cpu, v| cpu.registers.e = v,
        |cpu| cpu.registers.e,
    );
    assert!(
        cpu.is_between_instructions(),
        "RES 5,E deve completar sem lockup"
    );
    assert_eq!(result, 0xDF, "RES 5,E com 0xFF: bit 5 deve ser zero → 0xDF");
}

// ── RES 3,D ($CB 9A) — bit 3, registrador D ───────────────────────────────

#[test]
fn res_3_of_d_leaves_other_registers_unchanged() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x9A]); // RES 3,D
    cpu.registers.a = 0xA5;
    cpu.registers.b = 0x7B;
    cpu.registers.c = 0x3C;
    cpu.registers.d = 0xFF;
    cpu.registers.e = 0x91;
    cpu.registers.set_hl(0x1234);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(cpu.registers.a, 0xA5, "RES 3,D não modifica A");
    assert_eq!(cpu.registers.b, 0x7B, "RES 3,D não modifica B");
    assert_eq!(cpu.registers.c, 0x3C, "RES 3,D não modifica C");
    assert_eq!(
        cpu.registers.d, 0xF7,
        "RES 3,D: D deve ser 0xF7 (bit 3 zerado)"
    );
    assert_eq!(cpu.registers.e, 0x91, "RES 3,D não modifica E");
    assert_eq!(cpu.registers.hl(), 0x1234, "RES 3,D não modifica HL");
}

// ── Flags totalmente preservadas ───────────────────────────────────────────

#[test]
fn res_preserves_all_flags_unchanged() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x80]); // RES 0,B
    cpu.registers.b = 0xFF;
    cpu.registers.set_flag(Flag::Z, true);
    cpu.registers.set_flag(Flag::N, true);
    cpu.registers.set_flag(Flag::H, false);
    cpu.registers.set_flag(Flag::C, true);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert!(
        cpu.is_between_instructions(),
        "RES 0,B deve completar sem lockup"
    );
    assert_eq!(
        cpu.registers.b, 0xFE,
        "RES 0,B: B deve ter sido modificado de 0xFF → 0xFE"
    );
    assert!(cpu.registers.flag(Flag::Z), "RES: Z deve continuar true");
    assert!(cpu.registers.flag(Flag::N), "RES: N deve continuar true");
    assert!(!cpu.registers.flag(Flag::H), "RES: H deve continuar false");
    assert!(cpu.registers.flag(Flag::C), "RES: C deve continuar true");
}

#[test]
fn res_does_not_touch_any_flag_regardless_of_input() {
    for (opcode, byte_idx) in [
        (0x80, 0u8), // RES 0,B
        (0x89, 1u8), // RES 1,C
        (0x92, 2u8), // RES 2,D
        (0x9B, 3u8), // RES 3,E
        (0xA4, 4u8), // RES 4,H
        (0xAD, 5u8), // RES 5,L
        (0xBF, 7u8), // RES 7,A
    ] {
        let (mut cpu, mut bus) = machine(&[0xCB, opcode]);

        let value = 0xFFu8;
        set_reg(&mut cpu, byte_idx, value);
        cpu.registers.set_flag(Flag::Z, true);
        cpu.registers.set_flag(Flag::N, true);
        cpu.registers.set_flag(Flag::H, true);
        cpu.registers.set_flag(Flag::C, true);

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert!(
            cpu.is_between_instructions(),
            "RES {byte_idx},r deve completar sem lockup"
        );

        let after = read_reg(&cpu, byte_idx);
        assert_ne!(after, 0xFF, "RES deve modificar o registrador (bit zerado)");
        let expected = value & !(1u8 << byte_idx);
        assert_eq!(
            after, expected,
            "RES {byte_idx},r: valor deve ser {expected:#04X}, foi {after:#04X}"
        );
        assert!(
            cpu.registers.flag(Flag::Z),
            "RES: Z deve continuar true (caso byte_idx={byte_idx})"
        );
        assert!(
            cpu.registers.flag(Flag::N),
            "RES: N deve continuar true (caso byte_idx={byte_idx})"
        );
        assert!(
            cpu.registers.flag(Flag::H),
            "RES: H deve continuar true (caso byte_idx={byte_idx})"
        );
        assert!(
            cpu.registers.flag(Flag::C),
            "RES: C deve continuar true (caso byte_idx={byte_idx})"
        );
    }
}

fn set_reg(cpu: &mut Cpu, byte_idx: u8, value: u8) {
    match byte_idx {
        0 => cpu.registers.b = value,
        1 => cpu.registers.c = value,
        2 => cpu.registers.d = value,
        3 => cpu.registers.e = value,
        4 => cpu.registers.h = value,
        5 => cpu.registers.l = value,
        _ => cpu.registers.a = value,
    }
}

fn read_reg(cpu: &Cpu, byte_idx: u8) -> u8 {
    match byte_idx {
        0 => cpu.registers.b,
        1 => cpu.registers.c,
        2 => cpu.registers.d,
        3 => cpu.registers.e,
        4 => cpu.registers.h,
        5 => cpu.registers.l,
        _ => cpu.registers.a,
    }
}

// ── RES (HL) — 4 M-cycles, read-modify-write ───────────────────────────────

#[test]
fn res_hl_clears_bit_and_writes_back_in_four_m_cycles() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x86, 0x00, 0x00, 0x00]); // RES 0,(HL)

    cpu.registers.set_hl(0xC000);
    bus.write(0xC000, 0xFF);

    cpu.step(&mut bus); // M1: fetch $CB
    assert_eq!(cpu.registers.pc, ENTRY as u16 + 1);

    cpu.step(&mut bus); // M2: fetch opcode
    assert_eq!(cpu.registers.pc, ENTRY as u16 + 2);
    assert!(
        !cpu.is_between_instructions(),
        "depois do M2 (RES (HL)) a instrução não acabou"
    );

    cpu.step(&mut bus); // M3: read((HL))
    assert!(
        !cpu.is_between_instructions(),
        "depois do M3 (RES (HL)) a instrução não acabou"
    );

    cpu.step(&mut bus); // M4: write((HL))
    assert!(
        cpu.is_between_instructions(),
        "depois do M4 a instrução RES (HL) acabou"
    );

    assert_eq!(bus.read(0xC000), 0xFE, "RES 0,(HL): memória deve ser 0xFE");
}

#[test]
fn res_hl_does_not_modify_flags() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x96]); // RES 2,(HL)
    cpu.registers.set_hl(0xC000);
    bus.write(0xC000, 0x04); // bit 2 = 1
    cpu.registers.set_flag(Flag::Z, true);
    cpu.registers.set_flag(Flag::N, false);
    cpu.registers.set_flag(Flag::H, false);
    cpu.registers.set_flag(Flag::C, false);

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        bus.read(0xC000),
        0x00,
        "RES 2,(HL) com 0x04: bit 2 zerado → 0x00"
    );
    assert!(cpu.registers.flag(Flag::Z), "RES (HL): Z intocado (true)");
    assert!(!cpu.registers.flag(Flag::N), "RES (HL): N intocado (false)");
    assert!(!cpu.registers.flag(Flag::H), "RES (HL): H intocado (false)");
    assert!(!cpu.registers.flag(Flag::C), "RES (HL): C intocado (false)");
}

#[test]
fn res_hl_one_each_bit_index() {
    for (opcode, bit_idx, value) in [
        (0x86u8, 0u8, 0xFFu8), // RES 0,(HL)
        (0x8E, 1, 0xFF),       // RES 1,(HL)
        (0x96, 2, 0xFF),       // RES 2,(HL)
        (0x9E, 3, 0xFF),       // RES 3,(HL)
        (0xA6, 4, 0xFF),       // RES 4,(HL)
        (0xAE, 5, 0xFF),       // RES 5,(HL)
        (0xB6, 6, 0xFF),       // RES 6,(HL)
        (0xBE, 7, 0xFF),       // RES 7,(HL)
    ] {
        let (mut cpu, mut bus) = machine(&[0xCB, opcode]);
        cpu.registers.set_hl(0xC000);
        bus.write(0xC000, value);

        cpu.step(&mut bus);
        cpu.step(&mut bus);
        cpu.step(&mut bus);
        cpu.step(&mut bus);

        let expected = !(1u8 << bit_idx);
        assert_eq!(
            bus.read(0xC000),
            expected,
            "RES {bit_idx},(HL): {value:#04X} → {expected:#04X}"
        );
    }
}

// ── Controle negativo ──────────────────────────────────────────────────────

#[test]
fn res_second_bytes_80_to_bf_known_ones_are_still_in_decoded_elsewhere() {
    let known: &[u8] = &[
        0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, // RES 0
        0x88, 0x89, 0x8A, 0x8B, 0x8C, 0x8D, 0x8E, 0x8F, // RES 1
        0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, // RES 2
        0x98, 0x99, 0x9A, 0x9B, 0x9C, 0x9D, 0x9E, 0x9F, // RES 3
        0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7, // RES 4
        0xA8, 0xA9, 0xAA, 0xAB, 0xAC, 0xAD, 0xAE, 0xAF, // RES 5
        0xB0, 0xB1, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7, // RES 6
        0xB8, 0xB9, 0xBA, 0xBB, 0xBC, 0xBD, 0xBE, 0xBF, // RES 7
    ];
    for &opcode in known {
        assert!(
            support::decoded_elsewhere(opcode),
            "{opcode:#04X} já estava em decoded_elsewhere (como opcode não-CB)"
        );
    }
}
