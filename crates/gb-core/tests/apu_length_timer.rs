//! ROADMAP 6.8a — APU length counter: load, tick, disable, reload, NR52 readback.
//! spec: docs/reference/07-apu.md § Common concepts > Length timer, § NRx1, § NRx4, § NR52.

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::Cpu;

const NR21: u16 = 0xFF16;
const NR22: u16 = 0xFF17;
const NR23: u16 = 0xFF18;
const NR24: u16 = 0xFF19;

const NR11: u16 = 0xFF11;
const NR12: u16 = 0xFF12;
const NR13: u16 = 0xFF13;
const NR14: u16 = 0xFF14;
const NR10: u16 = 0xFF10;

const NR30: u16 = 0xFF1A;
const NR31: u16 = 0xFF1B;
const NR32: u16 = 0xFF1C;
const NR33: u16 = 0xFF1D;
const NR34: u16 = 0xFF1E;

const NR41: u16 = 0xFF20;
const NR42: u16 = 0xFF21;
const NR43: u16 = 0xFF22;
const NR44: u16 = 0xFF23;

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

const FRAME_SEQUENCER_MCYCLES: u32 = 2048;

#[test]
fn nr11_carrega_length_timer_interno_com_64_menos_l() {
    let (_cpu, mut bus) = machine(&[]);

    bus.write(NR11, 0x00);
    assert_eq!(
        bus.ch1_length_timer_internal(),
        64,
        "NR11=0x00 → length interno = 64 - 0 = 64"
    );

    bus.write(NR11, 0x3F);
    assert_eq!(
        bus.ch1_length_timer_internal(),
        1,
        "NR11=0x3F (L=63) → length interno = 64 - 63 = 1"
    );

    bus.write(NR11, 0x20);
    assert_eq!(
        bus.ch1_length_timer_internal(),
        32,
        "NR11=0x20 (L=32) → length interno = 64 - 32 = 32"
    );
}

#[test]
fn nr21_carrega_length_timer_interno_com_64_menos_l() {
    let (_cpu, mut bus) = machine(&[]);

    bus.write(NR21, 0x00);
    assert_eq!(
        bus.ch2_length_timer_internal(),
        64,
        "NR21=0x00 → length interno = 64 - 0 = 64"
    );

    bus.write(NR21, 0x3F);
    assert_eq!(
        bus.ch2_length_timer_internal(),
        1,
        "NR21=0x3F (L=63) → length interno = 64 - 63 = 1"
    );
}

#[test]
fn nr31_carrega_length_timer_interno_com_256_menos_l() {
    let (_cpu, mut bus) = machine(&[]);

    bus.write(NR31, 0x00);
    assert_eq!(
        bus.ch3_length_timer_internal(),
        256,
        "NR31=0x00 → length interno = 256 - 0 = 256"
    );

    bus.write(NR31, 0xFF);
    assert_eq!(
        bus.ch3_length_timer_internal(),
        1,
        "NR31=0xFF → length interno = 256 - 255 = 1"
    );
}

#[test]
fn nr41_carrega_length_timer_interno_com_64_menos_l() {
    let (_cpu, mut bus) = machine(&[]);

    bus.write(NR41, 0x00);
    assert_eq!(
        bus.ch4_length_timer_internal(),
        64,
        "NR41=0x00 → length interno = 64 - 0 = 64"
    );

    bus.write(NR41, 0x3F);
    assert_eq!(
        bus.ch4_length_timer_internal(),
        1,
        "NR41=0x3F (L=63) → length interno = 64 - 63 = 1"
    );
}

#[test]
fn length_timer_do_ch2_desliga_canal_ao_expirar_com_length_enable() {
    let (mut cpu, mut bus) = machine(&[]);

    // Posiciona step=1 (next=2 IS length clock) para evitar obscure behavior
    step_n(&mut cpu, &mut bus, FRAME_SEQUENCER_MCYCLES);

    // NR21=0x3F (L=63): contador carrega com 64-63=1 → expira em 1 tick de length
    bus.write(NR21, 0x3F);
    bus.write(NR22, 0xF1);
    bus.write(NR23, 0x00);
    bus.write(NR24, 0xC7); // trigger + length enable

    assert!(bus.ch2_enabled(), "canal 2 deve estar ligado após trigger");

    // step 1→2 (length tick: 1→0, desliga) → 3.   2 * 2048 M-cycles.
    step_n(&mut cpu, &mut bus, 2 * FRAME_SEQUENCER_MCYCLES);

    assert!(
        !bus.ch2_enabled(),
        "canal 2 deve ser desligado após o length timer expirar"
    );
}

#[test]
fn length_timer_do_ch2_nao_desliga_canal_sem_length_enable() {
    let (mut cpu, mut bus) = machine(&[]);

    // NR21=0x3F (L=63): contador=1
    bus.write(NR21, 0x3F);
    bus.write(NR22, 0xF1);
    bus.write(NR23, 0x00);
    bus.write(NR24, 0x87); // trigger SEM bit 6 (length enable)

    assert!(bus.ch2_enabled(), "canal 2 deve estar ligado após trigger");

    // Avança o suficiente para vários ticks de length passarem
    step_n(&mut cpu, &mut bus, 10 * 2 * FRAME_SEQUENCER_MCYCLES);

    assert!(
        bus.ch2_enabled(),
        "canal 2 deve continuar ligado — length enable desabilitado"
    );
}

#[test]
fn length_timer_e_recarregado_ao_disparar_quando_expirou() {
    let (mut cpu, mut bus) = machine(&[]);

    // Posiciona step=1 (next=2 IS length clock) para evitar obscure behavior
    step_n(&mut cpu, &mut bus, FRAME_SEQUENCER_MCYCLES);

    // NR21=0x3F (L=63): contador=1, expira rapidamente
    bus.write(NR21, 0x3F);
    bus.write(NR22, 0xF1);
    bus.write(NR23, 0x00);
    bus.write(NR24, 0xC7); // trigger + length enable

    // Deixa expirar
    step_n(&mut cpu, &mut bus, 2 * FRAME_SEQUENCER_MCYCLES);
    assert!(
        !bus.ch2_enabled(),
        "canal deve estar desligado após expirar"
    );

    // step=3, next=4 IS length clock. Re-trigger carrega 64 normalmente.
    bus.write(NR24, 0xC7); // trigger + length enable
    assert!(
        bus.ch2_length_timer_internal() == 64,
        "length timer deve recarregar para 64 no trigger quando expirou"
    );
    assert!(bus.ch2_enabled(), "canal deve estar ligado após re-trigger");
}

#[test]
fn nr52_reflete_canal_desligado_por_length_expirar() {
    let (mut cpu, mut bus) = machine(&[]);

    // Posiciona step=1 (next=2 IS length clock) para evitar obscure behavior
    step_n(&mut cpu, &mut bus, FRAME_SEQUENCER_MCYCLES);

    bus.write(NR21, 0x3F);
    bus.write(NR22, 0xF1);
    bus.write(NR23, 0x00);
    bus.write(NR24, 0xC7);

    let nr52 = bus.read(NR52);
    assert_eq!(
        nr52 & 0x02,
        0x02,
        "NR52 bit 1 deve estar setado com CH2 ligado"
    );

    step_n(&mut cpu, &mut bus, 2 * FRAME_SEQUENCER_MCYCLES);

    let nr52 = bus.read(NR52);
    assert_eq!(
        nr52 & 0x02,
        0x00,
        "NR52 bit 1 deve estar limpo após length do CH2 expirar"
    );
}

#[test]
fn length_timer_do_ch1_desliga_canal_ao_expirar() {
    let (mut cpu, mut bus) = machine(&[]);

    // Posiciona step=1 (next=2 IS length clock) para evitar obscure behavior
    step_n(&mut cpu, &mut bus, FRAME_SEQUENCER_MCYCLES);

    bus.write(NR10, 0x00); // sweep off
    bus.write(NR11, 0x3F); // L=63, contador=1
    bus.write(NR12, 0xF1);
    bus.write(NR13, 0x00);
    bus.write(NR14, 0xC7); // trigger + length enable

    assert!(bus.ch1_enabled());

    step_n(&mut cpu, &mut bus, 2 * FRAME_SEQUENCER_MCYCLES);

    assert!(
        !bus.ch1_enabled(),
        "canal 1 deve ser desligado após length expirar"
    );
}

#[test]
fn length_timer_do_ch3_desliga_canal_ao_expirar() {
    let (mut cpu, mut bus) = machine(&[]);

    // Posiciona step=1 (next=2 IS length clock) para evitar obscure behavior
    step_n(&mut cpu, &mut bus, FRAME_SEQUENCER_MCYCLES);

    bus.write(NR30, 0x80); // DAC on para CH3
    bus.write(NR31, 0xFF); // L=255, contador=1
    bus.write(NR32, 0x80); // output level 100%
    bus.write(NR33, 0x00);
    bus.write(NR34, 0xC7); // trigger + length enable

    assert!(bus.ch3_enabled());

    step_n(&mut cpu, &mut bus, 2 * FRAME_SEQUENCER_MCYCLES);

    assert!(
        !bus.ch3_enabled(),
        "canal 3 deve ser desligado após length expirar"
    );
}

#[test]
fn length_timer_do_ch4_desliga_canal_ao_expirar() {
    let (mut cpu, mut bus) = machine(&[]);

    // Posiciona step=1 (next=2 IS length clock) para evitar obscure behavior
    step_n(&mut cpu, &mut bus, FRAME_SEQUENCER_MCYCLES);

    bus.write(NR41, 0x3F); // L=63, contador=1
    bus.write(NR42, 0xF1);
    bus.write(NR43, 0x00);
    bus.write(NR44, 0xC7); // trigger + length enable

    assert!(bus.ch4_enabled());

    step_n(&mut cpu, &mut bus, 2 * FRAME_SEQUENCER_MCYCLES);

    assert!(
        !bus.ch4_enabled(),
        "canal 4 deve ser desligado após length expirar"
    );
}

#[test]
fn length_timer_do_ch3_nao_desliga_sem_dac_ligado() {
    let (_cpu, mut bus) = machine(&[]);

    bus.write(NR30, 0x00); // DAC off
    bus.write(NR31, 0xFF);
    bus.write(NR32, 0x80);
    bus.write(NR33, 0x00);
    bus.write(NR34, 0xC7); // trigger + length enable

    assert!(
        !bus.ch3_enabled(),
        "canal 3 não deve ligar com DAC off, mesmo com trigger e length"
    );
}

#[test]
fn length_timer_da_tick_em_cada_2_passos_do_frame_sequencer() {
    let (mut cpu, mut bus) = machine(&[]);

    // Posiciona step=1 (next=2 IS length clock) para evitar obscure behavior
    step_n(&mut cpu, &mut bus, FRAME_SEQUENCER_MCYCLES);

    // Contador = 4: precisa de 4 ticks de length. Começa em passo 1.
    bus.write(NR21, 0x3C); // L=60, contador=64-60=4
    bus.write(NR22, 0xF1);
    bus.write(NR23, 0x00);
    bus.write(NR24, 0xC7);

    assert!(bus.ch2_enabled());

    // Passo 1→2: tick de length. Contador 4→3.
    step_n(&mut cpu, &mut bus, FRAME_SEQUENCER_MCYCLES);
    assert_eq!(bus.ch2_length_timer_internal(), 3);

    // Passo 2→3: sem tick. Contador continua 3.
    step_n(&mut cpu, &mut bus, FRAME_SEQUENCER_MCYCLES);
    assert_eq!(bus.ch2_length_timer_internal(), 3);

    // Passo 3→4: tick. 3→2.
    step_n(&mut cpu, &mut bus, FRAME_SEQUENCER_MCYCLES);
    assert_eq!(bus.ch2_length_timer_internal(), 2);

    // Passo 4→5: sem tick.
    step_n(&mut cpu, &mut bus, FRAME_SEQUENCER_MCYCLES);
    assert_eq!(bus.ch2_length_timer_internal(), 2);

    // Passo 5→6: tick. 2→1.
    step_n(&mut cpu, &mut bus, FRAME_SEQUENCER_MCYCLES);
    assert_eq!(bus.ch2_length_timer_internal(), 1);

    // Passo 6→7: sem tick.
    step_n(&mut cpu, &mut bus, FRAME_SEQUENCER_MCYCLES);
    assert_eq!(bus.ch2_length_timer_internal(), 1);

    // Passo 7→0: tick. 1→0, canal desliga.
    step_n(&mut cpu, &mut bus, FRAME_SEQUENCER_MCYCLES);
    assert_eq!(bus.ch2_length_timer_internal(), 0);
    assert!(
        !bus.ch2_enabled(),
        "canal deve desligar quando length chega a 0 no passo 0"
    );
}
