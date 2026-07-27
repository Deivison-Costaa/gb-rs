//! ROADMAP 6.1 — APU frame sequencer 512 Hz.
//! spec: docs/reference/07-apu.md § DIV-APU, § Audio details.

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::Cpu;

const NR10: u16 = 0xFF10;
const NR11: u16 = 0xFF11;
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

#[test]
fn apu_registers_tem_os_valores_do_hand_off_da_boot_rom() {
    let (_cpu, bus) = machine(&[]);

    assert_eq!(
        bus.read(NR10),
        0x80,
        "NR10 ($FF10) = $80 no hand-off da boot ROM"
    );
    assert_eq!(
        bus.read(NR11),
        0xBF,
        "NR11 ($FF11) = $BF no hand-off da boot ROM"
    );
    assert_eq!(
        bus.read(NR50),
        0x77,
        "NR50 ($FF24) = $77 no hand-off da boot ROM"
    );
    assert_eq!(
        bus.read(NR51),
        0xF3,
        "NR51 ($FF25) = $F3 no hand-off da boot ROM"
    );
    assert_eq!(
        bus.read(NR52),
        0xF0,
        "NR52 ($FF26) bits 3-0 refletem estado real dos canais — sem trigger, todos 0"
    );
}

#[test]
fn frame_sequencer_step_comeca_em_zero() {
    let (_cpu, bus) = machine(&[]);

    assert_eq!(
        bus.apu_frame_sequencer_step(),
        0,
        "o passo do frame sequencer começa em 0 no boot"
    );
}

#[test]
fn frame_sequencer_step_avanca_a_cada_2048_m_cycles() {
    let (mut cpu, mut bus) = machine(&[]);

    assert_eq!(bus.apu_frame_sequencer_step(), 0);

    // 2047 M-cycles: ainda não deve ter avançado.
    step_n(&mut cpu, &mut bus, 2047);
    assert_eq!(
        bus.apu_frame_sequencer_step(),
        0,
        "depois de 2047 M-cycles o passo do frame sequencer ainda é 0"
    );

    // +1 M-cycle: atinge 2048, avança para o passo 1.
    step_n(&mut cpu, &mut bus, 1);
    assert_eq!(
        bus.apu_frame_sequencer_step(),
        1,
        "depois de 2048 M-cycles o passo do frame sequencer avança para 1"
    );
}

#[test]
fn frame_sequencer_completa_o_ciclo_de_8_passos() {
    let (mut cpu, mut bus) = machine(&[]);

    // Cada passo = 2048 M-cycles. 7 passos = 7 * 2048 = 14336
    step_n(&mut cpu, &mut bus, 14336);
    assert_eq!(
        bus.apu_frame_sequencer_step(),
        7,
        "depois de 7*2048 M-cycles o frame sequencer está no passo 7"
    );

    // Mais 2048 M-cycles: volta para o passo 0 (wrap)
    step_n(&mut cpu, &mut bus, 2048);
    assert_eq!(
        bus.apu_frame_sequencer_step(),
        0,
        "o frame sequencer dá a volta: 8*2048 M-cycles = passo 0 de novo"
    );
}

#[test]
fn escrever_nr52_bit_7_liga_e_desliga_a_apu() {
    let (_cpu, bus) = machine(&[]);

    // NR52 bit 7 é 1 no boot (power on)
    assert_eq!(bus.read(NR52) & 0x80, 0x80, "APU está ligada no boot");

    // Desliga a APU escrevendo bit 7 = 0
    let mut bus = bus;
    bus.write(NR52, 0x00);
    assert_eq!(
        bus.read(NR52) & 0x80,
        0x00,
        "APU desligada: bit 7 de NR52 é 0"
    );

    // Liga de novo
    bus.write(NR52, 0x80);
    assert_eq!(bus.read(NR52) & 0x80, 0x80, "APU ligada: bit 7 de NR52 é 1");
}

#[test]
fn escrever_nr10_armazena_o_valor_corretamente() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    bus.write(NR10, 0x42);
    assert_eq!(
        bus.read(NR10),
        0x42,
        "escrever $42 em NR10 ($FF10) deve ser lido de volta"
    );
}

#[test]
fn escrever_nr52_nos_bits_0_a_3_preserva_os_bits_de_status() {
    let (_cpu, bus) = machine(&[]);
    let mut bus = bus;

    let initial = bus.read(NR52);
    // Tenta escrever 1 nos bits de status dos canais (bits 0-3)
    bus.write(NR52, 0x8F);
    // Os bits 0-3 de NR52 são read-only: devem manter o valor original
    assert_eq!(
        bus.read(NR52) & 0x0F,
        initial & 0x0F,
        "bits 0-3 de NR52 são read-only: escrever neles não altera o estado"
    );
    // Bit 7 (audio on/off) deve ter sido escrito
    assert_eq!(
        bus.read(NR52) & 0x80,
        0x80,
        "bit 7 de NR52 (audio on/off) é R/W e persiste o valor escrito"
    );
}
