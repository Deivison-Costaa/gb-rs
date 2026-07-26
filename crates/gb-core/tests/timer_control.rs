//! ROADMAP 2.1 — TIMA, TMA, TAC: registradores de controle do timer.
//! spec: `docs/reference/04-timers.md` § FF05–FF07 e § Timer obscure behaviour.

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::Cpu;

const DIV: u16 = 0xFF04;
const TIMA: u16 = 0xFF05;
const TMA: u16 = 0xFF06;
const TAC: u16 = 0xFF07;
const IF: u16 = 0xFF0F;
const ENTRY: usize = 0x0100;

fn rom_with(program: &[u8]) -> Vec<u8> {
    let mut rom = vec![0x00; NoMbc::MAX_ROM_LEN];
    rom[ENTRY..ENTRY + program.len()].copy_from_slice(program);
    rom
}

fn machine(program: &[u8]) -> (Cpu, Bus) {
    let rom = rom_with(program);
    let checksum = CartridgeHeader::parse(&rom)
        .expect("32 KiB de ROM têm cabeçalho de sobra")
        .checksum();
    let cartridge = NoMbc::new(rom).expect("32 KiB cabem num cartucho sem MBC");

    (Cpu::after_boot_rom(checksum), Bus::new(Box::new(cartridge)))
}

fn step_many(cpu: &mut Cpu, bus: &mut Bus, n: usize) {
    for _ in 0..n {
        cpu.step(bus);
    }
}

#[test]
fn tima_tma_tac_start_at_boot_values() {
    let (_cpu, bus) = machine(&[]);

    assert_eq!(bus.read(TIMA), 0x00, "TIMA começa em $00 (valor de boot)");
    assert_eq!(bus.read(TMA), 0x00, "TMA começa em $00 (valor de boot)");
    assert_eq!(
        bus.read(TAC),
        0xF8,
        "TAC começa em $F8 (timer desabilitado, clock 00, bits altos em 1)"
    );
}

#[test]
fn tima_does_not_increment_when_timer_disabled() {
    let (mut cpu, mut bus) = machine(&[]);
    bus.write(DIV, 0x00);

    // TAC já vem $F8 (desabilitado) do boot — não mexe.
    step_many(&mut cpu, &mut bus, 512);

    assert_eq!(
        bus.read(TIMA),
        0x00,
        "TIMA não avança nenhuma unidade com o timer desabilitado"
    );
}

#[test]
fn tima_increments_at_4096hz_with_clock_00() {
    let (mut cpu, mut bus) = machine(&[]);
    bus.write(DIV, 0x00);
    bus.write(TAC, 0x04); // enable=1, clock=00

    // Primeiro falling edge do bit 9: sys_counter=1024 → 256 M-cycles.
    step_many(&mut cpu, &mut bus, 255);
    assert_eq!(
        bus.read(TIMA),
        0x00,
        "TIMA=$00 nos primeiros 255 M-cycles (clock 00)"
    );

    cpu.step(&mut bus); // M-cycle 256
    assert_eq!(
        bus.read(TIMA),
        0x01,
        "TIMA=$01 após 256 M-cycles (primeiro falling edge do bit 9)"
    );

    step_many(&mut cpu, &mut bus, 256);
    assert_eq!(
        bus.read(TIMA),
        0x02,
        "TIMA=$02 após 512 M-cycles (segundo falling edge)"
    );
}

#[test]
fn tima_increments_at_262144hz_with_clock_01() {
    let (mut cpu, mut bus) = machine(&[]);
    bus.write(DIV, 0x00);
    bus.write(TAC, 0x05); // enable=1, clock=01

    step_many(&mut cpu, &mut bus, 3);
    assert_eq!(
        bus.read(TIMA),
        0x00,
        "TIMA=$00 nos primeiros 3 M-cycles (clock 01)"
    );

    cpu.step(&mut bus); // M-cycle 4
    assert_eq!(
        bus.read(TIMA),
        0x01,
        "TIMA=$01 após 4 M-cycles (primeiro falling edge do bit 3)"
    );

    step_many(&mut cpu, &mut bus, 4);
    assert_eq!(
        bus.read(TIMA),
        0x02,
        "TIMA=$02 após 8 M-cycles (segundo falling edge)"
    );
}

#[test]
fn tima_increments_at_65536hz_with_clock_10() {
    let (mut cpu, mut bus) = machine(&[]);
    bus.write(DIV, 0x00);
    bus.write(TAC, 0x06); // enable=1, clock=10

    step_many(&mut cpu, &mut bus, 15);
    assert_eq!(
        bus.read(TIMA),
        0x00,
        "TIMA=$00 nos primeiros 15 M-cycles (clock 10)"
    );

    cpu.step(&mut bus); // M-cycle 16
    assert_eq!(
        bus.read(TIMA),
        0x01,
        "TIMA=$01 após 16 M-cycles (primeiro falling edge do bit 5)"
    );

    step_many(&mut cpu, &mut bus, 16);
    assert_eq!(
        bus.read(TIMA),
        0x02,
        "TIMA=$02 após 32 M-cycles (segundo falling edge)"
    );
}

#[test]
fn tima_increments_at_16384hz_with_clock_11() {
    let (mut cpu, mut bus) = machine(&[]);
    bus.write(DIV, 0x00);
    bus.write(TAC, 0x07); // enable=1, clock=11

    step_many(&mut cpu, &mut bus, 63);
    assert_eq!(
        bus.read(TIMA),
        0x00,
        "TIMA=$00 nos primeiros 63 M-cycles (clock 11)"
    );

    cpu.step(&mut bus); // M-cycle 64
    assert_eq!(
        bus.read(TIMA),
        0x01,
        "TIMA=$01 após 64 M-cycles (primeiro falling edge do bit 7)"
    );

    step_many(&mut cpu, &mut bus, 64);
    assert_eq!(
        bus.read(TIMA),
        0x02,
        "TIMA=$02 após 128 M-cycles (segundo falling edge)"
    );
}

#[test]
fn tima_overflow_reloads_from_tma_after_one_m_cycle_delay() {
    let (mut cpu, mut bus) = machine(&[]);
    bus.write(DIV, 0x00);
    bus.write(TMA, 0x42);
    bus.write(TAC, 0x05); // clock 01 (4 M-cycles por tick)

    // Preenche TIMA até $FF: 255 ticks × 4 = 1020 M-cycles
    step_many(&mut cpu, &mut bus, 1020);
    assert_eq!(
        bus.read(TIMA),
        0xFF,
        "TIMA=$FF após 255 ticks (1020 M-cycles)"
    );

    // +1 tick (4 M-cycles): TIMA overflowa para $00
    step_many(&mut cpu, &mut bus, 4);
    assert_eq!(
        bus.read(TIMA),
        0x00,
        "TIMA=$00 imediatamente após o overflow (ciclo A da spec)"
    );

    // +1 M-cycle: o reload acontece no próximo tick_timer → TIMA = TMA.
    cpu.step(&mut bus);
    assert_eq!(
        bus.read(TIMA),
        0x42,
        "TIMA=$42 depois de 1 M-cycle do reload (TMA=$42)"
    );
}

#[test]
fn tima_overflow_sets_if_timer_bit_after_delay() {
    let (mut cpu, mut bus) = machine(&[]);
    bus.write(DIV, 0x00);
    bus.write(TMA, 0x23);
    bus.write(TAC, 0x05); // clock 01

    // Enche TIMA até $FF.
    step_many(&mut cpu, &mut bus, 1020);
    assert_eq!(bus.read(TIMA), 0xFF);

    // Ciclo A: TIMA overflowa para $00, IF ainda sem bit de timer.
    step_many(&mut cpu, &mut bus, 4);
    assert_eq!(bus.read(TIMA), 0x00);
    assert_eq!(
        bus.read(IF) & 0x04,
        0x00,
        "IF bit 2 (timer) ainda em 0 no ciclo A do overflow"
    );

    // Ciclo B: reload + IF bit 2 ← 1.
    cpu.step(&mut bus);
    assert_eq!(bus.read(TIMA), 0x23, "TIMA recarregou de TMA no ciclo B");
    assert_eq!(
        bus.read(IF) & 0x04,
        0x04,
        "IF bit 2 (timer) está em 1 após o ciclo B do overflow"
    );
}

#[test]
fn tima_reads_and_writes_the_stored_value() {
    let (mut _cpu, mut bus) = machine(&[]);

    bus.write(TIMA, 0x7A);
    assert_eq!(bus.read(TIMA), 0x7A);

    bus.write(TIMA, 0x00);
    assert_eq!(bus.read(TIMA), 0x00);

    bus.write(TIMA, 0xFF);
    assert_eq!(bus.read(TIMA), 0xFF);
}

#[test]
fn tma_reads_and_writes_the_stored_value() {
    let (mut _cpu, mut bus) = machine(&[]);

    bus.write(TMA, 0xAB);
    assert_eq!(bus.read(TMA), 0xAB);

    bus.write(TMA, 0x00);
    assert_eq!(bus.read(TMA), 0x00);
}

#[test]
fn tac_reads_and_writes_with_upper_bits_preserved() {
    let (mut _cpu, mut bus) = machine(&[]);

    bus.write(TAC, 0x05);
    assert_eq!(bus.read(TAC) & 0x07, 0x05);

    bus.write(TAC, 0x00);
    assert_eq!(bus.read(TAC) & 0x07, 0x00);
}

#[test]
fn disabling_tac_stops_tima_from_incrementing() {
    let (mut cpu, mut bus) = machine(&[]);
    bus.write(DIV, 0x00);
    bus.write(TAC, 0x05); // clock 01, habilitado

    // 12 M-cycles: 3 ticks (M-cycles 4, 8, 12)
    step_many(&mut cpu, &mut bus, 12);
    assert_eq!(bus.read(TIMA), 0x03);

    // Desabilita
    bus.write(TAC, 0x01); // mesmo clock, enable=0

    step_many(&mut cpu, &mut bus, 40);
    assert_eq!(
        bus.read(TIMA),
        0x03,
        "TIMA parou de contar depois que o timer foi desabilitado"
    );
}

#[test]
fn changing_tac_clock_select_changes_tima_tick_rate() {
    let (mut cpu, mut bus) = machine(&[]);
    bus.write(DIV, 0x00);
    bus.write(TAC, 0x06); // clock 10, 16 M-cycles por tick

    // 32 M-cycles: 2 ticks (M-cycles 16, 32)
    step_many(&mut cpu, &mut bus, 32);
    assert_eq!(bus.read(TIMA), 0x02);

    // Troca para clock 11 (64 M-cycles)
    bus.write(TAC, 0x07);

    // +64 M-cycles: 1 tick com o novo clock
    step_many(&mut cpu, &mut bus, 64);
    assert_eq!(
        bus.read(TIMA),
        0x03,
        "TIMA=$03: 2 ticks (clock 10) + 1 tick (clock 11)"
    );
}

#[test]
fn writing_to_tima_during_overflow_cycle_cancels_reload() {
    let (mut cpu, mut bus) = machine(&[]);
    bus.write(DIV, 0x00);
    bus.write(TMA, 0x55);
    bus.write(TAC, 0x05); // clock 01

    // Enche TIMA até $FF: 255 ticks × 4 = 1020 M-cycles.
    step_many(&mut cpu, &mut bus, 1020);
    assert_eq!(
        bus.read(TIMA),
        0xFF,
        "pré-condição: TIMA=$FF após 255 ticks"
    );

    // Ciclo A: overflow. TIMA=$00.
    step_many(&mut cpu, &mut bus, 4);
    assert_eq!(
        bus.read(TIMA),
        0x00,
        "pré-condição: TIMA=$00 no ciclo A do overflow"
    );

    // Escreve em TIMA durante o ciclo A → cancela o reload.
    bus.write(TIMA, 0x77);
    assert_eq!(bus.read(TIMA), 0x77);

    // +1 M-cycle: se o reload NÃO foi cancelado, TIMA seria $55.
    // Com o cancelamento, TIMA continua $77.
    cpu.step(&mut bus);
    assert_eq!(bus.read(TIMA), 0x77, "reload cancelado: TIMA=$77, não $55");
    assert_eq!(
        bus.read(IF) & 0x04,
        0x00,
        "IF bit 2 não foi ativado porque o overflow foi cancelado"
    );
}

#[test]
fn tima_is_not_affected_by_div_writes_directly() {
    let (mut cpu, mut bus) = machine(&[]);
    bus.write(DIV, 0x00);
    bus.write(TAC, 0x05);

    // Avança algumas unidades de TIMA
    step_many(&mut cpu, &mut bus, 8);
    let tima_antes = bus.read(TIMA);
    assert!(tima_antes > 0);

    // Escreve em DIV: zera o contador mas TIMA não mexe diretamente.
    bus.write(DIV, 0xFF);

    assert_eq!(
        bus.read(TIMA),
        tima_antes,
        "escrever em DIV não altera TIMA diretamente"
    );
}
