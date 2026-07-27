//! ROADMAP 2.3 — HALT ($76) e o bug do HALT.
//! spec: docs/reference/05-interrupts.md § halt e § halt bug

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::Cpu;

mod support;
use support::decoded_elsewhere;

const ENTRY: usize = 0x0100;
const HALT: u8 = 0x76;
const NOP: u8 = 0x00;
const IE: u16 = 0xFFFF;
const IF: u16 = 0xFF0F;

fn machine(program: &[u8]) -> (Cpu, Bus) {
    let mut rom = vec![0x00; NoMbc::MAX_ROM_LEN];
    rom[ENTRY..ENTRY + program.len()].copy_from_slice(program);

    let checksum = CartridgeHeader::parse(&rom)
        .expect("32 KiB de ROM têm cabeçalho de sobra")
        .checksum();
    let cartridge = NoMbc::new(rom).expect("32 KiB cabem num cartucho sem MBC");

    (Cpu::after_boot_rom(checksum), Bus::new(Box::new(cartridge)))
}

// ── HALT normal: CPU pausa e não avança ─────────────────────────────────

#[test]
fn halted_cpu_is_not_locked() {
    let (mut cpu, mut bus) = machine(&[HALT]);
    bus.write(IE, 0x00);
    bus.write(IF, 0x00);

    cpu.step(&mut bus);

    assert_eq!(
        cpu.lockup(),
        None,
        "HALT não trava a CPU — o estado é halted, não locked"
    );
}

#[test]
fn halt_pauses_cpu_and_pc_does_not_advance() {
    let (mut cpu, mut bus) = machine(&[HALT, NOP, NOP]);
    bus.write(IE, 0x00);
    bus.write(IF, 0x00);

    cpu.step(&mut bus);
    let pc_after_halt = cpu.registers.pc;
    assert_eq!(pc_after_halt, 0x0101, "PC avança sobre o byte do HALT");

    for _ in 0..5 {
        cpu.step(&mut bus);
    }
    assert_eq!(
        cpu.registers.pc, pc_after_halt,
        "PC não muda enquanto halted"
    );
}

#[test]
fn halt_wakes_up_when_ie_and_if_non_zero() {
    let (mut cpu, mut bus) = machine(&[HALT, NOP]);
    bus.write(IE, 0x00);
    bus.write(IF, 0x00);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x0101);

    bus.write(IE, 0x01);
    bus.write(IF, 0x01);

    cpu.step(&mut bus);
    assert_eq!(
        cpu.registers.pc, 0x0102,
        "PC avançou — CPU acordou e executou NOP"
    );
}

#[test]
fn after_wake_up_cpu_is_no_longer_halted() {
    let (mut cpu, mut bus) = machine(&[HALT, NOP, NOP]);
    bus.write(IE, 0x00);
    bus.write(IF, 0x00);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x0101, "PC sobre HALT, halted");

    bus.write(IE, 0x01);
    bus.write(IF, 0x01);
    cpu.step(&mut bus);
    assert_eq!(
        cpu.registers.pc, 0x0102,
        "wake-up executou NOP na posição $0101"
    );

    bus.write(IE, 0x00);
    bus.write(IF, 0x00);
    cpu.step(&mut bus);
    assert_eq!(
        cpu.registers.pc, 0x0103,
        "CPU não está mais halted — executou o segundo NOP"
    );
}

#[test]
fn halt_with_ime_1_wakes_and_dispatches_interrupt() {
    let (mut cpu, mut bus) = machine(&[HALT, NOP]);
    cpu.ime = true;
    bus.write(IE, 0x01);
    bus.write(IF, 0x00);

    cpu.step(&mut bus);
    assert_eq!(
        cpu.registers.pc, 0x0101,
        "PC avança sobre HALT, então halted"
    );

    bus.write(IF, 0x01);

    cpu.step(&mut bus);
    assert!(!cpu.ime, "IME zerado — dispatch iniciou");

    for _ in 0..4 {
        cpu.step(&mut bus);
    }

    assert_eq!(
        cpu.registers.pc, 0x0040,
        "PC = vetor VBlank ($0040) após dispatch"
    );
}

#[test]
fn halt_with_ime_0_wakes_and_resumes_normal_fetch() {
    let (mut cpu, mut bus) = machine(&[HALT, NOP]);
    cpu.ime = false;
    bus.write(IE, 0x01);
    bus.write(IF, 0x00);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x0101, "PC sobre HALT, halted");

    bus.write(IF, 0x01);

    cpu.step(&mut bus);
    assert_eq!(
        cpu.registers.pc, 0x0102,
        "PC avançou — wake-up executou NOP, sem dispatch"
    );
    assert_eq!(
        bus.read(IF) & 0x01,
        0x01,
        "IF bit 0 ainda setado — sem dispatch com IME=0"
    );
}

// ── HALT bug: IME=0 e (IE & IF) != 0 no momento do HALT ────────────────

#[test]
fn halt_bug_does_not_halt_when_ime_0_and_pending() {
    let (mut cpu, mut bus) = machine(&[HALT, NOP, NOP]);
    cpu.ime = false;
    bus.write(IE, 0x01);
    bus.write(IF, 0x01);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x0101, "PC avança sobre HALT");

    cpu.step(&mut bus);
    assert_eq!(
        cpu.registers.pc, 0x0101,
        "halt bug: PC não incrementou — NOP da posição $0101 lido sem incremento"
    );

    cpu.step(&mut bus);
    assert_eq!(
        cpu.registers.pc, 0x0102,
        "segunda leitura do mesmo byte avança PC"
    );
}

#[test]
fn halt_bug_byte_after_halt_executed_twice() {
    // Usa INC B ($04) que é observável: B incrementa duas vezes.
    let (mut cpu, mut bus) = machine(&[HALT, 0x04]);
    cpu.ime = false;
    bus.write(IE, 0x01);
    bus.write(IF, 0x01);

    let b_before = cpu.registers.b;

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x0101, "PC sobre HALT");

    cpu.step(&mut bus);
    assert_eq!(
        cpu.registers.pc, 0x0101,
        "halt bug: PC estaciona — INC B no $0101 lido sem incremento"
    );
    assert_eq!(
        cpu.registers.b,
        b_before.wrapping_add(1),
        "INC B executou uma vez"
    );

    cpu.step(&mut bus);
    assert_eq!(
        cpu.registers.pc, 0x0102,
        "PC avança na segunda leitura do mesmo byte"
    );
    assert_eq!(
        cpu.registers.b,
        b_before.wrapping_add(2),
        "INC B executou segunda vez com PC normal"
    );
}

#[test]
fn halt_bug_does_not_lock_cpu() {
    let (mut cpu, mut bus) = machine(&[HALT]);
    cpu.ime = false;
    bus.write(IE, 0x01);
    bus.write(IF, 0x01);

    cpu.step(&mut bus);

    assert_eq!(cpu.lockup(), None, "halt bug não trava a CPU");
}

// ── Controle negativo ──────────────────────────────────────────────────

#[test]
fn opcodes_used_in_halt_tests_are_decoded_elsewhere() {
    assert!(decoded_elsewhere(NOP));
    assert!(decoded_elsewhere(0x04)); // INC B
}
