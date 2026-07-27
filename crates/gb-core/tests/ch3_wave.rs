//! ROADMAP 6.4 — Canal 3: wave RAM.
//! spec: docs/reference/07-apu.md § Sound Channel 3 — Wave output, § Wave channel (CH3).

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::Cpu;

const NR30: u16 = 0xFF1A;
const NR31: u16 = 0xFF1B;
const NR32: u16 = 0xFF1C;
const NR33: u16 = 0xFF1D;
const NR34: u16 = 0xFF1E;

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

#[test]
fn ch3_comeca_desligado() {
    let (_cpu, bus) = machine(&[]);

    assert!(!bus.ch3_enabled(), "canal 3 começa desligado após o boot");
}

#[test]
fn nr30_configura_dac_enable() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR30, 0x80);
    assert!(
        bus.ch3_dac_enabled(),
        "NR30 bit 7 = 1 liga o DAC do canal 3"
    );

    bus.write(NR30, 0x00);
    assert!(
        !bus.ch3_dac_enabled(),
        "NR30 bit 7 = 0 desliga o DAC do canal 3"
    );
}

#[test]
fn nr31_armazena_length_timer() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR31, 0xAB);
    assert_eq!(
        bus.read(NR31),
        0xAB,
        "NR31 armazena os 8 bits do length timer"
    );
}

#[test]
fn nr32_configura_output_level() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR32, 0x00);
    assert_eq!(
        bus.ch3_output_level(),
        0,
        "NR32 bits 6-5 = 00 → mute (output level 0)"
    );

    bus.write(NR32, 0x20);
    assert_eq!(
        bus.ch3_output_level(),
        1,
        "NR32 bits 6-5 = 01 → 100% volume (output level 1)"
    );

    bus.write(NR32, 0x40);
    assert_eq!(
        bus.ch3_output_level(),
        2,
        "NR32 bits 6-5 = 10 → 50% volume (output level 2)"
    );

    bus.write(NR32, 0x60);
    assert_eq!(
        bus.ch3_output_level(),
        3,
        "NR32 bits 6-5 = 11 → 25% volume (output level 3)"
    );
}

#[test]
fn periodo_de_11_bits_combina_nr33_e_nr34() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR33, 0x00);
    bus.write(NR34, 0x05);

    assert_eq!(
        bus.ch3_period(),
        0x0500u16,
        "NR33=$00 + NR34 bits 2-0=5 → período = $0500"
    );
}

#[test]
fn trigger_do_ch3_liga_o_canal_e_carrega_freq_timer() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR30, 0x80);
    bus.write(NR33, 0x00);
    bus.write(NR34, 0x85);

    assert!(
        bus.ch3_enabled(),
        "escrever NR34 com bit 7=1 liga o canal 3"
    );
    assert_eq!(
        bus.ch3_frequency_timer(),
        0x0500u16,
        "freq_timer do CH3 é carregado com o período ($0500) no trigger"
    );
}

#[test]
fn trigger_com_dac_desligado_nao_liga_o_canal() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR30, 0x00);
    bus.write(NR33, 0x00);
    bus.write(NR34, 0x85);

    assert!(
        !bus.ch3_enabled(),
        "trigger não liga o canal 3 se o DAC (NR30 bit 7) está desligado"
    );
}

#[test]
fn trigger_define_sample_index_em_1_nao_em_0() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR30, 0x80);
    bus.write(NR33, 0x00);
    bus.write(NR34, 0x85);

    assert_eq!(
        bus.ch3_sample_index(),
        1,
        "sample_index começa em 1 após o trigger (sample 0 é pulado na primeira volta)"
    );
}

#[test]
fn wave_ram_acesso_normal_quando_ch3_inativo() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(WAVE_RAM_BASE, 0xAB);
    assert_eq!(
        bus.read(WAVE_RAM_BASE),
        0xAB,
        "wave RAM pode ser lida/escrita quando canal 3 está inativo"
    );
}

#[test]
fn wave_ram_retorna_ff_na_leitura_quando_ch3_ativo() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(WAVE_RAM_BASE, 0xAB);

    bus.write(NR30, 0x80);
    bus.write(NR33, 0x00);
    bus.write(NR34, 0x85);

    assert!(bus.ch3_enabled());
    assert_eq!(
        bus.read(WAVE_RAM_BASE),
        0xFF,
        "leitura da wave RAM retorna $FF quando canal 3 está ativo"
    );
}

#[test]
fn wave_ram_escrita_ignorada_quando_ch3_ativo() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(WAVE_RAM_BASE, 0xAB);

    bus.write(NR30, 0x80);
    bus.write(NR33, 0x00);
    bus.write(NR34, 0x85);

    bus.write(WAVE_RAM_BASE, 0x42);

    // Desliga o canal 3 para poder ler a wave RAM sem bloqueio
    bus.write(NR30, 0x00);

    assert_eq!(
        bus.read(WAVE_RAM_BASE),
        0xAB,
        "escrita na wave RAM ($42) foi ignorada com CH3 ativo; valor original ($AB) preservado"
    );
}

#[test]
fn freq_timer_do_ch3_avanca_8_por_m_cycle_e_sofre_overflow() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR30, 0x80);
    bus.write(NR33, 0x00);
    bus.write(NR34, 0x85);

    let (mut cpu, mut bus) = (
        Cpu::after_boot_rom(CartridgeHeader::parse(&[0x00; 32768]).expect("").checksum()),
        bus,
    );

    bus.write(NR30, 0x80);
    bus.write(NR33, 0x00);
    bus.write(NR34, 0x85);

    // freq_timer começa em $0500 = 1280. Overflow em 2048.
    // 2048 - 1280 = 768. Com +8 por M-cycle: 768 / 8 = 96 M-cycles.
    // Aos 95 M-cycles: 1280 + 95*8 = 2040, ainda não overflow.
    step_n(&mut cpu, &mut bus, 95);
    assert_eq!(
        bus.ch3_frequency_timer(),
        2040,
        "depois de 95 M-cycles o freq_timer do CH3 é 2040 (sem overflow ainda)"
    );

    step_n(&mut cpu, &mut bus, 1);
    assert_eq!(
        bus.ch3_frequency_timer(),
        0x0500,
        "após 96 M-cycles o freq_timer do CH3 sofre overflow e recarrega $0500"
    );
}

#[test]
fn sample_index_avanca_no_overflow_do_freq_timer() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(NR30, 0x80);
    bus.write(NR33, 0x00);
    bus.write(NR34, 0x85);

    assert_eq!(bus.ch3_sample_index(), 1);

    // 96 M-cycles = 1 overflow → sample_index = 2
    step_n(&mut cpu, &mut bus, 96);
    assert_eq!(
        bus.ch3_sample_index(),
        2,
        "sample_index avança para 2 após o primeiro overflow do freq_timer"
    );

    step_n(&mut cpu, &mut bus, 96);
    assert_eq!(
        bus.ch3_sample_index(),
        3,
        "sample_index avança para 3 após o segundo overflow"
    );
}

#[test]
fn sample_index_volta_ao_zero_ao_completar_32_samples() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(NR30, 0x80);
    bus.write(NR33, 0x00);
    bus.write(NR34, 0x85);

    assert_eq!(bus.ch3_sample_index(), 1);

    // Avançar 31 samples (1 já é o atual, precisamos de 31 overflows para dar a volta)
    // 31 * 96 = 2976 M-cycles
    step_n(&mut cpu, &mut bus, 96 * 31);
    assert_eq!(
        bus.ch3_sample_index(),
        0,
        "sample_index dá a volta: após 32 samples volta para 0"
    );
}

#[test]
fn ch3_le_sample_da_wave_ram_no_overflow() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(WAVE_RAM_BASE, 0xAB);

    bus.write(NR30, 0x80);
    bus.write(NR33, 0x00);
    bus.write(NR34, 0x85);

    // Após o primeiro overflow: sample_index passa de 1 para 2.
    // sample 1 = nibble baixo de $FF30 = $B
    step_n(&mut cpu, &mut bus, 96);
    assert_eq!(
        bus.ch3_last_sample_buffer(),
        0x0B,
        "sample lido da wave RAM: nibble baixo de $FF30 ($AB → $0B)"
    );
}

#[test]
fn ch3_le_nibble_alto_no_sample_par() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(WAVE_RAM_BASE, 0x6C);
    bus.write(WAVE_RAM_BASE + 1, 0x5A);

    bus.write(NR30, 0x80);
    bus.write(NR33, 0x00);
    bus.write(NR34, 0x85);

    // Primeiro overflow: sample_index 1→2, lê sample 1 (nibble baixo $C)
    step_n(&mut cpu, &mut bus, 96);
    assert_eq!(bus.ch3_sample_index(), 2);
    assert_eq!(bus.ch3_last_sample_buffer(), 0x0C);

    // Segundo overflow: sample_index 2→3, lê sample 2 (nibble alto de $FF31)
    step_n(&mut cpu, &mut bus, 96);
    assert_eq!(bus.ch3_sample_index(), 3);
    assert_eq!(
        bus.ch3_last_sample_buffer(),
        0x05,
        "sample 2 lê nibble alto de $FF31 ($5A → $05)"
    );
}

#[test]
fn ch3_nao_avanca_freq_se_canal_estiver_desligado() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(NR30, 0x80);
    bus.write(NR33, 0x00);
    bus.write(NR34, 0x85);

    assert!(bus.ch3_enabled());

    // Desliga escrevendo NR30 com DAC=0
    bus.write(NR30, 0x00);
    assert!(!bus.ch3_enabled());

    let freq_antes = bus.ch3_frequency_timer();
    step_n(&mut cpu, &mut bus, 100);

    assert_eq!(
        bus.ch3_frequency_timer(),
        freq_antes,
        "freq_timer não avança quando canal 3 está desligado"
    );
}
