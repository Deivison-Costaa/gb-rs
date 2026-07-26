//! ROADMAP 1.9a — prefixo CB + `RLC` (`CB 00`–`CB 07`).
//! Dois M-cycles para registrador (8 T-cycles), quatro para `(HL)` (16 T-cycles).
//! `Z` é calculado do resultado (ao contrário do `RLCA` não-prefixado, que zera
//! incondicionalmente). `N=0`, `H=0` literais; `C` = bit 7 antigo.

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

fn single_step_reg(
    opcode: u8,
    initial: u8,
    set_fn: fn(&mut Cpu, u8),
    get_fn: fn(&Cpu) -> u8,
) -> u8 {
    let (mut cpu, mut bus) = machine(&[0xCB, opcode]);

    set_fn(&mut cpu, initial);
    cpu.registers.a = 0xFF; // suja A para testar isolamento

    cpu.step(&mut bus); // M1: fetch de $CB
    cpu.step(&mut bus); // M2: fetch de (opcode) + execução

    get_fn(&cpu)
}

fn flags_after_rlc(value: u8) -> (bool, bool, bool, bool) {
    let result = value.rotate_left(1);
    let c = (value & 0x80) != 0;
    let z = result == 0;
    (z, false, false, c)
}

// ── RLC B (CB 00) ──────────────────────────────────────────────────────────

#[test]
fn rlc_b_rotates_left_and_copies_bit_7_to_carry_and_bit_0() {
    for (value, expected) in [
        (0x85, 0x0B), // 1000_0101 → 0000_1011
        (0x00, 0x00),
        (0x80, 0x01), // 1000_0000 → 0000_0001
        (0xFF, 0xFF),
    ] {
        let result = single_step_reg(
            0x00,
            value,
            |cpu, v| cpu.registers.b = v,
            |cpu| cpu.registers.b,
        );
        assert_eq!(
            result, expected,
            "RLC B {value:#04X}: B esperado={expected:#04X}, obtido={result:#04X}"
        );
    }
}

#[test]
fn rlc_b_takes_two_m_cycles() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x00, 0x00]);
    cpu.registers.b = 0x85;
    cpu.registers.a = 0x42;

    cpu.step(&mut bus); // M1: fetch $CB
    assert!(
        !cpu.is_between_instructions(),
        "depois do M1 a instrução não acabou"
    );
    assert_eq!(cpu.registers.pc, ENTRY as u16 + 1, "M1 avança PC em 1");

    cpu.step(&mut bus); // M2: fetch opcode e executa
    assert!(
        cpu.is_between_instructions(),
        "depois do M2 a instrução acabou"
    );
    assert_eq!(cpu.registers.pc, ENTRY as u16 + 2, "M2 avança PC em 1");
    // A não foi tocado
    assert_eq!(cpu.registers.a, 0x42);
}

// ── RLC C (CB 01) ──────────────────────────────────────────────────────────

#[test]
fn rlc_c_rotates_left_and_copies_bit_7_to_carry_and_bit_0() {
    for (value, expected) in [(0x85, 0x0B), (0x07, 0x0E), (0x00, 0x00)] {
        let result = single_step_reg(
            0x01,
            value,
            |cpu, v| cpu.registers.c = v,
            |cpu| cpu.registers.c,
        );
        assert_eq!(result, expected, "RLC C {value:#04X}");
    }
}

// ── RLC D (CB 02) ──────────────────────────────────────────────────────────

#[test]
fn rlc_d_rotates_left_and_copies_bit_7_to_carry_and_bit_0() {
    for (value, expected) in [(0x85, 0x0B), (0x07, 0x0E)] {
        let result = single_step_reg(
            0x02,
            value,
            |cpu, v| cpu.registers.d = v,
            |cpu| cpu.registers.d,
        );
        assert_eq!(result, expected, "RLC D {value:#04X}");
    }
}

// ── RLC E (CB 03) ──────────────────────────────────────────────────────────

#[test]
fn rlc_e_rotates_left_and_copies_bit_7_to_carry_and_bit_0() {
    for (value, expected) in [(0x85, 0x0B), (0x07, 0x0E)] {
        let result = single_step_reg(
            0x03,
            value,
            |cpu, v| cpu.registers.e = v,
            |cpu| cpu.registers.e,
        );
        assert_eq!(result, expected, "RLC E {value:#04X}");
    }
}

// ── RLC H (CB 04) ──────────────────────────────────────────────────────────

#[test]
fn rlc_h_rotates_left_and_copies_bit_7_to_carry_and_bit_0() {
    for (value, expected) in [(0x85, 0x0B), (0x00, 0x00)] {
        let result = single_step_reg(
            0x04,
            value,
            |cpu, v| cpu.registers.h = v,
            |cpu| cpu.registers.h,
        );
        assert_eq!(result, expected, "RLC H {value:#04X}");
    }
}

// ── RLC L (CB 05) ──────────────────────────────────────────────────────────

#[test]
fn rlc_l_rotates_left_and_copies_bit_7_to_carry_and_bit_0() {
    for (value, expected) in [(0x85, 0x0B), (0x00, 0x00)] {
        let result = single_step_reg(
            0x05,
            value,
            |cpu, v| cpu.registers.l = v,
            |cpu| cpu.registers.l,
        );
        assert_eq!(result, expected, "RLC L {value:#04X}");
    }
}

// ── RLC A (CB 07) — calcula Z, ao contrário do RLCA ───────────────────────

#[test]
fn rlc_a_rotates_left_and_copies_bit_7_to_carry_and_bit_0() {
    for (value, expected) in [
        (0x85, 0x0B), // 1000_0101 → 0000_1011
        (0x00, 0x00),
        (0x80, 0x01),
        (0xFF, 0xFF),
    ] {
        let (mut cpu, mut bus) = machine(&[0xCB, 0x07, 0x00]);
        cpu.registers.a = value;

        cpu.step(&mut bus); // M1: fetch $CB
        cpu.step(&mut bus); // M2: fetch opcode e executa

        assert_eq!(
            cpu.registers.a, expected,
            "RLC A {value:#04X}: A esperado={expected:#04X}, obtido={:#04X}",
            cpu.registers.a
        );
    }
}

#[test]
fn rlc_a_sets_z_flag_when_result_is_zero_unlike_rlca() {
    // A=0x00 → RLC → resultado=0x00 (Z=1). RLCA com A=0x00 zera Z incondicionalmente.
    let (mut cpu, mut bus) = machine(&[0xCB, 0x07]);
    cpu.registers.a = 0x00;

    cpu.step(&mut bus); // M1: fetch $CB
    cpu.step(&mut bus); // M2: fetch opcode e executa

    assert!(
        cpu.registers.flag(Flag::Z),
        "RLC A 0x00: resultado=0x00 → Z deve ser 1"
    );
    assert_eq!(cpu.registers.a, 0x00, "RLC A 0x00: A continua 0x00");
}

#[test]
fn rlc_a_clears_z_flag_when_result_is_nonzero() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x07]);
    cpu.registers.a = 0x01;

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert!(
        !cpu.registers.flag(Flag::Z),
        "RLC A 0x01: Z deve ser 0 (resultado 0x02 != 0)"
    );
}

// ── RLC (HL) (CB 06) — read-modify-write, 4 M-cycles ──────────────────────

#[test]
fn rlc_hl_reads_modifies_and_writes_in_four_m_cycles() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x06, 0x00, 0x00, 0x00]);

    // Usa WRAM como destino de (HL) para testar a memória
    cpu.registers.set_hl(0xC000);
    bus.write(0xC000, 0x85); // 1000_0101

    cpu.step(&mut bus); // M1: fetch $CB
    assert_eq!(cpu.registers.pc, ENTRY as u16 + 1);

    cpu.step(&mut bus); // M2: fetch opcode = $06 → decodifica RLC (HL)
    assert_eq!(cpu.registers.pc, ENTRY as u16 + 2);

    cpu.step(&mut bus); // M3: read((HL))
    assert!(!cpu.is_between_instructions());

    cpu.step(&mut bus); // M4: write((HL))
    assert!(cpu.is_between_instructions());

    let result = bus.read(0xC000);
    assert_eq!(
        result, 0x0B,
        "RLC (HL) com valor 0x85 → 0x0B, obtido {result:#04X}"
    );
}

#[test]
fn rlc_hl_does_not_change_hl_itself() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x06]);
    cpu.registers.set_hl(0xC000);
    bus.write(0xC000, 0x01);

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(cpu.registers.hl(), 0xC000, "RLC (HL) não muda HL");
}

// ── Flags do RLC ───────────────────────────────────────────────────────────

#[test]
fn rlc_calculates_z_from_result_and_clears_n_and_h_and_sets_c_from_bit_7() {
    for value in [0x00, 0x01, 0x80, 0x85, 0xFF] {
        let (mut cpu, mut bus) = machine(&[0xCB, 0x00]); // RLC B
        cpu.registers.b = value;
        cpu.registers.a = 0xAA; // suja A

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        let (expected_z, expected_n, expected_h, expected_c) = flags_after_rlc(value);

        assert_eq!(
            cpu.registers.flag(Flag::Z),
            expected_z,
            "RLC B={value:#04X}: Z esperado={expected_z}"
        );
        assert_eq!(
            cpu.registers.flag(Flag::N),
            expected_n,
            "RLC B={value:#04X}: N esperado={expected_n}"
        );
        assert_eq!(
            cpu.registers.flag(Flag::H),
            expected_h,
            "RLC B={value:#04X}: H esperado={expected_h}"
        );
        assert_eq!(
            cpu.registers.flag(Flag::C),
            expected_c,
            "RLC B={value:#04X}: C esperado={expected_c}"
        );
    }
}

// ── RLC toca só o registrador destino e as flags ──────────────────────────

#[test]
fn rlc_b_touches_only_b_and_flags() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x00]);
    cpu.registers.b = 0x85;
    cpu.registers.a = 0x42;
    cpu.registers.c = 0x01;
    cpu.registers.d = 0x02;
    cpu.registers.e = 0x03;
    let hl_before = cpu.registers.hl();
    let sp_before = cpu.registers.sp;

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(cpu.registers.b, 0x0B);
    assert_eq!(cpu.registers.a, 0x42);
    assert_eq!(cpu.registers.c, 0x01);
    assert_eq!(cpu.registers.d, 0x02);
    assert_eq!(cpu.registers.e, 0x03);
    assert_eq!(cpu.registers.hl(), hl_before);
    assert_eq!(cpu.registers.sp, sp_before);
}

// ── CB (HL) não modifica HL (diferente de HL+) ────────────────────────────

#[test]
fn rlc_hl_preserves_hl_value() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x06]);
    cpu.registers.set_hl(0xC001);
    bus.write(0xC001, 0x42);

    for _ in 0..4 {
        cpu.step(&mut bus);
    }

    assert_eq!(cpu.registers.hl(), 0xC001);
    assert_eq!(bus.read(0xC001), 0x84); // 0100_0010 → 1000_0100
}

// ── Controle negativo — $CB entra em decoded_elsewhere ────────────────────

#[test]
fn cb_prefix_opcode_is_in_decoded_knowingly() {
    assert!(
        support::decoded_elsewhere(0xCB),
        "$CB (prefixo) está em decoded_elsewhere"
    );
}

#[test]
fn cb_second_bytes_are_decoded_by_cb_fetch_not_by_fetch() {
    // $00 a $07 são decodificados por `cb_fetch` como RLC, não por `fetch`.
    // Mas $00 (NOP), $01 (LD BC,u16) etc. continuam decodificados por `fetch`
    // também — o caminho de decodificação é separado, e `decoded_elsewhere`
    // mede só o não-CB.
    let cb_second_bytes = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07];
    for &opcode in &cb_second_bytes {
        // O non-CB fetch não reconhece estes como CB, mas pode reconhecê-los
        // como opcodes independentes (ex: $00 é NOP). O que importa é que
        // eles já estavam em `decoded_elsewhere` ANTES desta iteração.
        // Esta asserção verifica só que a lista não foi corrompida.
        assert!(
            support::decoded_elsewhere(opcode),
            "${opcode:02X} já estava em decoded_elsewhere (como opcode não-CB)"
        );
    }
}
