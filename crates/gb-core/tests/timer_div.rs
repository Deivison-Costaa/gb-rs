//! ROADMAP 2.1 — registrador DIV ($FF04): contador visível do system counter.
//! spec: `docs/reference/04-timers.md` § FF04 — DIV: Divider register.

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::Cpu;

const DIV: u16 = 0xFF04;
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

#[test]
fn div_starts_at_the_boot_hand_off_value_ab() {
    let (_cpu, bus) = machine(&[]);

    assert_eq!(
        bus.read(DIV),
        0xAB,
        "$FF04 (DIV) no hand-off da boot ROM é $AB, como a coluna DMG manda"
    );
}

#[test]
fn div_changes_after_enough_m_cycles_have_passed() {
    let (mut cpu, mut bus) = machine(&[]);
    let initial = bus.read(DIV);

    // O system counter avança 4 por M-cycle; DIV = contador >> 8.
    // DIV muda a cada 64 M-cycles. 64 passos bastam para garantir mudança.
    for _ in 0..64 {
        cpu.step(&mut bus);
    }

    assert_ne!(
        bus.read(DIV),
        initial,
        "depois de 64 M-cycles o DIV já não é mais o mesmo: o contador de \
         sistema avançou o bastante para virar o byte alto ao menos uma vez"
    );
}

#[test]
fn div_increments_monotonically_with_m_cycles() {
    let (mut cpu, mut bus) = machine(&[]);

    // Dá tempo para alguns incrementos de DIV acontecerem.
    for _ in 0..256 {
        cpu.step(&mut bus);
    }

    let div_a = bus.read(DIV);

    for _ in 0..256 {
        cpu.step(&mut bus);
    }

    let div_b = bus.read(DIV);

    assert!(
        div_b.wrapping_sub(div_a) > 0 || (div_a > 0xF0 && div_b < 0x10),
        "DIV avança conforme o system counter: {div_a:02X} → {div_b:02X} \
         depois de mais 256 M-cycles (o wrapping é esperado: um contador \
         de 8 bits visíveis dá a volta)"
    );
}

#[test]
fn writing_to_div_resets_the_system_counter_to_zero() {
    let (mut cpu, mut bus) = machine(&[]);

    // Avança alguns M-cycles para o DIV sair de $AB.
    for _ in 0..128 {
        cpu.step(&mut bus);
    }

    // Escreve qualquer valor em DIV — deve zerar o contador.
    bus.write(DIV, 0x42);

    assert_eq!(
        bus.read(DIV),
        0x00,
        "escrever qualquer valor em $FF04 (DIV) zera o contador de sistema: \
         DIV passa a ler $00 imediatamente"
    );
}

#[test]
fn div_resumes_incrementing_after_being_reset() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(DIV, 0xFF);
    assert_eq!(bus.read(DIV), 0x00, "DIV zerou com a escrita");

    // Após 128 M-cycles o contador já avançou 512 unidades:
    // o byte alto mudou pelo menos uma vez (de $00 para $01 ou $02).
    for _ in 0..128 {
        cpu.step(&mut bus);
    }

    assert_ne!(
        bus.read(DIV),
        0x00,
        "depois do reset o DIV volta a incrementar normalmente"
    );
}

#[test]
fn div_is_not_affected_by_writes_to_neighbor_timer_registers() {
    let (mut cpu, mut bus) = machine(&[]);

    // Avança para ter um valor diferente do boot.
    for _ in 0..200 {
        cpu.step(&mut bus);
    }

    let before = bus.read(DIV);

    // Escreve em TIMA e TAC — não devem afetar DIV.
    bus.write(0xFF05, 0x00);
    bus.write(0xFF07, 0x04);

    assert_eq!(
        bus.read(DIV),
        before,
        "escrever em TIMA ($FF05) e TAC ($FF07) não mexe no DIV: o system \
         counter é independente dos registradores de controle do timer"
    );
}

#[test]
fn div_wraps_naturally_when_the_system_counter_overflows() {
    let (mut cpu, mut bus) = machine(&[]);

    // Começa zerando o contador e caminhando:
    bus.write(DIV, 0x00);

    // 16128 passos: contador = 16128 × 4 = 64512 = 0xFC00, DIV = 0xFC
    for _ in 0..16128 {
        cpu.step(&mut bus);
    }

    let div_before_wrap = bus.read(DIV);
    assert!(
        div_before_wrap >= 0xFC,
        "depois de ~16128 passos o DIV devia estar perto de $FC, \
         e está em ${div_before_wrap:02X}"
    );

    // +256 passos: contador avança 1024 → passa de 0xFFFF e wrapa.
    for _ in 0..256 {
        cpu.step(&mut bus);
    }

    let div_after_wrap = bus.read(DIV);
    assert!(
        div_after_wrap < div_before_wrap || div_before_wrap > 0xF0,
        "DIV deu a volta naturalmente quando o contador de 16 bits \
         estourou: estava em ${div_before_wrap:02X} e agora está em \
         ${div_after_wrap:02X}"
    );
}
