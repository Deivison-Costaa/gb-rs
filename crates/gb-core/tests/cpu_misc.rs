//! ROADMAP 1.11 — misc: `CPL`/`SCF`/`CCF`/`DAA`/`DI`/`EI`/`STOP`, 1 M-cycle,
//! 4 T-cycles cada. `NOP` já foi feito no 1.3.

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::{Cpu, Flag};

mod support;
use support::decoded_elsewhere;

const ENTRY: usize = 0x0100;

const NOP: u8 = 0x00;
const CPL: u8 = 0x2F;
const SCF: u8 = 0x37;
const CCF: u8 = 0x3F;
const DAA: u8 = 0x27;
const DI: u8 = 0xF3;
const EI: u8 = 0xFB;
const STOP: u8 = 0x10;

fn machine(program: &[u8]) -> (Cpu, Bus) {
    let mut rom = vec![0x00; NoMbc::MAX_ROM_LEN];
    rom[ENTRY..ENTRY + program.len()].copy_from_slice(program);

    let checksum = CartridgeHeader::parse(&rom)
        .expect("32 KiB de ROM têm cabeçalho de sobra")
        .checksum();
    let cartridge = NoMbc::new(rom).expect("32 KiB cabem num cartucho sem MBC");

    (Cpu::after_boot_rom(checksum), Bus::new(Box::new(cartridge)))
}

fn step_one(opcode: u8) -> (Cpu, Bus) {
    let (mut cpu, mut bus) = machine(&[opcode]);
    cpu.step(&mut bus);
    (cpu, bus)
}

fn step_one_with_a(opcode: u8, a: u8) -> (Cpu, Bus) {
    let (mut cpu, mut bus) = machine(&[opcode]);
    cpu.registers.a = a;
    cpu.step(&mut bus);
    (cpu, bus)
}

// ── CPL ($2F): A = ~A, N=1, H=1, Z/C intactos ─────────────────────────────

#[test]
fn cpl_complements_a_and_sets_n_and_h() {
    for (a_in, expected_a) in [
        (0x00, 0xFF),
        (0xFF, 0x00),
        (0x55, 0xAA),
        (0x0F, 0xF0),
        (0x33, 0xCC),
    ] {
        let (cpu, _bus) = step_one_with_a(CPL, a_in);
        assert_eq!(
            cpu.registers.a, expected_a,
            "CPL A={a_in:#04X}: A esperado={expected_a:#04X}"
        );
        assert!(cpu.registers.flag(Flag::N), "CPL: N deve ser 1");
        assert!(cpu.registers.flag(Flag::H), "CPL: H deve ser 1");
    }
}

#[test]
fn cpl_preserves_z_flag() {
    let (cpu, _bus) = step_one_with_a(CPL, 0xFF);
    assert!(
        cpu.registers.flag(Flag::Z),
        "CPL A=0xFF → A=0x00, mas Z é preservado do valor anterior"
    );
}

#[test]
fn cpl_preserves_c_flag() {
    let (mut cpu, mut bus) = machine(&[CPL]);
    cpu.registers.a = 0x55;
    cpu.registers.set_flag(Flag::C, true);
    cpu.registers.set_flag(Flag::Z, true);

    cpu.step(&mut bus);

    assert!(cpu.registers.flag(Flag::C), "CPL: C era 1 e continuou 1");
    assert!(cpu.registers.flag(Flag::Z), "CPL: Z era 1 e continuou 1");
}

#[test]
fn cpl_takes_one_m_cycle() {
    let (cpu, _bus) = step_one(CPL);
    assert!(cpu.is_between_instructions(), "CPL tem 1 M-cycle");
    assert_eq!(cpu.registers.pc, ENTRY as u16 + 1);
}

// ── SCF ($37): C=1, N=0, H=0, Z intacto ──────────────────────────────────

#[test]
fn scf_sets_c_and_clears_n_and_h() {
    for c_before in [false, true] {
        let (mut cpu, mut bus) = machine(&[SCF]);
        cpu.registers.set_flag(Flag::C, c_before);
        cpu.registers.set_flag(Flag::N, true);
        cpu.registers.set_flag(Flag::H, true);

        cpu.step(&mut bus);

        assert!(
            cpu.registers.flag(Flag::C),
            "SCF C_before={c_before}: C deve ser 1"
        );
        assert!(!cpu.registers.flag(Flag::N), "SCF: N deve ser 0");
        assert!(!cpu.registers.flag(Flag::H), "SCF: H deve ser 0");
    }
}

#[test]
fn scf_preserves_z_flag() {
    for z_before in [false, true] {
        let (mut cpu, mut bus) = machine(&[SCF]);
        cpu.registers.set_flag(Flag::Z, z_before);
        cpu.registers.set_flag(Flag::C, !z_before);

        cpu.step(&mut bus);

        assert_eq!(
            cpu.registers.flag(Flag::Z),
            z_before,
            "SCF Z_before={z_before}: Z deve continuar {z_before}"
        );
    }
}

#[test]
fn scf_does_not_touch_a() {
    let (mut cpu, mut bus) = machine(&[SCF]);
    cpu.registers.a = 0xAB;

    cpu.step(&mut bus);

    assert_eq!(cpu.registers.a, 0xAB, "SCF não altera A");
}

#[test]
fn scf_takes_one_m_cycle() {
    let (cpu, _bus) = step_one(SCF);
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY as u16 + 1);
}

// ── CCF ($3F): C = ~C, N=0, H=0, Z intacto ────────────────────────────────

#[test]
fn ccf_flips_c_and_clears_n_and_h() {
    for c_before in [false, true] {
        let (mut cpu, mut bus) = machine(&[CCF]);
        cpu.registers.set_flag(Flag::C, c_before);
        cpu.registers.set_flag(Flag::N, true);
        cpu.registers.set_flag(Flag::H, true);

        cpu.step(&mut bus);

        assert_eq!(
            cpu.registers.flag(Flag::C),
            !c_before,
            "CCF C_before={c_before}: C deve ser {}",
            !c_before
        );
        assert!(!cpu.registers.flag(Flag::N), "CCF: N deve ser 0");
        assert!(!cpu.registers.flag(Flag::H), "CCF: H deve ser 0");
    }
}

#[test]
fn ccf_preserves_z_flag() {
    for z_before in [false, true] {
        let (mut cpu, mut bus) = machine(&[CCF]);
        cpu.registers.set_flag(Flag::Z, z_before);

        cpu.step(&mut bus);

        assert_eq!(
            cpu.registers.flag(Flag::Z),
            z_before,
            "CCF Z_before={z_before}: Z deve continuar {z_before}"
        );
    }
}

#[test]
fn ccf_does_not_touch_a() {
    let (mut cpu, mut bus) = machine(&[CCF]);
    cpu.registers.a = 0xAB;

    cpu.step(&mut bus);

    assert_eq!(cpu.registers.a, 0xAB, "CCF não altera A");
}

#[test]
fn ccf_takes_one_m_cycle() {
    let (cpu, _bus) = step_one(CCF);
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY as u16 + 1);
}

// ── DAA ($27): ajuste BCD de A após ADD/SUB — flags: Z calculado, H=0,
//    N preservado, C pode ser setado ──────────────────────────────────────

#[test]
fn daa_after_addition_adjusts_bcd_correctly() {
    // (A_in, N, C, H, expected_A, expected_C)
    let cases: [(u8, bool, bool, bool, u8, bool); 12] = [
        (0x00, false, false, false, 0x00, false),
        (0x09, false, false, false, 0x09, false),
        (0x0A, false, false, false, 0x10, false),
        (0x0F, false, false, false, 0x15, false),
        (0x99, false, false, false, 0x99, false),
        (0xA0, false, false, false, 0x00, true),
        (0x00, false, false, true, 0x06, false),
        (0x9A, false, false, true, 0x00, true),
        // O ajuste do nibble baixo estoura u8; a comparação do nibble alto
        // usa o intermediário antes do truncamento. Ver iter 0048.
        (0xFA, false, false, false, 0x60, true),
        (0x9F, false, false, false, 0x05, true),
        // H=1 idêntico a (A&0x0F)>9 para o nibble baixo com 0xFA, mas
        // C de entrada também aciona o ajuste do nibble alto com 0xFA
        (0xFA, false, true, false, 0x60, true),
        // A=0x99 com H=1: o ajuste do nibble baixo leva a = 0x9F, que
        // é exatamente igual ao threshold. Sem 0x60. Ver iter 0048.
        (0x99, false, false, true, 0x9F, false),
    ];

    for (a_in, n, c, h, expected_a, expected_c) in cases {
        let (mut cpu, mut bus) = machine(&[DAA]);
        cpu.registers.a = a_in;
        cpu.registers.set_flag(Flag::N, n);
        cpu.registers.set_flag(Flag::C, c);
        cpu.registers.set_flag(Flag::H, h);

        cpu.step(&mut bus);

        assert_eq!(
            cpu.registers.a, expected_a,
            "DAA ADD A={a_in:#04X} H={h} C={c}: A esperado={expected_a:#04X}"
        );
        assert!(!cpu.registers.flag(Flag::H), "DAA: H deve ser 0");
        assert_eq!(
            cpu.registers.flag(Flag::C),
            expected_c,
            "DAA ADD A={a_in:#04X} H={h} C={c}: C esperado={expected_c}"
        );
    }
}

#[test]
fn daa_z_is_computed_from_final_a() {
    let (cpu, _bus) = step_one_with_a(DAA, 0xA0);
    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.registers.flag(Flag::Z), "DAA: Z=1 quando resultado é 0");

    let (mut cpu, mut bus) = machine(&[DAA]);
    cpu.registers.a = 0x01;
    cpu.step(&mut bus);
    assert!(
        !cpu.registers.flag(Flag::Z),
        "DAA: Z=0 quando resultado não é 0"
    );
}

#[test]
fn daa_preserves_n_flag() {
    for n_before in [false, true] {
        let (mut cpu, mut bus) = machine(&[DAA]);
        cpu.registers.a = 0x0A;
        cpu.registers.set_flag(Flag::N, n_before);

        cpu.step(&mut bus);

        assert_eq!(
            cpu.registers.flag(Flag::N),
            n_before,
            "DAA: N deve continuar {n_before}"
        );
    }
}

#[test]
fn daa_after_subtraction_adjusts_bcd_correctly() {
    // (A_in, H, C, expected_A)
    let cases: [(u8, bool, bool, u8); 4] = [
        (0x00, false, false, 0x00),
        (0x06, false, true, 0xA6),
        (0x06, true, false, 0x00),
        (0x06, true, true, 0xA0),
    ];

    for (a_in, h, c, expected_a) in cases {
        let (mut cpu, mut bus) = machine(&[DAA]);
        cpu.registers.a = a_in;
        cpu.registers.set_flag(Flag::N, true);
        cpu.registers.set_flag(Flag::H, h);
        cpu.registers.set_flag(Flag::C, c);

        cpu.step(&mut bus);

        assert_eq!(
            cpu.registers.a, expected_a,
            "DAA SUB A={a_in:#04X} H={h} C={c}: A esperado={expected_a:#04X}"
        );
    }
}

#[test]
fn daa_takes_one_m_cycle() {
    let (cpu, _bus) = step_one(DAA);
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY as u16 + 1);
}

// ── DI ($F3): IME = 0, sem flags ──────────────────────────────────────────

#[test]
fn di_clears_ime() {
    let (mut cpu, mut bus) = machine(&[DI]);
    cpu.ime = true;

    cpu.step(&mut bus);

    assert!(!cpu.ime, "DI: IME deve ser 0");
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY as u16 + 1);
}

#[test]
fn di_does_not_change_flags_or_registers_except_pc_and_ime() {
    let (mut cpu, mut bus) = machine(&[DI]);
    let before = cpu.registers;

    cpu.step(&mut bus);

    assert_eq!(cpu.registers.a, before.a, "DI não altera A");
    assert_eq!(cpu.registers.bc(), before.bc(), "DI não altera BC");
    assert_eq!(cpu.registers.de(), before.de(), "DI não altera DE");
    assert_eq!(cpu.registers.hl(), before.hl(), "DI não altera HL");
    assert_eq!(cpu.registers.sp, before.sp, "DI não altera SP");
    assert_eq!(cpu.registers.f, before.f, "DI não altera F");
    assert_eq!(cpu.registers.pc, before.pc + 1, "DI avança PC de 1");
}

// ── EI ($FB): IME = 1, sem flags ──────────────────────────────────────────

#[test]
fn ei_sets_ime() {
    let (mut cpu, mut bus) = machine(&[EI]);
    cpu.ime = false;

    cpu.step(&mut bus);

    assert!(cpu.ime, "EI: IME deve ser 1");
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY as u16 + 1);
}

#[test]
fn ei_does_not_change_flags_or_registers_except_pc_and_ime() {
    let (mut cpu, mut bus) = machine(&[EI]);
    let before = cpu.registers;

    cpu.step(&mut bus);

    assert_eq!(cpu.registers.a, before.a, "EI não altera A");
    assert_eq!(cpu.registers.bc(), before.bc(), "EI não altera BC");
    assert_eq!(cpu.registers.de(), before.de(), "EI não altera DE");
    assert_eq!(cpu.registers.hl(), before.hl(), "EI não altera HL");
    assert_eq!(cpu.registers.sp, before.sp, "EI não altera SP");
    assert_eq!(cpu.registers.f, before.f, "EI não altera F");
    assert_eq!(cpu.registers.pc, before.pc + 1, "EI avança PC de 1");
}

// ── STOP ($10): stub — para a CPU, PC avança 1 ───────────────────────────

#[test]
fn stop_advances_pc_and_stops_the_cpu() {
    let (cpuf, _bus) = step_one(STOP);
    assert_eq!(cpuf.registers.pc, ENTRY as u16 + 1);
    assert!(
        !cpuf.is_between_instructions(),
        "STOP: CPU está parada, não entre instruções"
    );
}

#[test]
fn stop_does_not_change_flags_or_registers_except_pc() {
    let (mut cpu, mut bus) = machine(&[STOP]);
    let mut expected = cpu.registers;
    expected.pc = ENTRY as u16 + 1;

    cpu.step(&mut bus);

    assert_eq!(cpu.registers, expected, "STOP só avança o PC");
}

#[test]
fn stopped_cpu_stays_stopped() {
    let (mut cpu, mut bus) = machine(&[STOP, NOP, NOP]);

    cpu.step(&mut bus);

    let frozen = cpu.registers;

    for _ in 0..8 {
        cpu.step(&mut bus);
    }

    assert_eq!(cpu.registers, frozen, "CPU parada não executa mais nada");
}

// ── Controle negativo ─────────────────────────────────────────────────────

#[test]
fn the_seven_misc_opcodes_are_in_decoded_knowingly() {
    for opcode in [CPL, SCF, CCF, DAA, DI, EI, STOP] {
        assert!(
            decoded_elsewhere(opcode),
            "${opcode:02X} está em decoded_elsewhere"
        );
    }
}
