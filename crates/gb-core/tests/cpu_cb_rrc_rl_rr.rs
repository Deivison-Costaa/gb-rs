//! ROADMAP 1.9b — prefixo CB: `RRC` (`CB 08`–`CB 0F`), `RL` (`CB 10`–`CB 17`),
//! `RR` (`CB 18`–`CB 1F`). Mesmo mecanismo do `RLC` (0033): `CbFetch` decodifica
//! o segundo byte; 2 M-cycles para registrador, 4 para `(HL)`. `Z` calculado,
//! `N=0`, `H=0`, `C` = bit deslocado para fora. `RL`/`RR` consomem o `C` antigo.

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

// ── RRC B (CB 08) — rotação circular à direita ─────────────────────────────

#[test]
fn rrc_b_rotates_right_and_copies_bit_0_to_carry_and_bit_7() {
    // 1000_0101 → 1100_0010, carry=1
    let opcode_byte = 0x08;
    for (value, expected) in [
        (0x85, 0xC2), // 1000_0101 → 1100_0010, C=1
        (0x00, 0x00), // tudo zero
        (0x01, 0x80), // 0000_0001 → 1000_0000, C=1
        (0xFF, 0xFF),
    ] {
        let (mut cpu, mut bus) = machine(&[0xCB, opcode_byte]);
        cpu.registers.b = value;
        cpu.registers.a = 0xFF;

        cpu.step(&mut bus); // M1: fetch $CB
        cpu.step(&mut bus); // M2: fetch opcode + execução

        assert_eq!(
            cpu.registers.b, expected,
            "RRC B {value:#04X}: esperado={expected:#04X}, obtido={:#04X}",
            cpu.registers.b
        );
    }
}

#[test]
fn rrc_b_takes_two_m_cycles() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x08, 0x00]);
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

// ── RRC A (CB 0F) — calcula Z, ao contrário do RRCA ────────────────────────

#[test]
fn rrc_a_sets_z_flag_when_result_is_zero_unlike_rrca() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x0F]);
    cpu.registers.a = 0x00;

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert!(
        cpu.registers.flag(Flag::Z),
        "RRC A 0x00: resultado=0x00 → Z deve ser 1"
    );
    assert_eq!(cpu.registers.a, 0x00);
}

#[test]
fn rrc_a_clears_z_flag_when_result_is_nonzero() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x0F]);
    cpu.registers.a = 0x01;

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert!(
        !cpu.registers.flag(Flag::Z),
        "RRC A 0x01: Z deve ser 0 (resultado 0x80 != 0)"
    );
}

// ── RL B (CB 10) — rotação à esquerda via carry ────────────────────────────

#[test]
fn rl_b_rotates_left_and_shifts_old_carry_into_bit_0() {
    let opcode_byte = 0x10;
    // C=0 antes da operação
    for (value, expected) in [
        (0x85, 0x0A), // 1000_0101 + C=0 → 0000_1010, C_out=1
        (0x00, 0x00), // tudo zero
        (0x80, 0x00), // 1000_0000 + C=0 → 0000_0000, C_out=1
        (0xFF, 0xFE), // 1111_1111 + C=0 → 1111_1110, C_out=1
    ] {
        let (mut cpu, mut bus) = machine(&[0xCB, opcode_byte]);
        cpu.registers.b = value;
        cpu.registers.set_flag(Flag::C, false);
        cpu.registers.a = 0xFF;

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(
            cpu.registers.b, expected,
            "RL B {value:#04X} C=0: esperado={expected:#04X}, obtido={:#04X}",
            cpu.registers.b
        );
    }
}

#[test]
fn rl_b_with_carry_one_inserts_one_into_bit_0() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x10]);
    cpu.registers.b = 0x85;
    cpu.registers.set_flag(Flag::C, true);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    // 1000_0101 + C_in=1 → 0000_1011, C_out=1
    assert_eq!(cpu.registers.b, 0x0B, "RL B 0x85 C=1 → 0x0B");
    assert!(cpu.registers.flag(Flag::C), "RL B 0x85: C_out = bit 7 = 1");
}

// ── RL A (CB 17) — calcula Z, ao contrário do RLA ──────────────────────────

#[test]
fn rl_a_sets_z_flag_when_result_is_zero_unlike_rla() {
    // A=0x00, C=0 → RL → resultado=0x00, Z=1
    let (mut cpu, mut bus) = machine(&[0xCB, 0x17]);
    cpu.registers.a = 0x00;
    cpu.registers.set_flag(Flag::C, false);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert!(
        cpu.registers.flag(Flag::Z),
        "RL A 0x00 C=0: resultado=0x00 → Z deve ser 1"
    );
}

#[test]
fn rl_a_clears_z_flag_when_result_is_nonzero() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x17]);
    cpu.registers.a = 0x01;
    cpu.registers.set_flag(Flag::C, false);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert!(
        !cpu.registers.flag(Flag::Z),
        "RL A 0x01 C=0: Z deve ser 0 (resultado 0x02 != 0)"
    );
}

// ── RR B (CB 18) — rotação à direita via carry ─────────────────────────────

#[test]
fn rr_b_rotates_right_and_shifts_old_carry_into_bit_7() {
    let opcode_byte = 0x18;
    // C=0 antes da operação
    for (value, expected) in [
        (0x85, 0x42), // 1000_0101 + C=0 → 0100_0010, C_out=1
        (0x00, 0x00),
        (0x01, 0x00), // 0000_0001 + C=0 → 0000_0000, C_out=1
        (0xFF, 0x7F), // 1111_1111 + C=0 → 0111_1111, C_out=1
    ] {
        let (mut cpu, mut bus) = machine(&[0xCB, opcode_byte]);
        cpu.registers.b = value;
        cpu.registers.set_flag(Flag::C, false);
        cpu.registers.a = 0xFF;

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(
            cpu.registers.b, expected,
            "RR B {value:#04X} C=0: esperado={expected:#04X}, obtido={:#04X}",
            cpu.registers.b
        );
    }
}

#[test]
fn rr_b_with_carry_one_inserts_one_into_bit_7() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x18]);
    cpu.registers.b = 0x85;
    cpu.registers.set_flag(Flag::C, true);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    // 1000_0101 + C_in=1 → 1100_0010, C_out=1
    assert_eq!(cpu.registers.b, 0xC2, "RR B 0x85 C=1 → 0xC2");
    assert!(cpu.registers.flag(Flag::C), "RR B 0x85: C_out = bit 0 = 1");
}

// ── RR A (CB 1F) — calcula Z, ao contrário do RRA ──────────────────────────

#[test]
fn rr_a_sets_z_flag_when_result_is_zero_unlike_rra() {
    // A=0x00, C=0 → RR → resultado=0x00, Z=1
    let (mut cpu, mut bus) = machine(&[0xCB, 0x1F]);
    cpu.registers.a = 0x00;
    cpu.registers.set_flag(Flag::C, false);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert!(
        cpu.registers.flag(Flag::Z),
        "RR A 0x00 C=0: resultado=0x00 → Z deve ser 1"
    );
}

#[test]
fn rr_a_clears_z_flag_when_result_is_nonzero() {
    // A=0x02 C=0 → 0000_0010 >> 1 │ 0<<7 = 0000_0001, Z=0
    let (mut cpu, mut bus) = machine(&[0xCB, 0x1F]);
    cpu.registers.a = 0x02;
    cpu.registers.set_flag(Flag::C, false);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert!(
        !cpu.registers.flag(Flag::Z),
        "RR A 0x02 C=0: Z deve ser 0 (resultado 0x01 != 0)"
    );
}

// ── Flags de cada operação ──────────────────────────────────────────────────

fn flags_rrc(value: u8) -> (bool, bool, bool, bool) {
    let carry = (value & 0x01) != 0;
    let result = (value >> 1) | (u8::from(carry) << 7);
    (result == 0, false, false, carry)
}

fn flags_rl(value: u8, carry_in: bool) -> (bool, bool, bool, bool) {
    let carry = (value & 0x80) != 0;
    let result = (value << 1) | u8::from(carry_in);
    (result == 0, false, false, carry)
}

fn flags_rr(value: u8, carry_in: bool) -> (bool, bool, bool, bool) {
    let carry = (value & 0x01) != 0;
    let result = (value >> 1) | (u8::from(carry_in) << 7);
    (result == 0, false, false, carry)
}

#[test]
fn rrc_calculates_z_and_clears_n_h_and_sets_c_from_bit_0() {
    for value in [0x00, 0x01, 0x80, 0x85, 0xFF] {
        let (mut cpu, mut bus) = machine(&[0xCB, 0x08]); // RRC B
        cpu.registers.b = value;
        cpu.registers.a = 0xAA;

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        let (expected_z, _expected_n, _expected_h, expected_c) = flags_rrc(value);

        assert_eq!(
            cpu.registers.flag(Flag::Z),
            expected_z,
            "RRC B={value:#04X}: Z esperado={expected_z}"
        );
        assert!(
            !cpu.registers.flag(Flag::N),
            "RRC B={value:#04X}: N deve ser 0"
        );
        assert!(
            !cpu.registers.flag(Flag::H),
            "RRC B={value:#04X}: H deve ser 0"
        );
        assert_eq!(
            cpu.registers.flag(Flag::C),
            expected_c,
            "RRC B={value:#04X}: C esperado={expected_c}"
        );
    }
}

#[test]
fn rl_calculates_z_and_clears_n_h_and_sets_c_from_bit_7() {
    for carry_in in [false, true] {
        for value in [0x00, 0x01, 0x80, 0x85, 0xFF] {
            let (mut cpu, mut bus) = machine(&[0xCB, 0x10]); // RL B
            cpu.registers.b = value;
            cpu.registers.set_flag(Flag::C, carry_in);
            cpu.registers.a = 0xAA;

            cpu.step(&mut bus);
            cpu.step(&mut bus);

            let (expected_z, _expected_n, _expected_h, expected_c) = flags_rl(value, carry_in);

            // N e H são fixos: o teste os verifica independentemente
            assert!(
                !cpu.registers.flag(Flag::N),
                "RL B={value:#04X} C_in={carry_in}: N deve ser 0"
            );
            assert!(
                !cpu.registers.flag(Flag::H),
                "RL B={value:#04X} C_in={carry_in}: H deve ser 0"
            );
            assert_eq!(
                cpu.registers.flag(Flag::Z),
                expected_z,
                "RL B={value:#04X} C_in={carry_in}: Z esperado={expected_z}"
            );
            assert_eq!(
                cpu.registers.flag(Flag::C),
                expected_c,
                "RL B={value:#04X} C_in={carry_in}: C esperado={expected_c}"
            );
        }
    }
}

#[test]
fn rr_calculates_z_and_clears_n_h_and_sets_c_from_bit_0() {
    for carry_in in [false, true] {
        for value in [0x00, 0x01, 0x80, 0x85, 0xFF] {
            let (mut cpu, mut bus) = machine(&[0xCB, 0x18]); // RR B
            cpu.registers.b = value;
            cpu.registers.set_flag(Flag::C, carry_in);
            cpu.registers.a = 0xAA;

            cpu.step(&mut bus);
            cpu.step(&mut bus);

            let (expected_z, _expected_n, _expected_h, expected_c) = flags_rr(value, carry_in);

            assert!(
                !cpu.registers.flag(Flag::N),
                "RR B={value:#04X} C_in={carry_in}: N deve ser 0"
            );
            assert!(
                !cpu.registers.flag(Flag::H),
                "RR B={value:#04X} C_in={carry_in}: H deve ser 0"
            );
            assert_eq!(
                cpu.registers.flag(Flag::Z),
                expected_z,
                "RR B={value:#04X} C_in={carry_in}: Z esperado={expected_z}"
            );
            assert_eq!(
                cpu.registers.flag(Flag::C),
                expected_c,
                "RR B={value:#04X} C_in={carry_in}: C esperado={expected_c}"
            );
        }
    }
}

// ── Isolamento — toca só o registrador destino ──────────────────────────────

#[test]
fn rrc_b_touches_only_b_and_flags() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x08]);
    cpu.registers.b = 0x85;
    cpu.registers.a = 0x42;
    cpu.registers.c = 0x01;
    cpu.registers.d = 0x02;
    cpu.registers.e = 0x03;
    let hl_before = cpu.registers.hl();
    let sp_before = cpu.registers.sp;

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(cpu.registers.b, 0xC2);
    assert_eq!(cpu.registers.a, 0x42);
    assert_eq!(cpu.registers.c, 0x01);
    assert_eq!(cpu.registers.d, 0x02);
    assert_eq!(cpu.registers.e, 0x03);
    assert_eq!(cpu.registers.hl(), hl_before);
    assert_eq!(cpu.registers.sp, sp_before);
}

// ── (HL) — read-modify-write, 4 M-cycles ────────────────────────────────────

#[test]
fn rrc_hl_reads_modifies_and_writes_in_four_m_cycles() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x0E, 0x00, 0x00, 0x00]);
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
        result, 0xC2,
        "RRC (HL) com valor 0x85 → 0xC2, obtido {result:#04X}"
    );
}

#[test]
fn rrc_hl_does_not_change_hl_itself() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x0E]);
    cpu.registers.set_hl(0xC000);
    bus.write(0xC000, 0x01);

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(cpu.registers.hl(), 0xC000, "RRC (HL) não muda HL");
}

#[test]
fn rl_hl_reads_modifies_and_writes_in_four_m_cycles() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x16, 0x00, 0x00, 0x00]);
    cpu.registers.set_hl(0xC000);
    bus.write(0xC000, 0x85);
    cpu.registers.set_flag(Flag::C, false);

    cpu.step(&mut bus); // M1
    cpu.step(&mut bus); // M2
    cpu.step(&mut bus); // M3
    cpu.step(&mut bus); // M4

    let result = bus.read(0xC000);
    assert_eq!(
        result, 0x0A,
        "RL (HL) com valor 0x85 C=0 → 0x0A, obtido {result:#04X}"
    );
}

#[test]
fn rl_hl_with_carry_one_uses_it_as_bit_0() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x16]);
    cpu.registers.set_hl(0xC000);
    bus.write(0xC000, 0x85);
    cpu.registers.set_flag(Flag::C, true);

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(bus.read(0xC000), 0x0B, "RL (HL) 0x85 C=1 → 0x0B");
}

#[test]
fn rr_hl_reads_modifies_and_writes_in_four_m_cycles() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x1E, 0x00, 0x00, 0x00]);
    cpu.registers.set_hl(0xC000);
    bus.write(0xC000, 0x85);
    cpu.registers.set_flag(Flag::C, false);

    cpu.step(&mut bus); // M1
    cpu.step(&mut bus); // M2
    cpu.step(&mut bus); // M3
    cpu.step(&mut bus); // M4

    let result = bus.read(0xC000);
    assert_eq!(
        result, 0x42,
        "RR (HL) com valor 0x85 C=0 → 0x42, obtido {result:#04X}"
    );
}

#[test]
fn rr_hl_with_carry_one_uses_it_as_bit_7() {
    let (mut cpu, mut bus) = machine(&[0xCB, 0x1E]);
    cpu.registers.set_hl(0xC000);
    bus.write(0xC000, 0x85);
    cpu.registers.set_flag(Flag::C, true);

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(bus.read(0xC000), 0xC2, "RR (HL) 0x85 C=1 → 0xC2");
}

// ── Controle negativo — bytes do CB continuam decodificados ────────────────

#[test]
fn cb_second_bytes_08_to_1f_are_decoded_by_cb_fetch_not_by_fetch() {
    // Segundos bytes de CB que já estavam em decoded_elsewhere.
    // 0x10 (STOP) e 0x18 (JR e8) não estão — ainda não implementados como não-CB.
    // 0x16 (LD D,u8), 0x1E (LD E,u8) etc. estão por máscara de load imediato.
    let known: &[u8] = &[
        0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
        0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F,
    ];
    for &opcode in known {
        assert!(
            support::decoded_elsewhere(opcode),
            "{opcode:#04X} já estava em decoded_elsewhere (como opcode não-CB)"
        );
    }
}
