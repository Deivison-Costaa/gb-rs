//! ROADMAP 2.2 — Interrupções: IE/IF/IME, vetores, dispatch, EI com delay.
//! spec: docs/reference/05-interrupts.md

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::Cpu;

mod support;
use support::decoded_elsewhere;

const ENTRY: usize = 0x0100;
const IE: u16 = 0xFFFF;
const IF: u16 = 0xFF0F;
const NOP: u8 = 0x00;
const EI: u8 = 0xFB;
const DI: u8 = 0xF3;

fn machine(program: &[u8]) -> (Cpu, Bus) {
    let mut rom = vec![0x00; NoMbc::MAX_ROM_LEN];
    rom[ENTRY..ENTRY + program.len()].copy_from_slice(program);

    let checksum = CartridgeHeader::parse(&rom)
        .expect("32 KiB de ROM têm cabeçalho de sobra")
        .checksum();
    let cartridge = NoMbc::new(rom).expect("32 KiB cabem num cartucho sem MBC");

    (Cpu::after_boot_rom(checksum), Bus::new(Box::new(cartridge)))
}

// ── IE ($FFFF) e IF ($FF0F) ──────────────────────────────────────────────

#[test]
fn ie_is_readable_and_writable_at_ffff() {
    let (mut _cpu, mut bus) = machine(&[]);
    bus.write(IE, 0x0F);
    assert_eq!(bus.read(IE), 0x0F);
}

#[test]
fn if_is_readable_and_writable_at_ff0f() {
    let (mut _cpu, mut bus) = machine(&[]);
    bus.write(IF, 0x07);
    assert_eq!(bus.read(IF), 0x07);
}

#[test]
fn ie_starts_at_zero_after_boot() {
    let (mut _cpu, bus) = machine(&[]);
    assert_eq!(bus.read(IE), 0x00);
}

#[test]
fn if_starts_at_e1_after_boot() {
    let (mut _cpu, bus) = machine(&[]);
    assert_eq!(bus.read(IF), 0xE1);
}

// ── Interrupção não dispara sem IME ou sem IE ───────────────────────────

#[test]
fn interrupt_does_not_fire_when_ime_is_zero() {
    let (mut cpu, mut bus) = machine(&[]);
    cpu.ime = false;
    bus.write(IE, 0x04);
    bus.write(IF, 0x04);

    cpu.step(&mut bus);

    assert!(!cpu.ime, "IME continua 0");
    assert_eq!(bus.read(IF), 0x04, "IF não foi alterado");
}

#[test]
fn interrupt_does_not_fire_when_ie_bit_is_zero() {
    let (mut cpu, mut bus) = machine(&[]);
    cpu.ime = true;
    bus.write(IE, 0x00);
    bus.write(IF, 0x04);

    cpu.step(&mut bus);

    assert_eq!(bus.read(IF), 0x04, "IF não foi alterado");
}

#[test]
fn interrupt_does_not_fire_when_if_bit_is_zero() {
    let (mut cpu, mut bus) = machine(&[]);
    cpu.ime = true;
    bus.write(IE, 0x04);
    bus.write(IF, 0x00);

    let pc_before = cpu.registers.pc;
    cpu.step(&mut bus);

    assert_eq!(cpu.registers.pc, pc_before.wrapping_add(1), "fetch normal");
}

// ── Dispatch de interrupção — fluxo básico ─────────────────────────────

#[test]
fn timer_interrupt_dispatch_clears_ime_and_if_bit() {
    let (mut cpu, mut bus) = machine(&[]);
    cpu.ime = true;
    bus.write(IE, 0x04);
    bus.write(IF, 0x04);

    cpu.step(&mut bus);

    assert!(!cpu.ime, "IME é zerado ao iniciar o dispatch");
    assert_eq!(bus.read(IF) & 0x04, 0, "bit 2 de IF foi limpo");
}

#[test]
fn interrupt_dispatch_pushes_pc_onto_stack() {
    let (mut cpu, mut bus) = machine(&[]);
    cpu.ime = true;
    bus.write(IE, 0x01);
    bus.write(IF, 0x01);

    let sp_before = cpu.registers.sp;
    let pc_before = cpu.registers.pc;

    for _ in 0..5 {
        cpu.step(&mut bus);
    }

    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.sp, sp_before.wrapping_sub(2));
    let low = bus.read(sp_before.wrapping_sub(2));
    let high = bus.read(sp_before.wrapping_sub(1));
    let pushed = u16::from_le_bytes([low, high]);
    assert_eq!(pushed, pc_before, "PC é empilhado antes do salto");
}

#[test]
fn interrupt_dispatch_jumps_to_vector() {
    let cases: [(u8, u16); 5] = [
        (0x01, 0x0040), // VBlank
        (0x02, 0x0048), // LCD
        (0x04, 0x0050), // Timer
        (0x08, 0x0058), // Serial
        (0x10, 0x0060), // Joypad
    ];

    for (intr, vector) in cases {
        let (mut cpu, mut bus) = machine(&[]);
        cpu.ime = true;
        bus.write(IE, intr);
        bus.write(IF, intr);

        for _ in 0..5 {
            cpu.step(&mut bus);
        }

        assert_eq!(
            cpu.registers.pc, vector,
            "vetor {vector:#06X} para bit={intr:#04X}"
        );
    }
}

#[test]
fn interrupt_dispatch_takes_five_m_cycles() {
    let (mut cpu, mut bus) = machine(&[]);
    cpu.ime = true;
    bus.write(IE, 0x01);
    bus.write(IF, 0x01);

    for i in 0..4 {
        cpu.step(&mut bus);
        assert!(
            !cpu.is_between_instructions(),
            "M-cycle {} ainda em dispatch",
            i + 1
        );
    }
    cpu.step(&mut bus);
    assert!(
        cpu.is_between_instructions(),
        "após 5 M-cycles, fetch normal"
    );
}

// ── Prioridade ──────────────────────────────────────────────────────────

#[test]
fn highest_priority_interrupt_is_serviced_first() {
    let (mut cpu, mut bus) = machine(&[]);
    cpu.ime = true;
    bus.write(IE, 0x05);
    bus.write(IF, 0x05);

    cpu.step(&mut bus);

    assert_eq!(bus.read(IF) & 0x04, 0x04, "bit 2 (Timer) ainda ativo");
    assert_eq!(
        bus.read(IF) & 0x01,
        0x00,
        "bit 0 (VBlank) foi limpo — prioridade maior"
    );
}

// ── EI com delay de 1 instrução ─────────────────────────────────────────

#[test]
fn ei_does_not_set_ime_until_after_next_instruction() {
    let (mut cpu, mut bus) = machine(&[EI, NOP]);
    cpu.ime = false;

    cpu.step(&mut bus);
    assert!(!cpu.ime, "IME ainda 0 após EI (M-cycle 1)");

    cpu.step(&mut bus);
    assert!(!cpu.ime, "IME ainda 0 executando NOP (M-cycle 2)");

    cpu.step(&mut bus);
    assert!(cpu.ime, "IME = 1 no fetch após NOP (M-cycle 3)");
}

#[test]
fn ei_followed_by_di_does_not_leave_interrupts_enabled() {
    let (mut cpu, mut bus) = machine(&[EI, DI]);
    cpu.ime = false;

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert!(!cpu.ime, "DI cancela o EI pendente");
}

#[test]
fn ei_with_ime_already_set_stays_with_delay() {
    let (mut cpu, mut bus) = machine(&[EI, NOP]);
    cpu.ime = true;

    cpu.step(&mut bus);
    assert!(cpu.ime, "IME=1 durante EI (já estava 1)");

    cpu.step(&mut bus);
    assert!(cpu.ime, "IME=1 durante NOP");

    cpu.step(&mut bus);
    assert!(cpu.ime, "IME=1 após NOP");
}

#[test]
fn interrupt_does_not_fire_between_ei_and_next_instruction() {
    let (mut cpu, mut bus) = machine(&[EI, NOP]);
    cpu.ime = false;
    bus.write(IE, 0x01);
    bus.write(IF, 0x01);

    cpu.step(&mut bus);
    assert!(!cpu.ime, "após EI: IME=0");

    cpu.step(&mut bus);
    assert!(!cpu.ime, "executando NOP: IME=0, interrupção não dispara");
    assert_eq!(
        bus.read(IF) & 0x01,
        0x01,
        "bit 0 de IF ainda setado — dispatch não ocorreu"
    );
}

// ── RETI com delay ──────────────────────────────────────────────────────

#[test]
fn reti_enables_interrupts_with_delay() {
    let ret_addr: u16 = 0x0200;
    let (mut cpu, mut bus) = machine(&[0xD9]);

    cpu.registers.sp = 0xFFFE;
    bus.write(0xFFFE, (ret_addr & 0xFF) as u8);
    bus.write(0xFFFF, (ret_addr >> 8) as u8);
    cpu.ime = false;

    for _ in 0..4 {
        cpu.step(&mut bus);
    }

    assert!(!cpu.ime, "IME ainda 0 após RETI — delay de 1 instrução");
    assert_eq!(
        cpu.registers.pc, ret_addr,
        "PC foi para o endereço de retorno"
    );

    cpu.step(&mut bus);
    assert!(!cpu.ime, "IME=0 durante instrução seguinte ao RETI");

    cpu.step(&mut bus);
    assert!(cpu.ime, "IME=1 após instrução seguinte ao RETI");
}

// ── DI ──────────────────────────────────────────────────────────────────

#[test]
fn di_clears_pending_ei() {
    let (mut cpu, mut bus) = machine(&[EI, DI, NOP]);
    cpu.ime = false;

    cpu.step(&mut bus); // EI
    assert!(!cpu.ime, "IME=0 após EI");

    cpu.step(&mut bus); // DI
    assert!(!cpu.ime, "IME=0 após DI");

    bus.write(IE, 0x01);
    bus.write(IF, 0x01);
    let if_before = bus.read(IF);

    cpu.step(&mut bus); // NOP (instrução seguinte ao DI)
    assert!(!cpu.ime, "IME=0 durante NOP");

    cpu.step(&mut bus); // fetch após NOP
    assert!(!cpu.ime, "IME continua 0 — EI foi cancelado, sem dispatch");

    assert_eq!(
        bus.read(IF),
        if_before,
        "IF inalterado — interrupção não foi servida"
    );
}

// ── Controle negativo: decoded_elsewhere ────────────────────────────────

#[test]
fn opcodes_used_in_interrupt_tests_are_decoded_elsewhere() {
    assert!(decoded_elsewhere(EI));
    assert!(decoded_elsewhere(DI));
    assert!(decoded_elsewhere(NOP));
}
