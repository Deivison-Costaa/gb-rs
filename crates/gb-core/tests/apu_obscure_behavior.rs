//! ROADMAP 6.8b — extra length clocking + obscure behavior.
//! spec: docs/reference/07-apu.md § Obscure Behavior.

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::Cpu;

const NR10: u16 = 0xFF10;
const NR11: u16 = 0xFF11;
const NR12: u16 = 0xFF12;
const NR13: u16 = 0xFF13;
const NR14: u16 = 0xFF14;

const NR21: u16 = 0xFF16;
const NR22: u16 = 0xFF17;
const NR23: u16 = 0xFF18;
const NR24: u16 = 0xFF19;

const NR30: u16 = 0xFF1A;
const NR31: u16 = 0xFF1B;
const NR32: u16 = 0xFF1C;
const NR33: u16 = 0xFF1D;
const NR34: u16 = 0xFF1E;

const NR41: u16 = 0xFF20;
const NR42: u16 = 0xFF21;
const NR43: u16 = 0xFF22;
const NR44: u16 = 0xFF23;

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

fn step_n(cpu: &mut Cpu, bus: &mut Bus, n: u32) {
    for _ in 0..n {
        cpu.step(bus);
    }
}

const FS_MCYCLES: u32 = 2048;

#[test]
fn extra_length_clocking_decrementa_no_nrx4_com_transicao_0_para_1_e_next_step_nao_length() {
    let (_cpu, mut bus) = machine(&[]);

    bus.write(NR21, 0x3E);
    bus.write(NR22, 0xF1);
    bus.write(NR23, 0x00);
    bus.write(NR24, 0x87);
    assert!(bus.ch2_enabled());
    assert_eq!(bus.ch2_length_timer_internal(), 2);

    bus.write(NR24, 0x47);

    assert_eq!(
        bus.ch2_length_timer_internal(),
        1,
        "extra length clocking deve decrementar de 2 para 1"
    );
    assert!(bus.ch2_enabled());
}

#[test]
fn extra_length_clocking_desliga_canal_ao_decrementar_para_zero_sem_trigger_ch2() {
    let (_cpu, mut bus) = machine(&[]);

    bus.write(NR21, 0x3F);
    bus.write(NR22, 0xF1);
    bus.write(NR23, 0x00);
    bus.write(NR24, 0x87);
    assert!(bus.ch2_enabled());
    assert_eq!(bus.ch2_length_timer_internal(), 1);

    bus.write(NR24, 0x47);

    assert_eq!(bus.ch2_length_timer_internal(), 0);
    assert!(!bus.ch2_enabled());
}

#[test]
fn extra_length_clocking_desliga_canal_ao_decrementar_para_zero_sem_trigger_ch1() {
    let (_cpu, mut bus) = machine(&[]);

    bus.write(NR10, 0x00);
    bus.write(NR11, 0x3F);
    bus.write(NR12, 0xF1);
    bus.write(NR13, 0x00);
    bus.write(NR14, 0x87);
    assert!(bus.ch1_enabled());
    assert_eq!(bus.ch1_length_timer_internal(), 1);

    bus.write(NR14, 0x47);

    assert_eq!(bus.ch1_length_timer_internal(), 0);
    assert!(!bus.ch1_enabled());
}

#[test]
fn extra_length_clocking_desliga_canal_ao_decrementar_para_zero_sem_trigger_ch3() {
    let (_cpu, mut bus) = machine(&[]);

    bus.write(NR30, 0x80);
    bus.write(NR31, 0xFF);
    bus.write(NR32, 0x80);
    bus.write(NR33, 0x00);
    bus.write(NR34, 0x87);
    assert!(bus.ch3_enabled());
    assert_eq!(bus.ch3_length_timer_internal(), 1);

    bus.write(NR34, 0x47);

    assert_eq!(bus.ch3_length_timer_internal(), 0);
    assert!(!bus.ch3_enabled());
}

#[test]
fn extra_length_clocking_desliga_canal_ao_decrementar_para_zero_sem_trigger_ch4() {
    let (_cpu, mut bus) = machine(&[]);

    bus.write(NR41, 0x3F);
    bus.write(NR42, 0xF1);
    bus.write(NR43, 0x00);
    bus.write(NR44, 0x87);
    assert!(bus.ch4_enabled());
    assert_eq!(bus.ch4_length_timer_internal(), 1);

    bus.write(NR44, 0x47);

    assert_eq!(bus.ch4_length_timer_internal(), 0);
    assert!(!bus.ch4_enabled());
}

#[test]
fn extra_length_clocking_nao_decrementa_quando_next_step_e_length() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(NR21, 0x3E);
    bus.write(NR22, 0xF1);
    bus.write(NR23, 0x00);
    bus.write(NR24, 0x87);
    assert_eq!(bus.ch2_length_timer_internal(), 2);

    step_n(&mut cpu, &mut bus, FS_MCYCLES);
    assert_eq!(bus.apu_frame_sequencer_step(), 1);

    bus.write(NR24, 0x47);

    assert_eq!(
        bus.ch2_length_timer_internal(),
        2,
        "length não deve decrementar — next step é length clock"
    );
}

#[test]
fn extra_length_clocking_nao_decrementa_quando_bit6_nao_transicionou() {
    let (mut cpu, mut bus) = machine(&[]);

    step_n(&mut cpu, &mut bus, FS_MCYCLES);
    assert_eq!(bus.apu_frame_sequencer_step(), 1);

    bus.write(NR21, 0x3C);
    bus.write(NR22, 0xF1);
    bus.write(NR23, 0x00);
    bus.write(NR24, 0xC7);
    assert_eq!(bus.ch2_length_timer_internal(), 4);

    bus.write(NR24, 0x47);

    assert_eq!(
        bus.ch2_length_timer_internal(),
        4,
        "length não deve decrementar — bit6 já estava setado e next=length"
    );
}

#[test]
fn extra_length_clocking_nao_decrementa_segunda_vez_com_bit6_sem_transicao_e_next_non_length() {
    let (_cpu, mut bus) = machine(&[]);

    // Trigger com bit6=0, contador=2
    bus.write(NR21, 0x3E);
    bus.write(NR22, 0xF1);
    bus.write(NR23, 0x00);
    bus.write(NR24, 0x87);
    assert_eq!(bus.ch2_length_timer_internal(), 2);

    // Step=0, next=1 non-length. NR24=0x47: bit6 0→1 → extra clocking 2→1
    bus.write(NR24, 0x47);
    assert_eq!(bus.ch2_length_timer_internal(), 1);

    // Step=0, next=1 non-length. NR24=0x47: bit6 1→1 → sem extra clocking
    bus.write(NR24, 0x47);
    assert_eq!(
        bus.ch2_length_timer_internal(),
        1,
        "segunda escrita com bit6 já setado não deve decrementar, mesmo com next=NON-length"
    );
}

#[test]
fn extra_length_clocking_nao_decrementa_quando_length_timer_e_zero() {
    let (_cpu, mut bus) = machine(&[]);

    bus.write(NR21, 0x3F);
    bus.write(NR22, 0xF1);
    bus.write(NR23, 0x00);
    bus.write(NR24, 0x87);
    assert_eq!(bus.ch2_length_timer_internal(), 1);

    bus.write(NR24, 0x47);
    assert_eq!(bus.ch2_length_timer_internal(), 0);
    assert!(!bus.ch2_enabled());

    bus.write(NR24, 0x47);

    assert_eq!(
        bus.ch2_length_timer_internal(),
        0,
        "length timer zero não deve ser decrementado"
    );
}

#[test]
fn trigger_obscuro_carrega_63_em_vez_de_64_ch2_quando_next_step_nao_e_length() {
    let (_cpu, mut bus) = machine(&[]);

    bus.write(NR21, 0x3F);
    bus.write(NR22, 0xF1);
    bus.write(NR23, 0x00);
    bus.write(NR24, 0x87);
    assert!(bus.ch2_enabled());
    assert_eq!(bus.ch2_length_timer_internal(), 1);

    bus.write(NR24, 0x47);
    assert_eq!(bus.ch2_length_timer_internal(), 0);
    assert!(!bus.ch2_enabled());

    bus.write(NR24, 0xC7);

    assert_eq!(
        bus.ch2_length_timer_internal(),
        63,
        "trigger com next=NON-length e length=0 deve carregar 63"
    );
    assert!(bus.ch2_enabled());
}

#[test]
fn trigger_obscuro_carrega_63_em_vez_de_64_ch1() {
    let (_cpu, mut bus) = machine(&[]);

    bus.write(NR10, 0x00);
    bus.write(NR11, 0x3F);
    bus.write(NR12, 0xF1);
    bus.write(NR13, 0x00);
    bus.write(NR14, 0x87);
    assert!(bus.ch1_enabled());

    bus.write(NR14, 0x47);
    assert_eq!(bus.ch1_length_timer_internal(), 0);
    assert!(!bus.ch1_enabled());

    bus.write(NR14, 0xC7);
    assert_eq!(bus.ch1_length_timer_internal(), 63);
    assert!(bus.ch1_enabled());
}

#[test]
fn trigger_obscuro_carrega_63_em_vez_de_64_ch4() {
    let (_cpu, mut bus) = machine(&[]);

    bus.write(NR41, 0x3F);
    bus.write(NR42, 0xF1);
    bus.write(NR43, 0x00);
    bus.write(NR44, 0x87);
    assert!(bus.ch4_enabled());

    bus.write(NR44, 0x47);
    assert_eq!(bus.ch4_length_timer_internal(), 0);
    assert!(!bus.ch4_enabled());

    bus.write(NR44, 0xC7);
    assert_eq!(bus.ch4_length_timer_internal(), 63);
    assert!(bus.ch4_enabled());
}

#[test]
fn trigger_obscuro_carrega_255_em_vez_de_256_ch3() {
    let (_cpu, mut bus) = machine(&[]);

    bus.write(NR30, 0x80);
    bus.write(NR31, 0xFF);
    bus.write(NR32, 0x80);
    bus.write(NR33, 0x00);
    bus.write(NR34, 0x87);
    assert!(bus.ch3_enabled());

    bus.write(NR34, 0x47);
    assert_eq!(bus.ch3_length_timer_internal(), 0);
    assert!(!bus.ch3_enabled());

    bus.write(NR34, 0xC7);
    assert_eq!(bus.ch3_length_timer_internal(), 255);
    assert!(bus.ch3_enabled());
}

#[test]
fn trigger_normal_carrega_64_quando_next_step_e_length_ch2() {
    let (mut cpu, mut bus) = machine(&[]);

    step_n(&mut cpu, &mut bus, 3 * FS_MCYCLES);
    assert_eq!(bus.apu_frame_sequencer_step(), 3);

    bus.write(NR21, 0x00);
    bus.write(NR22, 0xF1);
    bus.write(NR23, 0x00);
    bus.write(NR24, 0xC7);

    assert_eq!(
        bus.ch2_length_timer_internal(),
        64,
        "trigger com next=length clock e length=0 carrega 64 normalmente"
    );
}

#[test]
fn extra_length_clocking_seguido_de_trigger_combina_casos_1_e_2() {
    let (_cpu, mut bus) = machine(&[]);

    bus.write(NR21, 0x3F);
    bus.write(NR22, 0xF1);
    bus.write(NR23, 0x00);
    bus.write(NR24, 0x87);
    assert_eq!(bus.ch2_length_timer_internal(), 1);
    assert_eq!(bus.apu_frame_sequencer_step(), 0);

    bus.write(NR24, 0xC7);

    assert_eq!(bus.ch2_length_timer_internal(), 63);
    assert!(bus.ch2_enabled());
}

#[test]
fn extra_length_clocking_com_trigger_e_next_step_length_carrega_64_normal() {
    let (mut cpu, mut bus) = machine(&[]);

    step_n(&mut cpu, &mut bus, FS_MCYCLES);
    assert_eq!(bus.apu_frame_sequencer_step(), 1);

    bus.write(NR21, 0x3F);
    bus.write(NR22, 0xF1);
    bus.write(NR23, 0x00);
    bus.write(NR24, 0x87);

    bus.write(NR24, 0xC7);

    assert_eq!(bus.ch2_length_timer_internal(), 1);
    assert!(bus.ch2_enabled());
}
