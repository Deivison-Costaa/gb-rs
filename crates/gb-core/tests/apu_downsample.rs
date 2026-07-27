//! ROADMAP 6.7a — Acumulador de downsample + ring buffer no Apu.
//! spec: docs/reference/07-apu.md § Audio Details.

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::Cpu;

const NR12: u16 = 0xFF12;
const NR14: u16 = 0xFF14;
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
fn buffer_starts_empty() {
    let (_, bus) = machine(&[]);
    assert_eq!(bus.sample_buffer_len(), 0);
}

#[test]
fn accumulates_mixer_samples_and_produces_output() {
    let (mut cpu, mut bus) = machine(&[]);
    bus.write(NR52, 0x80);
    bus.write(NR50, 0x77);

    step_n(&mut cpu, &mut bus, 30);

    assert!(
        bus.sample_buffer_len() > 0,
        "deve ter ao menos uma amostra após ~22 M-cycles"
    );
}

#[test]
fn draining_samples_reduces_buffer_count() {
    let (mut cpu, mut bus) = machine(&[]);
    bus.write(NR52, 0x80);
    bus.write(NR50, 0x77);

    step_n(&mut cpu, &mut bus, 100);

    let available = bus.sample_buffer_len();
    assert!(available > 1, "precisa de ao menos 2 amostras para drenar");

    let mut dst = vec![(0.0f32, 0.0f32); 1];
    let drained = bus.drain_sample_buffer(&mut dst);
    assert_eq!(drained, 1);
    assert_eq!(bus.sample_buffer_len(), available - 1);
}

#[test]
fn drain_returns_at_most_requested_count() {
    let (mut cpu, mut bus) = machine(&[]);
    bus.write(NR52, 0x80);
    bus.write(NR50, 0x77);

    step_n(&mut cpu, &mut bus, 100);

    let available = bus.sample_buffer_len();
    let request = (available / 2).max(1);

    let mut dst = vec![(0.0f32, 0.0f32); request];
    let drained = bus.drain_sample_buffer(&mut dst);
    assert!(drained <= request);
    assert!(drained > 0);
}

#[test]
fn powered_off_apu_produces_only_zero_samples() {
    let (mut cpu, mut bus) = machine(&[]);
    bus.write(NR52, 0x00);

    step_n(&mut cpu, &mut bus, 30);

    let available = bus.sample_buffer_len();
    assert!(
        available > 0,
        "o timer de downsample avança mesmo com APU desligada"
    );

    let mut dst = vec![(0.0f32, 0.0f32); available];
    let drained = bus.drain_sample_buffer(&mut dst);
    assert_eq!(drained, available);

    for &(left, right) in &dst[..drained] {
        assert_eq!(
            left, 0.0,
            "amostra esquerda deve ser zero com APU desligada"
        );
        assert_eq!(
            right, 0.0,
            "amostra direita deve ser zero com APU desligada"
        );
    }
}

#[test]
fn sample_values_reflect_mixer_output() {
    let (mut cpu, mut bus) = machine(&[]);
    bus.write(NR52, 0x80);
    bus.write(NR50, 0x00);
    bus.write(NR51, 0x11);
    bus.write(NR12, 0xF0);
    bus.write(NR14, 0x87);

    step_n(&mut cpu, &mut bus, 44);

    let available = bus.sample_buffer_len();
    assert!(available >= 2);

    let mut dst = vec![(0.0f32, 0.0f32); available];
    let drained = bus.drain_sample_buffer(&mut dst);

    let has_nonzero = dst[..drained].iter().any(|&(l, r)| l > 0.0 || r > 0.0);
    assert!(
        has_nonzero,
        "ao menos uma amostra deve refletir saída do mixer com CH1 ligado"
    );

    let has_matching_channels = dst[..drained]
        .iter()
        .all(|&(l, r)| (l - r).abs() < f32::EPSILON * 10.0);
    assert!(
        has_matching_channels,
        "canais esquerdo e direito devem ser iguais com panning central (NR51=0x11)"
    );
}
