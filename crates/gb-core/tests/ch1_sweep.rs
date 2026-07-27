//! ROADMAP 6.3 — Canal 1: square + sweep de frequência.
//! spec: docs/reference/07-apu.md § Sound Channel 1, § Pulse channel with sweep (CH1).

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::Cpu;

const NR10: u16 = 0xFF10;
const NR12: u16 = 0xFF12;
const NR13: u16 = 0xFF13;
const NR14: u16 = 0xFF14;

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
fn ch1_comeca_desligado() {
    let (_cpu, bus) = machine(&[]);
    assert!(!bus.ch1_enabled(), "canal 1 começa desligado após o boot");
}

#[test]
fn nr10_configura_pace_direction_e_step() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR10, 0x7B);

    assert_eq!(
        bus.ch1_sweep_pace(),
        7,
        "NR10 bits 6-4 = 7 configura pace do sweep"
    );
    assert_eq!(
        bus.ch1_sweep_direction(),
        1,
        "NR10 bit 3 = 1 configura direção subtract"
    );
    assert_eq!(
        bus.ch1_sweep_step(),
        3,
        "NR10 bits 2-0 = 3 configura step individual"
    );
}

#[test]
fn trigger_do_ch1_liga_o_canal_e_carrega_freq_timer() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR13, 0x00);
    bus.write(NR14, 0x85);

    assert!(
        bus.ch1_enabled(),
        "escrever NR14 com bit 7=1 liga o canal 1"
    );
    assert_eq!(
        bus.ch1_frequency_timer(),
        0x0500u16,
        "freq_timer do CH1 é carregado com o período ($0500) no trigger"
    );
}

#[test]
fn trigger_copia_periodo_para_shadow_register() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR13, 0x00);
    bus.write(NR14, 0x85);

    assert_eq!(
        bus.ch1_sweep_shadow(),
        0x0500u16,
        "shadow register do sweep é carregado com o período no trigger"
    );
}

#[test]
fn trigger_reseta_sweep_timer() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR10, 0x73);
    bus.write(NR13, 0x00);
    bus.write(NR14, 0x85);

    assert_eq!(
        bus.ch1_sweep_timer(),
        7,
        "sweep timer é resetado com pace=7 no trigger"
    );
}

#[test]
fn trigger_com_pace_e_step_zero_desabilita_sweep() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR10, 0x00);
    bus.write(NR13, 0x00);
    bus.write(NR14, 0x85);

    assert!(
        !bus.ch1_sweep_enabled(),
        "sweep é desabilitado no trigger se pace=0 e step=0"
    );
}

#[test]
fn trigger_com_step_nao_zero_abilita_sweep() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR10, 0x01);
    bus.write(NR13, 0x00);
    bus.write(NR14, 0x85);

    assert!(
        bus.ch1_sweep_enabled(),
        "sweep é habilitado no trigger se step != 0 (pace=0, step=1)"
    );
}

#[test]
fn trigger_com_step_nao_zero_faz_calculo_imediato_de_sweep() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR10, 0x79);
    bus.write(NR13, 0x00);
    bus.write(NR14, 0x85);

    // shadow = $0500 = 1280. step = 1. direction = 1 (subtract).
    // shift = 1280 >> 1 = 640. new = 1280 - 640 = 640.
    let new_period = bus.ch1_sweep_shadow();
    assert_eq!(
        new_period, 640,
        "cálculo imediato do sweep: shadow atualizado para 640 (era 1280, step=1, subtract)"
    );
}

#[test]
fn sweep_addition_aumenta_o_periodo() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR10, 0x11);
    bus.write(NR13, 0x00);
    bus.write(NR14, 0x84);

    // shadow = $0400 = 1024. step = 1. direction = 0 (addition).
    // shift = 1024 >> 1 = 512. new = 1024 + 512 = 1536.
    let shadow = bus.ch1_sweep_shadow();
    assert!(
        shadow == 1536,
        "sweep addition: shadow foi de 1024 para 1536 (step=1, addition)"
    );
}

#[test]
fn sweep_overflow_acima_de_2047_desliga_canal() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR10, 0x00);
    bus.write(NR13, 0xFF);
    bus.write(NR14, 0xC7);

    assert!(
        bus.ch1_enabled(),
        "canal 1 é ligado pelo trigger com período $07FF"
    );
    assert_eq!(bus.ch1_period(), 0x07FFu16);

    bus.write(NR10, 0x11);
    bus.write(NR13, 0xFF);
    bus.write(NR14, 0xC7);

    // shadow = $07FF = 2047. step = 1. direction = 0 (addition).
    // shift = 2047 >> 1 = 1023. new = 2047 + 1023 = 3070 > 2047 → overflow → desliga.
    assert!(
        !bus.ch1_enabled(),
        "overflow do sweep (3070 > 2047) desliga o canal 1"
    );
}

#[test]
fn sweep_overflow_desliga_mesmo_com_pace_zero() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR10, 0x00);
    bus.write(NR13, 0xFF);
    bus.write(NR14, 0xC7);

    // shadow = $07FF = 2047. step = 0? No, 0x00 has step=0, pace=0.
    // Wait, NR10=0x00 means pace=0, step=0.
    // Actually NR10=$10 means pace=1 and step=0.
    // But we need step != 0 so overflow check happens.
    // Let me reconsider the test...

    // spec: "In addition mode, if the period value would overflow...
    // This occurs even if sweep iterations are disabled by the pace being 0."
    // So even with pace=0, if step != 0, the immediate calc checks overflow.

    bus.write(NR10, 0x04);
    bus.write(NR13, 0xFF);
    bus.write(NR14, 0xC7);

    // NR10=$04: pace=0, direction=0, step=4
    // shadow = $07FF = 2047. step=4, direction=addition.
    // shift = 2047 >> 4 = 127. new = 2047 + 127 = 2174 > 2047 → overflow.
    assert!(
        !bus.ch1_enabled(),
        "overflow do sweep desliga canal mesmo com pace=0 (step=4, 2047+127 > 2047)"
    );
}

#[test]
fn sweep_timer_do_ch1_decrementa_nos_passos_2_e_6() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(NR10, 0x72);
    bus.write(NR13, 0x00);
    bus.write(NR14, 0x85);

    assert_eq!(bus.ch1_sweep_timer(), 7);

    step_n(&mut cpu, &mut bus, 2 * 2048);
    assert_eq!(bus.apu_frame_sequencer_step(), 2);
    assert_eq!(
        bus.ch1_sweep_timer(),
        6,
        "sweep timer decrementa no passo 2"
    );

    step_n(&mut cpu, &mut bus, 4 * 2048);
    assert_eq!(bus.apu_frame_sequencer_step(), 6);
    assert_eq!(
        bus.ch1_sweep_timer(),
        5,
        "sweep timer decrementa no passo 6"
    );
}

#[test]
fn sweep_iteracao_escreve_novo_periodo_de_volta_em_nr13_nr14() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(NR10, 0x19);
    bus.write(NR13, 0x00);
    bus.write(NR14, 0x85);

    // Trigger com pace=1, direction=1 (subtract), step=1.
    // Immediate: shadow = 1280 - 640 = 640. Timer = 1.
    assert_eq!(
        bus.ch1_sweep_shadow(),
        640,
        "cálculo imediato: shadow = 640"
    );
    assert_eq!(
        bus.ch1_period(),
        640,
        "período visível é 640 após cálculo imediato"
    );

    // Avançar ao passo 2 (4096 M-cycles): timer 1→0, reload 1, iteration.
    // shadow = 640 - 320 = 320. Write back. Second calc: 320 - 160 = 160.
    step_n(&mut cpu, &mut bus, 2 * 2048);
    assert_eq!(bus.apu_frame_sequencer_step(), 2);

    assert_eq!(
        bus.ch1_sweep_shadow(),
        320,
        "shadow do sweep foi para 320 após uma iteração (640 - 320 = 320)"
    );
    assert_eq!(
        bus.ch1_period(),
        320,
        "período visível atualizado para 320 após iteração do sweep"
    );
}

#[test]
fn sweep_com_pace_zero_nao_itera_mas_overflow_ainda_verifica() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(NR10, 0x79);
    bus.write(NR13, 0x00);
    bus.write(NR14, 0x85);

    let shadow_antes = bus.ch1_sweep_shadow();

    bus.write(NR10, 0x01);

    step_n(&mut cpu, &mut bus, 20 * 2048);

    assert_eq!(
        bus.ch1_sweep_shadow(),
        shadow_antes,
        "com pace=0 o sweep não itera: shadow permanece inalterado"
    );
}

#[test]
fn escrita_em_nr13_nr14_atualiza_periodo_mas_nao_afeta_shadow() {
    let (_cpu, mut bus) = machine(&[]);

    bus.write(NR10, 0x00);
    bus.write(NR13, 0x00);
    bus.write(NR14, 0x85);
    // ^ trigger com período $0500, sweep desabilitado

    bus.write(NR13, 0xFF);
    bus.write(NR14, 0x07);

    assert_eq!(
        bus.ch1_period(),
        0x07FFu16,
        "período visível é $07FF após escrever NR13/NR14"
    );
    assert_eq!(
        bus.ch1_sweep_shadow(),
        0x0500,
        "shadow register mantém $0500 — não é atualizado por escrita direta em NR13/NR14"
    );
}

#[test]
fn freq_timer_do_ch1_avanca_4_por_m_cycle_e_sofre_overflow() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR13, 0x00);
    bus.write(NR14, 0x85);

    let (mut cpu, mut bus) = (
        Cpu::after_boot_rom(CartridgeHeader::parse(&[0x00; 32768]).expect("").checksum()),
        bus,
    );

    bus.write(NR13, 0x00);
    bus.write(NR14, 0x85);

    step_n(&mut cpu, &mut bus, 191);
    assert_eq!(
        bus.ch1_frequency_timer(),
        2044,
        "depois de 191 M-cycles o freq_timer do CH1 é 2044"
    );

    step_n(&mut cpu, &mut bus, 1);
    assert_eq!(
        bus.ch1_frequency_timer(),
        0x0500,
        "após 192 M-cycles o freq_timer do CH1 sofre overflow e recarrega $0500"
    );
}

#[test]
fn envelope_do_ch1_carrega_volume_do_trigger_e_diminui_no_passo_2() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(NR12, 0xF1);
    bus.write(NR13, 0x00);
    bus.write(NR14, 0x87);

    assert_eq!(bus.ch1_envelope_volume(), 15);

    step_n(&mut cpu, &mut bus, 4096);
    assert_eq!(bus.apu_frame_sequencer_step(), 2);
    assert_eq!(bus.ch1_envelope_volume(), 14);
}

#[test]
fn duty_step_do_ch1_avanca_no_overflow_do_freq_timer() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(NR13, 0x00);
    bus.write(NR14, 0x85);

    assert_eq!(bus.ch1_duty_step(), 0);
    step_n(&mut cpu, &mut bus, 192);
    assert_eq!(bus.ch1_duty_step(), 1);
}

#[test]
fn ch1_dac_desligado_quando_nr12_bits_7_a_3_zerados() {
    let (_cpu, mut bus) = machine(&[]);

    bus.write(NR12, 0x07);

    assert!(
        !bus.ch1_dac_enabled(),
        "NR12 & $F8 == 0 → DAC do CH1 está desligado"
    );
}
