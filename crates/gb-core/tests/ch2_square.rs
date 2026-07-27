//! ROADMAP 6.2 — Canal 2: square sem sweep.
//! spec: docs/reference/07-apu.md § Sound Channel 2, § Pulse channels.

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::Cpu;

const NR21: u16 = 0xFF16;
const NR22: u16 = 0xFF17;
const NR23: u16 = 0xFF18;
const NR24: u16 = 0xFF19;

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
fn ch2_comeca_desligado() {
    let (_cpu, bus) = machine(&[]);

    assert!(!bus.ch2_enabled(), "canal 2 começa desligado após o boot");
}

#[test]
fn escrever_nr21_configura_duty_cycle() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR21, 0x40);
    assert_eq!(
        bus.ch2_duty_pattern(),
        0x01,
        "NR21 bits 7-6 = 01 configura duty cycle de 25%"
    );

    bus.write(NR21, 0xC0);
    assert_eq!(
        bus.ch2_duty_pattern(),
        0x03,
        "NR21 bits 7-6 = 11 configura duty cycle de 75%"
    );
}

#[test]
fn escrever_nr22_configura_volume_inicial_e_envelope() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR22, 0xF2);

    assert_eq!(
        bus.ch2_initial_volume(),
        0x0F,
        "NR22 bits 7-4 = $F configura volume inicial"
    );
    assert_eq!(
        bus.ch2_envelope_pace(),
        0x02,
        "NR22 bits 2-0 = 2 configura o pace do envelope"
    );
    assert!(
        bus.ch2_dac_enabled(),
        "NR22 bits 7-3 != 0 → DAC está ligado"
    );
}

#[test]
fn nr22_com_bits_7_a_3_zerados_desliga_o_dac() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR22, 0x07);

    assert!(
        !bus.ch2_dac_enabled(),
        "NR22 & $F8 == 0 → DAC está desligado"
    );
}

#[test]
fn periodo_de_11_bits_combina_nr23_e_nr24() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR23, 0x00);
    bus.write(NR24, 0x05);

    assert_eq!(
        bus.ch2_period(),
        0x0500u16,
        "NR23=$00 + NR24 bits 2-0=5 → período = $0500"
    );
}

#[test]
fn trigger_do_ch2_liga_o_canal_e_carrega_freq_timer_do_periodo() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR23, 0x00);
    bus.write(NR24, 0x85);

    assert!(
        bus.ch2_enabled(),
        "escrever NR24 com bit 7=1 liga o canal 2"
    );
    assert_eq!(
        bus.ch2_frequency_timer(),
        0x0500u16,
        "freq_timer é carregado com o período ($0500) no trigger"
    );
}

#[test]
fn freq_timer_do_ch2_avanca_4_por_m_cycle_e_sofre_overflow() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR23, 0x00);
    bus.write(NR24, 0x85);

    // freq_timer começa em $0500 = 1280. Overflow em $0800 = 2048.
    // 2048 - 1280 = 768 T-cycles = 192 M-cycles.
    let (mut cpu, mut bus) = (
        Cpu::after_boot_rom(CartridgeHeader::parse(&[0x00; 32768]).expect("").checksum()),
        bus,
    );

    // Recria a máquina com o mesmo estado da IO
    bus.write(NR23, 0x00);
    bus.write(NR24, 0x85);

    // 191 M-cycles: freq_timer = 1280 + 764 = 2044, ainda não overflow
    step_n(&mut cpu, &mut bus, 191);
    assert_eq!(
        bus.ch2_frequency_timer(),
        2044,
        "depois de 191 M-cycles o freq_timer é 2044 (sem overflow ainda)"
    );

    // +1 M-cycle: 2044 + 4 = 2048 → overflow, recarrega $0500
    step_n(&mut cpu, &mut bus, 1);
    assert_eq!(
        bus.ch2_frequency_timer(),
        0x0500,
        "após 192 M-cycles o freq_timer sofre overflow e recarrega $0500"
    );
}

#[test]
fn duty_step_avanca_no_overflow_do_freq_timer() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(NR23, 0x00);
    bus.write(NR24, 0x85);

    assert_eq!(
        bus.ch2_duty_step(),
        0,
        "duty_step começa em 0 após o trigger"
    );

    // 192 M-cycles = 1 overflow → duty_step avança para 1
    step_n(&mut cpu, &mut bus, 192);
    assert_eq!(
        bus.ch2_duty_step(),
        1,
        "duty_step avança para 1 após o primeiro overflow do freq_timer"
    );

    // +192 M-cycles = 2º overflow → duty_step = 2
    step_n(&mut cpu, &mut bus, 192);
    assert_eq!(
        bus.ch2_duty_step(),
        2,
        "duty_step avança para 2 após o segundo overflow do freq_timer"
    );
}

#[test]
fn envelope_do_ch2_diminui_somente_no_passo_7_e_nao_no_passo_2() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(NR22, 0xF1);
    bus.write(NR23, 0x00);
    bus.write(NR24, 0x87);

    assert_eq!(bus.ch2_envelope_volume(), 15);

    // Avançar até o passo 2 (4096 M-cycles): o envelope NÃO deve disparar aqui.
    step_n(&mut cpu, &mut bus, 4096);
    assert_eq!(bus.apu_frame_sequencer_step(), 2);
    assert_eq!(
        bus.ch2_envelope_volume(),
        15,
        "no passo 2 o envelope NÃO dispara — o volume continua 15"
    );

    // Avançar até o passo 7 (+10240 = 14336 M-cycles total): o envelope deve disparar.
    step_n(&mut cpu, &mut bus, 10240);
    assert_eq!(bus.apu_frame_sequencer_step(), 7);
    assert_eq!(
        bus.ch2_envelope_volume(),
        14,
        "no passo 7, com pace=1, o volume do envelope diminui de 15 para 14"
    );
}

#[test]
fn envelope_do_ch2_carrega_volume_do_trigger_e_diminui_no_passo_7() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(NR22, 0xF1);
    bus.write(NR23, 0x00);
    bus.write(NR24, 0x87);

    assert_eq!(
        bus.ch2_envelope_volume(),
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
        bus.ch2_envelope_volume(),
        14,
        "no passo 7, com pace=1, o volume do envelope diminui de 15 para 14"
    );
}

#[test]
fn envelope_do_ch2_diminui_de_novo_no_segundo_passo_7() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(NR22, 0xF1);
    bus.write(NR23, 0x00);
    bus.write(NR24, 0x87);

    // 15 * 2048 = 30720 M-cycles: um ciclo completo (passos 0–7) + mais 7 passos = passo 7 do segundo ciclo.
    step_n(&mut cpu, &mut bus, 30720);
    assert_eq!(
        bus.apu_frame_sequencer_step(),
        7,
        "depois de 30720 M-cycles o frame sequencer está no passo 7 do segundo ciclo"
    );
    assert_eq!(
        bus.ch2_envelope_volume(),
        13,
        "no segundo passo 7, com pace=1, o volume diminui mais uma vez: 14 → 13"
    );
}

#[test]
fn envelope_com_pace_2_salta_um_clock() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(NR22, 0xF2);
    bus.write(NR23, 0x00);
    bus.write(NR24, 0x87);

    assert_eq!(bus.ch2_envelope_volume(), 15);

    // Primeiro passo 7: pace=2 → timer decrementa de 2 para 1, volume NÃO muda.
    step_n(&mut cpu, &mut bus, 14336);
    assert_eq!(bus.apu_frame_sequencer_step(), 7);
    assert_eq!(
        bus.ch2_envelope_volume(),
        15,
        "no primeiro passo 7, com pace=2, o volume ainda é 15 (timer foi de 2→1)"
    );

    // Segundo passo 7: timer decrementa de 1 para 0, recarrega 2, volume diminui para 14.
    step_n(&mut cpu, &mut bus, 16384);
    assert_eq!(bus.apu_frame_sequencer_step(), 7);
    assert_eq!(
        bus.ch2_envelope_volume(),
        14,
        "no segundo passo 7, com pace=2, o volume diminui para 14"
    );
}

#[test]
fn envelope_com_pace_0_nao_altera_volume() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(NR22, 0xF0);
    bus.write(NR23, 0x00);
    bus.write(NR24, 0x87);

    assert_eq!(bus.ch2_envelope_volume(), 15);

    // Avançar até o passo 7 (14336 M-cycles).
    step_n(&mut cpu, &mut bus, 14336);
    assert_eq!(
        bus.ch2_envelope_volume(),
        15,
        "com pace=0 o envelope está desabilitado e o volume não muda"
    );
}

#[test]
fn envelope_com_direcao_1_aumenta_volume() {
    let (mut cpu, mut bus) = machine(&[]);

    // volume inicial = 0, direção = increase, pace = 1
    bus.write(NR22, 0x09);
    bus.write(NR23, 0x00);
    bus.write(NR24, 0x87);

    assert_eq!(bus.ch2_envelope_volume(), 0);

    step_n(&mut cpu, &mut bus, 14336);
    assert_eq!(bus.apu_frame_sequencer_step(), 7);
    assert_eq!(
        bus.ch2_envelope_volume(),
        1,
        "no passo 7, com direção=increase e pace=1, o volume sobe para 1"
    );
}

#[test]
fn envelope_do_ch2_nao_dispara_se_canal_estiver_desligado() {
    let (mut cpu, mut bus) = machine(&[]);

    // Configurar o envelope mas NÃO disparar o canal
    bus.write(NR22, 0xF1);
    bus.write(NR23, 0x00);

    assert!(!bus.ch2_enabled());

    step_n(&mut cpu, &mut bus, 14336);
    assert_eq!(bus.apu_frame_sequencer_step(), 7);
    assert_eq!(
        bus.ch2_envelope_volume(),
        0,
        "sem trigger, o volume do envelope permanece 0"
    );
}
