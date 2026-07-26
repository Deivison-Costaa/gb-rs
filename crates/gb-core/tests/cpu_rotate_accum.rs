//! ROADMAP 1.8 — `RLCA`/`RRCA`/`RLA`/`RRA`: rotações sobre `A`, 1 M-cycle,
//! 4 T-cycles. As quatro zeram `Z` incondicionalmente (coluna `0`), enquanto
//! os equivalentes `CB` calculam `Z` — mesmo nome, flag diferente.
//! `N` e `H` são `0` literais; `C` recebe o bit deslocado para fora.

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::{Cpu, Flag, Registers};

mod support;
use support::decoded_elsewhere;

const ENTRY: usize = 0x0100;

const RLCA: u8 = 0x07;
const RRCA: u8 = 0x0F;
const RLA: u8 = 0x17;
const RRA: u8 = 0x1F;

fn machine(program: &[u8]) -> (Cpu, Bus) {
    let mut rom = vec![0x00; NoMbc::MAX_ROM_LEN];
    rom[ENTRY..ENTRY + program.len()].copy_from_slice(program);

    let checksum = CartridgeHeader::parse(&rom)
        .expect("32 KiB de ROM têm cabeçalho de sobra")
        .checksum();
    let cartridge = NoMbc::new(rom).expect("32 KiB cabem num cartucho sem MBC");

    (Cpu::after_boot_rom(checksum), Bus::new(Box::new(cartridge)))
}

fn run_opcode(opcode: u8, a: u8, c_before: bool) -> (Registers, u16) {
    let (mut cpu, mut bus) = machine(&[opcode, 0x00]);
    cpu.registers.a = a;
    cpu.registers.set_flag(Flag::C, c_before);

    cpu.step(&mut bus);

    (cpu.registers, cpu.registers.pc)
}

fn flag_c(registers: &Registers) -> bool {
    registers.flag(Flag::C)
}

// ── RLCA ($07): rotaciona A para a esquerda, bit 7 vai para C e bit 0 ──────

#[test]
fn rlca_rotates_a_left_and_bit_7_goes_to_carry_and_bit_0() {
    let op = RLCA;
    for (a_in, expected_a, expected_c) in [
        (0x85, 0x0B, true),  // 1000_0101 → 0000_1011, C=1
        (0x07, 0x0E, false), // 0000_0111 → 0000_1110, C=0
        (0x00, 0x00, false), // tudo zero → tudo zero
        (0x80, 0x01, true),  // 1000_0000 → 0000_0001, C=1
        (0xFF, 0xFF, true),  // tudo 1 → tudo 1, C=1
    ] {
        let (regs, _) = run_opcode(op, a_in, false);
        assert_eq!(
            regs.a, expected_a,
            "RLCA A={a_in:#04X}: A esperado={expected_a:#04X}"
        );
        assert_eq!(
            flag_c(&regs),
            expected_c,
            "RLCA A={a_in:#04X}: C esperado={expected_c}"
        );
    }
}

#[test]
fn rlca_takes_one_m_cycle() {
    let (mut cpu, mut bus) = machine(&[RLCA, 0x42]);
    cpu.registers.a = 0x85;

    cpu.step(&mut bus);

    assert!(
        cpu.is_between_instructions(),
        "RLCA tem 1 M-cycle: depois de um step a instrução acabou"
    );
    assert_eq!(cpu.registers.pc, 0x0101, "RLCA avança PC em 1 byte");
}

#[test]
fn rlca_touches_only_a_and_flags() {
    let (mut cpu, mut bus) = machine(&[RLCA]);
    cpu.registers.a = 0x85;
    let bc_before = cpu.registers.bc();
    let de_before = cpu.registers.de();
    let hl_before = cpu.registers.hl();
    let sp_before = cpu.registers.sp;

    cpu.step(&mut bus);

    assert_eq!(cpu.registers.bc(), bc_before);
    assert_eq!(cpu.registers.de(), de_before);
    assert_eq!(cpu.registers.hl(), hl_before);
    assert_eq!(cpu.registers.sp, sp_before);
    assert_ne!(cpu.registers.a, 0x85);
}

// ── RRCA ($0F): rotaciona A para a direita, bit 0 vai para C e bit 7 ────────

#[test]
fn rrca_rotates_a_right_and_bit_0_goes_to_carry_and_bit_7() {
    let op = RRCA;
    for (a_in, expected_a, expected_c) in [
        (0x85, 0xC2, true),  // 1000_0101 → 1100_0010, C=1
        (0xE0, 0x70, false), // 1110_0000 → 0111_0000, C=0
        (0x00, 0x00, false),
        (0x01, 0x80, true), // 0000_0001 → 1000_0000, C=1
    ] {
        let (regs, _) = run_opcode(op, a_in, false);
        assert_eq!(
            regs.a, expected_a,
            "RRCA A={a_in:#04X}: A esperado={expected_a:#04X}"
        );
        assert_eq!(
            flag_c(&regs),
            expected_c,
            "RRCA A={a_in:#04X}: C esperado={expected_c}"
        );
    }
}

#[test]
fn rrca_takes_one_m_cycle() {
    let (mut cpu, mut bus) = machine(&[RRCA, 0x42]);

    cpu.step(&mut bus);

    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, 0x0101);
}

// ── RLA ($17): rotaciona A para a esquerda via Carry, C vai para bit 0 ──────

#[test]
fn rla_rotates_a_left_through_carry() {
    let op = RLA;
    for (a_in, c_in, expected_a, expected_c) in [
        (0x85, false, 0x0A, true),  // 1000_0101 C=0 → 0000_1010, C=1
        (0x85, true, 0x0B, true),   // 1000_0101 C=1 → 0000_1011, C=1
        (0x00, true, 0x01, false),  // 0000_0000 C=1 → 0000_0001, C=0
        (0x00, false, 0x00, false), // 0000_0000 C=0 → 0000_0000, C=0
        (0x80, false, 0x00, true),  // 1000_0000 C=0 → 0000_0000, C=1
        (0x80, true, 0x01, true),   // 1000_0000 C=1 → 0000_0001, C=1
    ] {
        let (regs, _) = run_opcode(op, a_in, c_in);
        assert_eq!(
            regs.a, expected_a,
            "RLA A={a_in:#04X} C={c_in}: A esperado={expected_a:#04X}"
        );
        assert_eq!(
            flag_c(&regs),
            expected_c,
            "RLA A={a_in:#04X} C={c_in}: C esperado={expected_c}"
        );
    }
}

#[test]
fn rla_takes_one_m_cycle() {
    let (mut cpu, mut bus) = machine(&[RLA, 0x42]);

    cpu.step(&mut bus);

    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, 0x0101);
}

// ── RRA ($1F): rotaciona A para a direita via Carry, C vai para bit 7 ────────

#[test]
fn rra_rotates_a_right_through_carry() {
    let op = RRA;
    for (a_in, c_in, expected_a, expected_c) in [
        (0x85, false, 0x42, true),  // 1000_0101 C=0 → 0100_0010, C=1
        (0x85, true, 0xC2, true),   // 1000_0101 C=1 → 1100_0010, C=1
        (0x00, true, 0x80, false),  // 0000_0000 C=1 → 1000_0000, C=0
        (0x00, false, 0x00, false), // 0000_0000 C=0 → 0000_0000, C=0
        (0x01, false, 0x00, true),  // 0000_0001 C=0 → 0000_0000, C=1
        (0x01, true, 0x80, true),   // 0000_0001 C=1 → 1000_0000, C=1
    ] {
        let (regs, _) = run_opcode(op, a_in, c_in);
        assert_eq!(
            regs.a, expected_a,
            "RRA A={a_in:#04X} C={c_in}: A esperado={expected_a:#04X}"
        );
        assert_eq!(
            flag_c(&regs),
            expected_c,
            "RRA A={a_in:#04X} C={c_in}: C esperado={expected_c}"
        );
    }
}

#[test]
fn rra_takes_one_m_cycle() {
    let (mut cpu, mut bus) = machine(&[RRA, 0x42]);

    cpu.step(&mut bus);

    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, 0x0101);
}

// ── Flags comuns: Z=0, N=0, H=0 incondicionais nas quatro ──────────────────

#[test]
fn all_four_rotate_opcodes_clear_z_n_and_h_unconditionally() {
    for opcode in [RLCA, RRCA, RLA, RRA] {
        for (a_in, c_before) in [(0x00, false), (0x01, false), (0x80, true), (0xFF, true)] {
            let (regs, _) = run_opcode(opcode, a_in, c_before);

            assert!(
                !regs.flag(Flag::Z),
                "${opcode:02X} A={a_in:#04X}: Z deve ser 0, é {}",
                regs.flag(Flag::Z)
            );
            assert!(
                !regs.flag(Flag::N),
                "${opcode:02X} A={a_in:#04X}: N deve ser 0"
            );
            assert!(
                !regs.flag(Flag::H),
                "${opcode:02X} A={a_in:#04X}: H deve ser 0"
            );
        }
    }
}

// Z incondicionalmente zero mesmo com A = 0 pós-rotação — a maior armadilha.
#[test]
fn rlca_clears_z_flag_even_when_result_is_zero() {
    let (regs, _) = run_opcode(RLCA, 0x00, false);
    assert!(
        !regs.flag(Flag::Z),
        "RLCA A=0 produz A=0, mas Z é 0 mesmo assim"
    );
}

#[test]
fn rrca_clears_z_flag_even_when_result_is_zero() {
    let (regs, _) = run_opcode(RRCA, 0x00, false);
    assert!(
        !regs.flag(Flag::Z),
        "RRCA A=0 produz A=0, mas Z é 0 mesmo assim"
    );
}

#[test]
fn rla_clears_z_flag_even_when_result_is_zero() {
    let (regs, _) = run_opcode(RLA, 0x00, false);
    assert!(
        !regs.flag(Flag::Z),
        "RLA A=0 C=0 → A=0, mas Z é 0 mesmo assim"
    );
}

#[test]
fn rra_clears_z_flag_even_when_result_is_zero() {
    let (regs, _) = run_opcode(RRA, 0x00, false);
    assert!(
        !regs.flag(Flag::Z),
        "RRA A=0 C=0 → A=0, mas Z é 0 mesmo assim"
    );
}

// ── Controle negativo ───────────────────────────────────────────────────────

#[test]
fn the_four_rotate_opcodes_are_in_decoded_knowingly() {
    assert!(
        decoded_elsewhere(RLCA),
        "$07 (RLCA) está em decoded_elsewhere"
    );
    assert!(
        decoded_elsewhere(RRCA),
        "$0F (RRCA) está em decoded_elsewhere"
    );
    assert!(
        decoded_elsewhere(RLA),
        "$17 (RLA) está em decoded_elsewhere"
    );
    assert!(
        decoded_elsewhere(RRA),
        "$1F (RRA) está em decoded_elsewhere"
    );
}
