//! ROADMAP 6.7a — Downsample acumulador + ring buffer.
//! spec: docs/reference/07-apu.md § Audio Details.

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

const NR50: u16 = 0xFF24;
const NR51: u16 = 0xFF25;
const NR52: u16 = 0xFF26;

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

fn trigger_ch1(bus: &mut Bus, duty: u8, initial_volume: u8) {
    bus.write(NR10, 0x00);
    bus.write(NR11, duty << 6);
    bus.write(NR12, (initial_volume << 4) | 0x01);
    bus.write(NR13, 0x00);
    bus.write(NR14, 0x87);
}

fn trigger_ch2(bus: &mut Bus, duty: u8, initial_volume: u8) {
    bus.write(NR21, duty << 6);
    bus.write(NR22, (initial_volume << 4) | 0x01);
    bus.write(NR23, 0x00);
    bus.write(NR24, 0x87);
}

#[test]
fn nenhuma_amostra_disponivel_no_inicio() {
    let (_cpu, bus) = machine(&[]);
    assert_eq!(bus.audio_samples_available(), 0);
    assert!(bus.audio_samples().is_empty());
}

#[test]
fn produz_uma_amostra_apos_ciclos_suficientes() {
    let (_cpu, mut bus) = machine(&[]);
    let mut cpu = Cpu::after_boot_rom(
        CartridgeHeader::parse(&vec![0x00; NoMbc::MAX_ROM_LEN])
            .expect("cabeçalho")
            .checksum(),
    );

    step_n(&mut cpu, &mut bus, 22);

    assert!(
        bus.audio_samples_available() >= 1,
        "uma amostra depois de {} M-cycles",
        22
    );
}

#[test]
fn acumula_multiplas_amostras_ao_longo_do_tempo() {
    let (_cpu, mut bus) = machine(&[]);
    let mut cpu = Cpu::after_boot_rom(
        CartridgeHeader::parse(&vec![0x00; NoMbc::MAX_ROM_LEN])
            .expect("cabeçalho")
            .checksum(),
    );

    step_n(&mut cpu, &mut bus, 1000);

    assert!(bus.audio_samples_available() >= 30);
}

#[test]
fn consumir_amostras_reduz_disponiveis() {
    let (_cpu, mut bus) = machine(&[]);
    let mut cpu = Cpu::after_boot_rom(
        CartridgeHeader::parse(&vec![0x00; NoMbc::MAX_ROM_LEN])
            .expect("cabeçalho")
            .checksum(),
    );

    step_n(&mut cpu, &mut bus, 200);
    let antes = bus.audio_samples_available();
    assert!(antes > 0);

    let n = antes.min(4);
    bus.consume_audio_samples(n);
    assert_eq!(bus.audio_samples_available(), antes - n);
}

#[test]
fn amostras_sao_fatia_contigua_do_ring_buffer() {
    let (_cpu, mut bus) = machine(&[]);
    let mut cpu = Cpu::after_boot_rom(
        CartridgeHeader::parse(&vec![0x00; NoMbc::MAX_ROM_LEN])
            .expect("cabeçalho")
            .checksum(),
    );

    step_n(&mut cpu, &mut bus, 1000);
    let available = bus.audio_samples_available();
    assert!(available > 0);

    let samples = bus.audio_samples();
    assert_eq!(samples.len(), available);
}

#[test]
fn disponiveis_continuam_iguais_a_fatia_depois_da_volta_do_anel() {
    let (_cpu, mut bus) = machine(&[]);
    let mut cpu = Cpu::after_boot_rom(
        CartridgeHeader::parse(&vec![0x00; NoMbc::MAX_ROM_LEN])
            .expect("cabeçalho")
            .checksum(),
    );

    // O anel tem 4096 posições e enche a cada ~22 M-cycles por amostra: 500 mil
    // passos sem consumir dão a volta com folga.
    step_n(&mut cpu, &mut bus, 500_000);

    let available = bus.audio_samples_available();
    assert!(available > 0, "500 mil M-cycles produzem amostras");
    assert_eq!(
        bus.audio_samples().len(),
        available,
        "o gb-desktop faz audio_samples()[..available]: fatia menor que o contador é pânico em runtime"
    );
}

#[test]
fn apu_desligado_acumula_silencio() {
    let (_cpu, mut bus) = machine(&[]);
    let mut cpu = Cpu::after_boot_rom(
        CartridgeHeader::parse(&vec![0x00; NoMbc::MAX_ROM_LEN])
            .expect("cabeçalho")
            .checksum(),
    );

    bus.write(NR52, 0x00);

    step_n(&mut cpu, &mut bus, 200);
    let available = bus.audio_samples_available();
    assert!(available > 0);

    let samples = bus.audio_samples();
    for &(l, r) in samples.iter() {
        assert_eq!(l, 0.0, "esquerdo com APU desligado");
        assert_eq!(r, 0.0, "direito com APU desligado");
    }
}

#[test]
fn valor_medio_reflete_saida_do_mixer() {
    let (_cpu, mut bus) = machine(&[]);
    let mut cpu = Cpu::after_boot_rom(
        CartridgeHeader::parse(&vec![0x00; NoMbc::MAX_ROM_LEN])
            .expect("cabeçalho")
            .checksum(),
    );

    let (raw_l, raw_r) = bus.mixer_sample();
    assert_eq!((raw_l, raw_r), (0, 0), "mixer_sample depois do boot");

    step_n(&mut cpu, &mut bus, 200);
    let available = bus.audio_samples_available();
    assert!(available > 0);

    let samples = bus.audio_samples();
    for &(l, r) in samples.iter() {
        assert!(
            (-1.0..=1.0).contains(&l),
            "canal esquerdo {} fora de [-1, 1]",
            l
        );
        assert!(
            (-1.0..=1.0).contains(&r),
            "canal direito {} fora de [-1, 1]",
            r
        );
    }
}

#[test]
fn ring_buffer_sobrescreve_quando_cheio() {
    let (_cpu, mut bus) = machine(&[]);
    let mut cpu = Cpu::after_boot_rom(
        CartridgeHeader::parse(&vec![0x00; NoMbc::MAX_ROM_LEN])
            .expect("cabeçalho")
            .checksum(),
    );

    step_n(&mut cpu, &mut bus, 120_000);

    assert!(
        bus.audio_samples_available() <= 4096,
        "ring buffer não pode ter mais que 4096"
    );
}

#[test]
fn canal_ligado_produz_saida_nao_silenciosa() {
    let (_cpu, mut bus) = machine(&[]);
    let mut cpu = Cpu::after_boot_rom(
        CartridgeHeader::parse(&vec![0x00; NoMbc::MAX_ROM_LEN])
            .expect("cabeçalho")
            .checksum(),
    );

    bus.write(NR50, 0x77);
    bus.write(NR51, 0x11);
    bus.write(NR52, 0x80);

    trigger_ch1(&mut bus, 0b10, 0x0F);

    assert!(bus.ch1_enabled());

    step_n(&mut cpu, &mut bus, 5000);
    let available = bus.audio_samples_available();
    assert!(available > 0);

    let samples = bus.audio_samples();
    let max_l = samples
        .iter()
        .map(|&(l, _)| l)
        .fold(f32::NEG_INFINITY, f32::max);
    let min_l = samples
        .iter()
        .map(|&(l, _)| l)
        .fold(f32::INFINITY, f32::min);
    let amplitude = max_l - min_l;
    assert!(
        amplitude > 0.01,
        "canal ligado deveria produzir amplitude > 0.01, mas foi {}",
        amplitude
    );
}

#[test]
fn dois_canais_somam_linearmente() {
    let (_cpu, mut bus) = machine(&[]);
    let mut cpu = Cpu::after_boot_rom(
        CartridgeHeader::parse(&vec![0x00; NoMbc::MAX_ROM_LEN])
            .expect("cabeçalho")
            .checksum(),
    );

    bus.write(NR50, 0x77);
    bus.write(NR51, 0x11);
    bus.write(NR52, 0x80);

    trigger_ch1(&mut bus, 0b10, 0x0F);
    trigger_ch2(&mut bus, 0b10, 0x0F);

    step_n(&mut cpu, &mut bus, 5000);
    let samples = bus.audio_samples();
    assert!(!samples.is_empty());

    let max_l = samples
        .iter()
        .map(|&(l, _)| l)
        .fold(f32::NEG_INFINITY, f32::max);
    let min_l = samples
        .iter()
        .map(|&(l, _)| l)
        .fold(f32::INFINITY, f32::min);

    let amplitude = max_l - min_l;
    assert!(
        amplitude > 0.01,
        "dois canais deveriam ter amplitude apreciável, mas foi {}",
        amplitude
    );
}
