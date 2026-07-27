//! ROADMAP 6.5 — Canal 4: noise (LFSR de 15/7 bits).
//! spec: docs/reference/07-apu.md § Sound Channel 4 — Noise, § Noise channel (CH4).

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::Cpu;

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

#[test]
fn ch4_comeca_desligado() {
    let (_cpu, bus) = machine(&[]);

    assert!(!bus.ch4_enabled(), "canal 4 começa desligado após o boot");
}

#[test]
fn nr41_armazena_6_bits_do_length_timer_nos_bits_5_a_0() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR41, 0x3F);
    assert_eq!(
        bus.ch4_length_timer(),
        0x3F,
        "NR41 bits 5-0 = $3F → length timer máximo"
    );

    bus.write(NR41, 0xC0);
    assert_eq!(
        bus.ch4_length_timer(),
        0x00,
        "bits 7-6 de NR41 não afetam o length timer"
    );
}

#[test]
fn nr42_configura_volume_inicial_e_envelope() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR42, 0xF2);

    assert_eq!(
        bus.ch4_initial_volume(),
        0x0F,
        "NR42 bits 7-4 = $F configura volume inicial"
    );
    assert_eq!(
        bus.ch4_envelope_pace(),
        0x02,
        "NR42 bits 2-0 = 2 configura o pace do envelope"
    );
    assert!(
        bus.ch4_dac_enabled(),
        "NR42 bits 7-3 != 0 → DAC está ligado"
    );
}

#[test]
fn nr42_com_bits_7_a_3_zerados_desliga_o_dac() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR42, 0x07);

    assert!(
        !bus.ch4_dac_enabled(),
        "NR42 & $F8 == 0 → DAC está desligado"
    );
}

#[test]
fn nr43_configura_clock_shift_divider_e_lfsr_width() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR43, 0x2A);

    assert_eq!(
        bus.ch4_clock_shift(),
        2,
        "NR43 bits 7-4 = 2 configura clock shift"
    );
    assert!(bus.ch4_lfsr_width_7bit(), "NR43 bit 3 = 1 → LFSR de 7 bits");
    assert_eq!(
        bus.ch4_clock_divider(),
        2,
        "NR43 bits 2-0 = 2 configura clock divider"
    );
}

#[test]
fn nr43_com_bit_3_zero_e_lfsr_de_15_bits() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR43, 0x00);

    assert!(
        !bus.ch4_lfsr_width_7bit(),
        "NR43 bit 3 = 0 → LFSR de 15 bits (padrão)"
    );
}

#[test]
fn trigger_do_ch4_liga_o_canal_e_reseta_lfsr_para_zero() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR42, 0xF1);
    bus.write(NR44, 0x80);

    assert!(
        bus.ch4_enabled(),
        "escrever NR44 com bit 7=1 liga o canal 4"
    );
    assert_eq!(
        bus.ch4_lfsr_value(),
        0x0000,
        "LFSR é resetado para 0 no trigger"
    );
}

#[test]
fn trigger_com_dac_desligado_nao_liga_o_canal() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR44, 0x80);

    assert!(
        !bus.ch4_enabled(),
        "trigger não liga o canal 4 se o DAC (NR42) está desligado"
    );
}

#[test]
fn lfsr_avanca_com_clock_divider_1_e_shift_0() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(NR42, 0xF1);
    bus.write(NR43, 0x01);
    bus.write(NR44, 0x80);

    assert_eq!(
        bus.ch4_noise_threshold(),
        4,
        "threshold com divider=1 shift=0 → 4"
    );

    let lfsr_antes = bus.ch4_lfsr_value();

    step_n(&mut cpu, &mut bus, 1);
    assert_ne!(
        bus.ch4_lfsr_value(),
        lfsr_antes,
        "LFSR muda após 1 M-cycle com threshold = 4"
    );
}

#[test]
fn lfsr_nao_avanca_com_shift_14() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(NR43, 0xE0);
    bus.write(NR44, 0x80);

    assert_eq!(
        bus.ch4_noise_threshold(),
        u16::MAX,
        "threshold com shift=14 → u16::MAX (canal congelado)"
    );

    let lfsr_antes = bus.ch4_lfsr_value();
    step_n(&mut cpu, &mut bus, 100);
    assert_eq!(
        bus.ch4_lfsr_value(),
        lfsr_antes,
        "LFSR não avança quando shift ≥ 14"
    );
}

#[test]
fn lfsr_nao_avanca_com_shift_15() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(NR43, 0xF0);
    bus.write(NR44, 0x80);

    assert_eq!(
        bus.ch4_noise_threshold(),
        u16::MAX,
        "threshold com shift=15 → u16::MAX"
    );

    let lfsr_antes = bus.ch4_lfsr_value();
    step_n(&mut cpu, &mut bus, 100);
    assert_eq!(
        bus.ch4_lfsr_value(),
        lfsr_antes,
        "LFSR não avança quando shift = 15"
    );
}

#[test]
fn divider_0_e_tratado_como_metade() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR43, 0x00);

    assert_eq!(
        bus.ch4_noise_threshold(),
        2,
        "threshold com divider=0 shift=0 → 2 (metade de 4)"
    );

    bus.write(NR43, 0x10);

    assert_eq!(
        bus.ch4_noise_threshold(),
        4,
        "threshold com divider=0 shift=1 → 2 << 1 = 4"
    );
}

#[test]
fn envelope_do_ch4_carrega_volume_do_trigger_e_diminui_no_passo_7() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(NR42, 0xF1);
    bus.write(NR44, 0x80);

    assert_eq!(
        bus.ch4_envelope_volume(),
        15,
        "volume do envelope é o initial_volume (15) após o trigger"
    );

    step_n(&mut cpu, &mut bus, 14336);
    assert_eq!(
        bus.apu_frame_sequencer_step(),
        7,
        "depois de 14336 M-cycles o frame sequencer está no passo 7"
    );
    assert_eq!(
        bus.ch4_envelope_volume(),
        14,
        "no passo 7, com pace=1, o volume do envelope diminui de 15 para 14"
    );
}

#[test]
fn envelope_do_ch4_diminui_de_novo_no_segundo_passo_7() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(NR42, 0xF1);
    bus.write(NR44, 0x80);

    // 15 * 2048 = 30720 M-cycles: um ciclo completo + 7 passos = passo 7 do segundo ciclo.
    step_n(&mut cpu, &mut bus, 30720);
    assert_eq!(
        bus.apu_frame_sequencer_step(),
        7,
        "depois de 30720 M-cycles o frame sequencer está no passo 7 do segundo ciclo"
    );
    assert_eq!(
        bus.ch4_envelope_volume(),
        13,
        "no segundo passo 7, com pace=1, o volume diminui mais uma vez: 14 → 13"
    );
}

#[test]
fn envelope_com_pace_0_nao_altera_volume() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(NR42, 0xF0);
    bus.write(NR44, 0x80);

    assert_eq!(bus.ch4_envelope_volume(), 15);

    // Avançar até o passo 7 (14336 M-cycles).
    step_n(&mut cpu, &mut bus, 14336);
    assert_eq!(
        bus.ch4_envelope_volume(),
        15,
        "com pace=0 o envelope está desabilitado e o volume não muda"
    );
}

#[test]
fn envelope_com_direcao_1_aumenta_volume() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(NR42, 0x09);
    bus.write(NR44, 0x80);

    assert_eq!(bus.ch4_envelope_volume(), 0);

    step_n(&mut cpu, &mut bus, 14336);
    assert_eq!(
        bus.ch4_envelope_volume(),
        1,
        "no passo 7, com direção=increase e pace=1, o volume sobe para 1"
    );
}

#[test]
fn envelope_do_ch4_nao_dispara_se_canal_estiver_desligado() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(NR42, 0xF1);

    assert!(!bus.ch4_enabled());

    step_n(&mut cpu, &mut bus, 14336);
    assert_eq!(bus.apu_frame_sequencer_step(), 7);
    assert_eq!(
        bus.ch4_envelope_volume(),
        0,
        "sem trigger, o volume do envelope permanece 0"
    );
}

#[test]
fn ch4_nao_avanca_lfsr_se_canal_estiver_desligado() {
    let (mut cpu, mut bus) = machine(&[]);

    assert!(!bus.ch4_enabled());

    let lfsr_antes = bus.ch4_lfsr_value();
    step_n(&mut cpu, &mut bus, 100);
    assert_eq!(
        bus.ch4_lfsr_value(),
        lfsr_antes,
        "LFSR não avança quando canal 4 está desligado"
    );
}

#[test]
fn lfsr_em_modo_15_bits_produz_sequencia_pseudoaleatoria() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(NR42, 0xF1);
    bus.write(NR43, 0x01);
    bus.write(NR44, 0x80);

    assert_eq!(bus.ch4_lfsr_value(), 0x0000);

    step_n(&mut cpu, &mut bus, 1);
    let v1 = bus.ch4_lfsr_value();
    step_n(&mut cpu, &mut bus, 1);
    let v2 = bus.ch4_lfsr_value();
    step_n(&mut cpu, &mut bus, 1);
    let v3 = bus.ch4_lfsr_value();

    assert_ne!(v1, v2, "sequência pseudoaleatória: v1 ≠ v2");
    assert_ne!(v2, v3, "sequência pseudoaleatória: v2 ≠ v3");
}

#[test]
fn trigger_define_freq_timer_em_zero() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR43, 0x01);
    bus.write(NR44, 0x80);

    assert_eq!(
        bus.ch4_frequency_timer(),
        0,
        "freq_timer começa em 0 após o trigger"
    );
}
