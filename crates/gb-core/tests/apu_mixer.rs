//! ROADMAP 6.6 — Mixer: NR50/NR51/NR52, panning, DAC enable.
//! spec: docs/reference/07-apu.md § Global control registers, § Mixer, § DACs.

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
const NR32: u16 = 0xFF1C;
const NR33: u16 = 0xFF1D;
const NR34: u16 = 0xFF1E;

const NR41: u16 = 0xFF20;
const NR42: u16 = 0xFF21;
const NR43: u16 = 0xFF22;
const NR44: u16 = 0xFF23;

const NR50: u16 = 0xFF24;
const NR51: u16 = 0xFF25;
const NR52: u16 = 0xFF26;

const WAVE_RAM_BASE: u16 = 0xFF30;

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

fn trigger_ch1_default(bus: &mut Bus) {
    trigger_ch1(bus, 0b10, 0x0F);
}

fn trigger_ch2(bus: &mut Bus, duty: u8, initial_volume: u8) {
    bus.write(NR21, duty << 6);
    bus.write(NR22, (initial_volume << 4) | 0x01);
    bus.write(NR23, 0x00);
    bus.write(NR24, 0x87);
}

fn trigger_ch2_default(bus: &mut Bus) {
    trigger_ch2(bus, 0b10, 0x0F);
}

fn trigger_ch3(bus: &mut Bus, nr32: u8) {
    for addr in WAVE_RAM_BASE..WAVE_RAM_BASE + 16 {
        bus.write(addr, 0xFF);
    }
    bus.write(NR30, 0x80);
    bus.write(NR32, nr32);
    bus.write(NR33, 0x00);
    bus.write(NR34, 0x87);
}

fn trigger_ch4(bus: &mut Bus, initial_volume: u8) {
    bus.write(NR41, 0x00);
    bus.write(NR42, (initial_volume << 4) | 0x01);
    bus.write(NR43, 0x01);
    bus.write(NR44, 0x80);
}

fn trigger_ch4_default(bus: &mut Bus) {
    trigger_ch4(bus, 0x0F);
}

// ── NR52 ────────────────────────────────────────────────────────────────

#[test]
fn nr52_bits_0_a_3_sao_readonly_e_refletem_chx_enabled() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    assert!(!bus.ch1_enabled());
    assert!(!bus.ch2_enabled());
    assert!(!bus.ch3_enabled());
    assert!(!bus.ch4_enabled());

    let status = bus.read(NR52);
    assert_eq!(
        status & 0x0F,
        0x00,
        "todos os canais desligados → bits 0-3 = 0000"
    );

    trigger_ch1_default(&mut bus);

    let status = bus.read(NR52);
    assert_eq!(status & 0x01, 0x01, "CH1 ligado → bit 0 = 1");
    assert_eq!(status & 0x0E, 0x00, "CH2/CH3/CH4 continuam 0");

    trigger_ch2_default(&mut bus);
    assert_eq!(
        bus.read(NR52) & 0x03,
        0x03,
        "CH1 e CH2 ligados → bits 0-1 = 11"
    );

    trigger_ch4_default(&mut bus);
    assert_eq!(
        bus.read(NR52) & 0x0B,
        0x0B,
        "CH1, CH2 e CH4 → bits 0,1,3 = 1011"
    );
}

#[test]
fn escrita_nos_bits_0_a_3_do_nr52_nao_liga_canais() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR52, 0x8F);

    assert!(!bus.ch1_enabled(), "escrever bit 0 nao liga CH1");
    assert!(!bus.ch2_enabled(), "escrever bit 1 nao liga CH2");
    assert!(!bus.ch3_enabled(), "escrever bit 2 nao liga CH3");
    assert!(!bus.ch4_enabled(), "escrever bit 3 nao liga CH4");

    let status = bus.read(NR52);
    assert_eq!(
        status & 0x0F,
        0x00,
        "bits 0-3 continuam lendo o estado real, nao o valor escrito"
    );
}

#[test]
fn nr52_bit_7_power_off_nao_afeta_wave_ram() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    for addr in WAVE_RAM_BASE..WAVE_RAM_BASE + 16 {
        bus.write(addr, (addr & 0xFF) as u8);
    }

    bus.write(NR52, 0x00);
    bus.write(NR52, 0x80);

    for addr in WAVE_RAM_BASE..WAVE_RAM_BASE + 16 {
        assert_eq!(
            bus.read(addr),
            (addr & 0xFF) as u8,
            "wave RAM sobrevive ao power-off"
        );
    }
}

#[test]
fn nr52_power_off_corta_saida_do_mixer() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    trigger_ch1_default(&mut bus);
    bus.write(NR51, 0x11);
    bus.write(NR50, 0x77);

    let (left, right) = bus.mixer_sample();
    assert!(left > 0 || right > 0, "com power on, canal produz saida");

    bus.write(NR52, 0x00);

    let (left, right) = bus.mixer_sample();
    assert_eq!(left, 0, "power off → saida esquerda 0");
    assert_eq!(right, 0, "power off → saida direita 0");
}

// ── NR50 ─────────────────────────────────────────────────────────────────

#[test]
fn nr50_leitura_e_escrita() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR50, 0x12);
    assert_eq!(bus.read(NR50), 0x12, "NR50 armazena o valor escrito");
}

#[test]
fn nr50_volume_0_escala_1_nao_muta() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    trigger_ch1_default(&mut bus);
    bus.write(NR51, 0x11);

    bus.write(NR50, 0x77);
    let (left_ref, right_ref) = bus.mixer_sample();

    bus.write(NR50, 0x00);
    let (left, right) = bus.mixer_sample();

    assert!(left > 0, "volume 0 nao deve mutar o canal esquerdo");
    assert!(right > 0, "volume 0 nao deve mutar o canal direito");
    assert!(left < left_ref, "volume 0 deve ser menor que volume 7");
    assert!(right < right_ref, "volume 0 deve ser menor que volume 7");
}

#[test]
fn nr50_volume_7_escala_8_sem_reducao() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    trigger_ch1_default(&mut bus);
    bus.write(NR51, 0x11);

    bus.write(NR50, 0x00);
    let (l0, _) = bus.mixer_sample();

    bus.write(NR50, 0x70);
    let (l7, _) = bus.mixer_sample();

    assert_eq!(l7, l0 * 8, "volume 7 (8×) deve ser 8 vezes volume 0 (1×)");
}

// ── NR51 ─────────────────────────────────────────────────────────────────

#[test]
fn nr51_leitura_e_escrita() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR51, 0xAA);
    assert_eq!(bus.read(NR51), 0xAA, "NR51 armazena o valor escrito");
}

#[test]
fn nr51_ch1_apenas_na_esquerda() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    trigger_ch1_default(&mut bus);
    bus.write(NR50, 0x77);
    bus.write(NR51, 0x10);

    let (left, right) = bus.mixer_sample();
    assert!(left > 0, "CH1 roteado para esquerda → saida > 0");
    assert_eq!(right, 0, "CH1 NAO roteado para direita → saida = 0");
}

#[test]
fn nr51_ch1_apenas_na_direita() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    trigger_ch1_default(&mut bus);
    bus.write(NR50, 0x77);
    bus.write(NR51, 0x01);

    let (left, right) = bus.mixer_sample();
    assert_eq!(left, 0, "CH1 NAO roteado para esquerda → saida = 0");
    assert!(right > 0, "CH1 roteado para direita → saida > 0");
}

#[test]
fn nr51_ch1_em_ambos_os_canais() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    trigger_ch1_default(&mut bus);
    bus.write(NR50, 0x77);
    bus.write(NR51, 0x11);

    let (left, right) = bus.mixer_sample();
    assert!(left > 0, "esquerda recebe CH1");
    assert!(right > 0, "direita recebe CH1");
    assert_eq!(left, right, "mesmo canal nos dois lados → mesma amplitude");
}

#[test]
fn nr51_canais_diferentes_em_lados_diferentes() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    trigger_ch1(&mut bus, 0b10, 15);
    trigger_ch2(&mut bus, 0b10, 5);
    bus.write(NR50, 0x77);
    bus.write(NR51, 0x12);

    let (left, right) = bus.mixer_sample();
    assert!(left > 0, "CH1 na esquerda → saida > 0");
    assert!(right > 0, "CH2 na direita → saida > 0");
    assert!(left > right, "CH1 volume 15 > CH2 volume 5 na esquerda");
}

// ── DAC ──────────────────────────────────────────────────────────────────

#[test]
fn ch1_com_dac_desligado_nao_contribui_no_mixer() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR11, 0x00);
    bus.write(NR12, 0x00);
    bus.write(NR13, 0x00);
    bus.write(NR14, 0x87);

    assert!(!bus.ch1_dac_enabled(), "NR12 = $00 → DAC off");

    bus.write(NR51, 0x11);
    bus.write(NR50, 0x77);

    let (left, right) = bus.mixer_sample();
    assert_eq!(left, 0, "DAC desligado → saida esquerda 0");
    assert_eq!(right, 0, "DAC desligado → saida direita 0");
}

#[test]
fn ch1_com_dac_ligado_mas_canal_desligado_tem_saida_zero() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR12, 0xF1);
    assert!(bus.ch1_dac_enabled(), "NR12 = $F1 → DAC on");

    bus.write(NR51, 0x11);
    bus.write(NR50, 0x77);

    let (left, right) = bus.mixer_sample();
    assert_eq!(left, 0, "DAC on mas canal desligado → saida 0");
    assert_eq!(right, 0, "DAC on mas canal desligado → saida 0");
}

#[test]
fn ch3_com_dac_desligado_nao_contribui_no_mixer() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    for addr in WAVE_RAM_BASE..WAVE_RAM_BASE + 16 {
        bus.write(addr, 0xFF);
    }
    bus.write(NR30, 0x00);
    bus.write(NR32, 0x20);
    bus.write(NR33, 0x00);
    bus.write(NR34, 0x87);

    bus.write(NR51, 0x44);
    bus.write(NR50, 0x77);

    let (left, right) = bus.mixer_sample();
    assert_eq!(left, 0, "CH3 DAC off → saida esquerda 0");
    assert_eq!(right, 0, "CH3 DAC off → saida direita 0");
}

// ── Pulse channel output ─────────────────────────────────────────────────

#[test]
fn ch1_duty_50_pct_com_envelope_15_produz_saida_15() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    trigger_ch1(&mut bus, 0b10, 15);
    bus.write(NR51, 0x11);
    bus.write(NR50, 0x77);

    let (left, right) = bus.mixer_sample();
    assert_eq!(
        left,
        15 * 8,
        "duty 50% step 0 → alto, volume 15 × NR50 esquerda 8"
    );
    assert_eq!(right, 15 * 8, "mesmo na direita");
}

#[test]
fn ch1_duty_75_pct_step_0_eh_baixo() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    trigger_ch1(&mut bus, 0b11, 10);
    bus.write(NR51, 0x11);
    bus.write(NR50, 0x77);

    let (left, right) = bus.mixer_sample();
    assert_eq!(left, 0, "duty 75% step 0 → baixo, saida 0");
    assert_eq!(right, 0, "mesmo na direita");
}

// ── Wave channel output ──────────────────────────────────────────────────

#[test]
fn ch3_output_level_100_pct_passa_o_nibble_como_saida() {
    let (mut cpu, bus) = machine(&[]);
    let mut bus = bus;

    trigger_ch3(&mut bus, 0x20);
    step_n(&mut cpu, &mut bus, 32);

    bus.write(NR51, 0x44);
    bus.write(NR50, 0x77);

    let (left, right) = bus.mixer_sample();
    assert!(left > 0, "CH3 com wave RAM $FF e output 100% produz saida");
    assert!(right > 0, "CH3 nos dois canais");
}

#[test]
fn ch3_output_level_0_muta_o_canal() {
    let (mut cpu, bus) = machine(&[]);
    let mut bus = bus;

    trigger_ch3(&mut bus, 0x00);
    step_n(&mut cpu, &mut bus, 32);

    bus.write(NR51, 0x44);
    bus.write(NR50, 0x77);

    let (left, right) = bus.mixer_sample();
    assert_eq!(left, 0, "CH3 output level 0 → saida 0");
    assert_eq!(right, 0, "CH3 output level 0 → saida 0");
}

#[test]
fn ch3_output_level_50_pct_reduz_pela_metade() {
    let (mut cpu, bus) = machine(&[]);
    let mut bus = bus;

    for addr in WAVE_RAM_BASE..WAVE_RAM_BASE + 16 {
        bus.write(addr, 0xEE);
    }
    bus.write(NR30, 0x80);
    bus.write(NR32, 0x20);
    bus.write(NR33, 0x00);
    bus.write(NR34, 0x87);
    step_n(&mut cpu, &mut bus, 32);

    bus.write(NR51, 0x44);
    bus.write(NR50, 0x77);
    let (l100, _) = bus.mixer_sample();

    for addr in WAVE_RAM_BASE..WAVE_RAM_BASE + 16 {
        bus.write(addr, 0xEE);
    }
    bus.write(NR32, 0x40);
    bus.write(NR34, 0x87);
    step_n(&mut cpu, &mut bus, 32);

    bus.write(NR51, 0x44);
    bus.write(NR50, 0x77);
    let (l50, _) = bus.mixer_sample();

    assert_eq!(l100, l50 * 2, "output 100% deve ser o dobro de 50%");
}

// ── Noise channel output ─────────────────────────────────────────────────

#[test]
fn ch4_lfsr_bit0_zero_tem_saida_zero() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    trigger_ch4_default(&mut bus);
    bus.write(NR51, 0x88);
    bus.write(NR50, 0x77);

    let estado_lfsr = bus.ch4_lfsr_value();
    assert_eq!(estado_lfsr, 0, "LFSR resetado para 0 no trigger");

    let (left, right) = bus.mixer_sample();
    assert_eq!(left, 0, "LFSR bit 0 = 0 → saida 0 para esquerda");
    assert_eq!(right, 0, "LFSR bit 0 = 0 → saida 0 para direita");
}

// ── Soma de múltiplos canais ─────────────────────────────────────────────

#[test]
fn dois_canais_somam_linearmente_no_mesmo_lado() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    trigger_ch1_default(&mut bus);
    bus.write(NR51, 0x11);
    bus.write(NR50, 0x77);
    let (l1, r1) = bus.mixer_sample();

    trigger_ch2_default(&mut bus);
    bus.write(NR51, 0x22);
    bus.write(NR50, 0x77);
    let (l2, r2) = bus.mixer_sample();

    trigger_ch1_default(&mut bus);
    trigger_ch2_default(&mut bus);
    bus.write(NR51, 0x33);
    bus.write(NR50, 0x77);
    let (l12, r12) = bus.mixer_sample();

    assert_eq!(l12, l1 + l2, "esquerda: CH1 + CH2 = soma dos individuais");
    assert_eq!(r12, r1 + r2, "direita:  CH1 + CH2 = soma dos individuais");
}

// ── NR52 power-off comportamento ─────────────────────────────────────────

#[test]
fn nr52_power_off_zera_nr10_a_nr51_e_desliga_canais() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR10, 0xAB);
    bus.write(NR12, 0xCD);
    bus.write(NR14, 0xEF);
    bus.write(NR50, 0x33);
    bus.write(NR51, 0xCC);
    bus.write(NR52, 0x00);

    assert_eq!(bus.read(NR10), 0x80, "NR10 volta ao default pos power-off");
    assert_eq!(bus.read(NR50), 0x77, "NR50 volta ao default pos power-off");
    assert_eq!(bus.read(NR51), 0xF3, "NR51 volta ao default pos power-off");
    assert_eq!(bus.read(NR52) & 0x80, 0x00, "NR52 bit 7 = 0 apos power-off");
}

#[test]
fn nr52_power_off_desliga_todos_os_canais() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    trigger_ch1_default(&mut bus);
    trigger_ch2_default(&mut bus);
    trigger_ch4_default(&mut bus);

    assert!(bus.ch1_enabled());
    assert!(bus.ch2_enabled());
    assert!(bus.ch4_enabled());

    bus.write(NR52, 0x00);

    assert!(!bus.ch1_enabled(), "power-off desliga CH1");
    assert!(!bus.ch2_enabled(), "power-off desliga CH2");
    assert!(!bus.ch4_enabled(), "power-off desliga CH4");
}
