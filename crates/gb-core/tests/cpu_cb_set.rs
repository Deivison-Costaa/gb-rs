//! ROADMAP 1.9f — `SET` (`CB C0`–`CB FF`).
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

fn step_set_reg(
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

// ── SET 0,B ($CB C0) — liga o bit 0 de B ───────────────────────────────────

#[test]
fn set_0_of_b_sets_bit_0() {
    let (result, cpu) = step_set_reg(
        0xC0,
        0x00,
        |cpu, v| cpu.registers.b = v,
        |cpu| cpu.registers.b,
    );
    assert!(
        cpu.is_between_instructions(),
        "SET 0,B deve completar sem lockup"
    );
    assert_eq!(result, 0x01, "SET 0,B com 0x00: bit 0 deve ser 1 → 0x01");
}

#[test]
fn set_0_of_b_leaves_other_bits_untouched() {
    let (result, cpu) = step_set_reg(
        0xC0,
        0xFA,
        |cpu, v| cpu.registers.b = v,
        |cpu| cpu.registers.b,
    );
    assert!(
        cpu.is_between_instructions(),
        "SET 0,B deve completar sem lockup"
    );
    assert_eq!(
        result, 0xFB,
        "SET 0,B com 0xFA: só o bit 0 deve ser ligado → 0xFB"
    );
}

#[test]
fn set_0_of_b_does_not_change_already_set_bit() {
    let (result, cpu) = step_set_reg(
        0xC0,
        0x01,
        |cpu, v| cpu.registers.b = v,
        |cpu| cpu.registers.b,
    );
    assert!(
        cpu.is_between_instructions(),
        "SET 0,B deve completar sem lockup"
    );
    assert_eq!(
        result, 0x01,
        "SET 0,B com 0x01 (bit 0 já é 1): valor inalterado"
    );
}

// ── SET 7,A ($CB FF) — último da faixa, bit 7 de A ─────────────────────────

#[test]
fn set_7_of_a_sets_bit_7() {
    let (result, cpu) = step_set_reg(
        0xFF,
        0x00,
        |cpu, v| cpu.registers.a = v,
        |cpu| cpu.registers.a,
    );
    assert!(
        cpu.is_between_instructions(),
        "SET 7,A deve completar sem lockup"
    );
    assert_eq!(result, 0x80, "SET 7,A com 0x00: bit 7 deve ser 1 → 0x80");
}

#[test]
fn set_7_of_a_leaves_low_bits_untouched() {
    let (result, cpu) = step_set_reg(
        0xFF,
        0x05,
        |cpu, v| cpu.registers.a = v,
        |cpu| cpu.registers.a,
    );
    assert!(
        cpu.is_between_instructions(),
        "SET 7,A deve completar sem lockup"
    );
    assert_eq!(
        result, 0x85,
        "SET 7,A com 0x05: bit 7 ligado, bits 2 e 0 preservados → 0x85"
    );
}

// ── SET 5,E ($CB EB) — meio da tabela: bit 5, registrador E ────────────────

#[test]
fn set_5_of_e_sets_bit_5() {
    let (result, cpu) = step_set_reg(
        0xEB,
        0x00,
        |cpu, v| cpu.registers.e = v,
        |cpu| cpu.registers.e,
    );
    assert!(
        cpu.is_between_instructions(),
        "SET 5,E deve completar sem lockup"
    );
    assert_eq!(result, 0x20, "SET 5,E com 0x00: bit 5 deve ser 1 → 0x20");
}

// ── SET 3,D ($CB DA) — bit 3, registrador D ────────────────────────────────

#[test]
fn set_3_of_d_leaves_other_registers_unchanged() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0xDA]); // SET 3,D
    cpu.registers.a = 0x00;
    cpu.registers.b = 0x00;
    cpu.registers.c = 0x00;
    cpu.registers.d = 0x00;
    cpu.registers.e = 0x00;
    cpu.registers.set_hl(0x0000);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(cpu.registers.a, 0x00, "SET 3,D não modifica A");
    assert_eq!(cpu.registers.b, 0x00, "SET 3,D não modifica B");
    assert_eq!(cpu.registers.c, 0x00, "SET 3,D não modifica C");
    assert_eq!(
        cpu.registers.d, 0x08,
        "SET 3,D: D deve ser 0x08 (bit 3 ligado)"
    );
    assert_eq!(cpu.registers.e, 0x00, "SET 3,D não modifica E");
    assert_eq!(cpu.registers.hl(), 0x0000, "SET 3,D não modifica HL");
}

// ── Flags totalmente preservadas ────────────────────────────────────────────

#[test]
fn set_preserves_all_flags_unchanged() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0xC0]); // SET 0,B
    cpu.registers.b = 0x00;
    cpu.registers.set_flag(Flag::Z, true);
    cpu.registers.set_flag(Flag::N, true);
    cpu.registers.set_flag(Flag::H, false);
    cpu.registers.set_flag(Flag::C, true);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert!(
        cpu.is_between_instructions(),
        "SET 0,B deve completar sem lockup"
    );
    assert_eq!(
        cpu.registers.b, 0x01,
        "SET 0,B: B deve ter sido modificado de 0x00 → 0x01"
    );
    assert!(cpu.registers.flag(Flag::Z), "SET: Z deve continuar true");
    assert!(cpu.registers.flag(Flag::N), "SET: N deve continuar true");
    assert!(!cpu.registers.flag(Flag::H), "SET: H deve continuar false");
    assert!(cpu.registers.flag(Flag::C), "SET: C deve continuar true");
}

#[test]
fn set_does_not_touch_any_flag_regardless_of_input() {
    for (opcode, byte_idx) in [
        (0xC0u8, 0u8), // SET 0,B
        (0xC9, 1u8),   // SET 1,C
        (0xD2, 2u8),   // SET 2,D
        (0xDB, 3u8),   // SET 3,E
        (0xE4, 4u8),   // SET 4,H
        (0xED, 5u8),   // SET 5,L
        (0xFF, 7u8),   // SET 7,A
    ] {
        let (mut cpu, mut bus) = machine(&[0xCB, opcode]);

        let value = 0x00u8;
        set_reg(&mut cpu, byte_idx, value);
        cpu.registers.set_flag(Flag::Z, true);
        cpu.registers.set_flag(Flag::N, true);
        cpu.registers.set_flag(Flag::H, true);
        cpu.registers.set_flag(Flag::C, true);

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert!(
            cpu.is_between_instructions(),
            "SET {byte_idx},r deve completar sem lockup"
        );

        let after = read_reg(&cpu, byte_idx);
        assert_ne!(after, 0x00, "SET deve modificar o registrador (bit ligado)");
        let expected = value | (1u8 << byte_idx);
        assert_eq!(
            after, expected,
            "SET {byte_idx},r: valor deve ser {expected:#04X}, foi {after:#04X}"
        );
        assert!(
            cpu.registers.flag(Flag::Z),
            "SET: Z deve continuar true (caso byte_idx={byte_idx})"
        );
        assert!(
            cpu.registers.flag(Flag::N),
            "SET: N deve continuar true (caso byte_idx={byte_idx})"
        );
        assert!(
            cpu.registers.flag(Flag::H),
            "SET: H deve continuar true (caso byte_idx={byte_idx})"
        );
        assert!(
            cpu.registers.flag(Flag::C),
            "SET: C deve continuar true (caso byte_idx={byte_idx})"
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

// ── SET (HL) — 4 M-cycles, read-modify-write ────────────────────────────────

#[test]
fn set_hl_sets_bit_and_writes_back_in_four_m_cycles() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0xC6, 0x00, 0x00, 0x00]); // SET 0,(HL)

    cpu.registers.set_hl(0xC000);
    bus.write(0xC000, 0x00);

    cpu.step(&mut bus); // M1: fetch $CB
    assert_eq!(cpu.registers.pc, ENTRY as u16 + 1);

    cpu.step(&mut bus); // M2: fetch opcode
    assert_eq!(cpu.registers.pc, ENTRY as u16 + 2);
    assert!(
        !cpu.is_between_instructions(),
        "depois do M2 (SET (HL)) a instrução não acabou"
    );

    cpu.step(&mut bus); // M3: read((HL))
    assert!(
        !cpu.is_between_instructions(),
        "depois do M3 (SET (HL)) a instrução não acabou"
    );

    cpu.step(&mut bus); // M4: write((HL))
    assert!(
        cpu.is_between_instructions(),
        "depois do M4 a instrução SET (HL) acabou"
    );

    assert_eq!(bus.read(0xC000), 0x01, "SET 0,(HL): memória deve ser 0x01");
}

#[test]
fn set_hl_does_not_modify_flags() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0xD6]); // SET 2,(HL)
    cpu.registers.set_hl(0xC000);
    bus.write(0xC000, 0x00);
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
        0x04,
        "SET 2,(HL) com 0x00: bit 2 ligado → 0x04"
    );
    assert!(cpu.registers.flag(Flag::Z), "SET (HL): Z intocado (true)");
    assert!(!cpu.registers.flag(Flag::N), "SET (HL): N intocado (false)");
    assert!(!cpu.registers.flag(Flag::H), "SET (HL): H intocado (false)");
    assert!(!cpu.registers.flag(Flag::C), "SET (HL): C intocado (false)");
}

#[test]
fn set_hl_one_each_bit_index() {
    for (opcode, bit_idx, value) in [
        (0xC6u8, 0u8, 0x00u8), // SET 0,(HL)
        (0xCE, 1, 0x00),       // SET 1,(HL)
        (0xD6, 2, 0x00),       // SET 2,(HL)
        (0xDE, 3, 0x00),       // SET 3,(HL)
        (0xE6, 4, 0x00),       // SET 4,(HL)
        (0xEE, 5, 0x00),       // SET 5,(HL)
        (0xF6, 6, 0x00),       // SET 6,(HL)
        (0xFE, 7, 0x00),       // SET 7,(HL)
    ] {
        let (mut cpu, mut bus) = machine(&[0xCB, opcode]);
        cpu.registers.set_hl(0xC000);
        bus.write(0xC000, value);

        cpu.step(&mut bus);
        cpu.step(&mut bus);
        cpu.step(&mut bus);
        cpu.step(&mut bus);

        let expected = 1u8 << bit_idx;
        assert_eq!(
            bus.read(0xC000),
            expected,
            "SET {bit_idx},(HL): {value:#04X} → {expected:#04X}"
        );
    }
}

// ── Controle negativo ───────────────────────────────────────────────────────
// Os 64 bytes 0xC0–0xFF têm significado não-CB que ainda não foi todo
// implementado (1.10 e 1.11). O controle negativo verifica o subconjunto
// que já está em decoded_elsewhere.

#[test]
fn set_second_bytes_c0_to_ff_known_ones_are_still_in_decoded_elsewhere() {
    let known: &[u8] = &[
        // Opcodes já decodificados como não-CB dentro da faixa 0xC0–0xFF
        0xC1, 0xC3, 0xC5, 0xC6, 0xCE, // POP BC, JP, PUSH BC, ADD A,u8, ADC A,u8
        0xD1, 0xD5, 0xD6, 0xDE, // POP DE, PUSH DE, SUB u8, SBC A,u8
        0xE0, 0xE1, 0xE2, 0xE5, 0xE6, 0xE8, 0xEA,
        0xEE, // LDH, POP/PUSH HL, AND, ADD SP, LD (u16), XOR
        0xF0, 0xF1, 0xF2, 0xF5, 0xF6, 0xF8, 0xF9, 0xFA,
        0xFE, // LDH, POP/PUSH AF, OR, LD HL,SP, LD SP,HL, LD A,(u16), CP
    ];
    for &opcode in known {
        assert!(
            support::decoded_elsewhere(opcode),
            "{opcode:#04X} já estava em decoded_elsewhere (como opcode não-CB)"
        );
    }
}
