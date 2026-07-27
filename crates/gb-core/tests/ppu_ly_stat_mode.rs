//! ROADMAP 3.1a — LY ($FF44) e STAT ($FF41) bits de modo.
//! spec: docs/reference/06-ppu.md § FF44 LY, § FF41 STAT, § PPU modes.

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::Cpu;

const LCDC: u16 = 0xFF40;
const STAT: u16 = 0xFF41;
const LY: u16 = 0xFF44;
const LYC: u16 = 0xFF45;
const ENTRY: usize = 0x0100;

const LCDC_PPU_ENABLE: u8 = 0x80;

const MODE_HBLANK: u8 = 0;
const MODE_VBLANK: u8 = 1;
const MODE_OAM_SCAN: u8 = 2;
const MODE_DRAW: u8 = 3;

const DOTS_PER_SCANLINE: u32 = 456;
const M_CYCLES_PER_SCANLINE: u32 = DOTS_PER_SCANLINE / 4; // 114
const MODE_2_M_CYCLES: u32 = 80 / 4; // 20 (dots 0-79)
const MODE_3_M_CYCLES: u32 = 172 / 4; // 43 (dots 80-251)
const VBLANK_FIRST_LY: u8 = 144;
const TOTAL_LINES: u8 = 154;

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

fn stat_mode(bus: &Bus) -> u8 {
    bus.read(STAT) & 0x03
}

fn lyc_eq_ly(bus: &Bus) -> bool {
    bus.read(STAT) & 0x04 != 0
}

#[test]
fn ly_starts_at_zero() {
    let (_cpu, bus) = machine(&[]);

    assert_eq!(bus.read(LY), 0x00, "LY começa em 0 no hand-off da boot ROM");
}

#[test]
fn ly_increments_after_one_scanline_of_m_cycles() {
    let (mut cpu, mut bus) = machine(&[]);

    step_n(&mut cpu, &mut bus, M_CYCLES_PER_SCANLINE);

    assert_eq!(
        bus.read(LY),
        1,
        "após 114 M-cycles (1 scanline de 456 T-cycles), LY deve ser 1"
    );
}

#[test]
fn ly_increments_by_one_per_scanline_for_the_first_five_lines() {
    let (mut cpu, mut bus) = machine(&[]);

    for expected_ly in 0..5 {
        assert_eq!(
            bus.read(LY),
            expected_ly,
            "LY deve ser {expected_ly} após {} scanlines",
            expected_ly
        );
        step_n(&mut cpu, &mut bus, M_CYCLES_PER_SCANLINE);
    }
}

#[test]
fn ly_wraps_from_153_to_0() {
    let (mut cpu, mut bus) = machine(&[]);

    let m_cycles_for_frame = TOTAL_LINES as u32 * M_CYCLES_PER_SCANLINE;
    step_n(&mut cpu, &mut bus, m_cycles_for_frame);

    assert_eq!(
        bus.read(LY),
        0,
        "após 154 linhas (70224 T-cycles), LY dá a volta e volta a 0"
    );
}

#[test]
fn ly_goes_to_153_in_vblank() {
    let (mut cpu, mut bus) = machine(&[]);

    // Avança até a última linha do VBlank (LY=153)
    let m_cycles_to_153 = TOTAL_LINES.saturating_sub(1) as u32 * M_CYCLES_PER_SCANLINE;
    step_n(&mut cpu, &mut bus, m_cycles_to_153);

    assert_eq!(
        bus.read(LY),
        153,
        "LY vai até 153 durante o VBlank (linhas 144-153)"
    );
}

#[test]
fn ly_is_read_only_writes_are_ignored() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(LY, 0x5A);
    cpu.step(&mut bus);

    assert_ne!(
        bus.read(LY),
        0x5A,
        "LY é read-only: escrever $5A não altera o valor"
    );
}

#[test]
fn stat_reports_mode_2_at_start_of_scanline() {
    let (_cpu, bus) = machine(&[]);

    // No início da scanline (LY=0, dots=0), deve ser Mode 2 (OAM scan)
    assert_eq!(
        stat_mode(&bus),
        MODE_OAM_SCAN,
        "STAT bits 1-0 devem ser {MODE_OAM_SCAN} (OAM scan) no início da scanline"
    );
}

#[test]
fn stat_transitions_through_all_modes_in_a_scanline() {
    let (mut cpu, mut bus) = machine(&[]);

    // Mode 2 (OAM scan): dots 0-79, M-cycles 0-19
    step_n(&mut cpu, &mut bus, MODE_2_M_CYCLES - 1);
    assert_eq!(
        stat_mode(&bus),
        MODE_OAM_SCAN,
        "ainda em Mode 2 após {} M-cycles",
        MODE_2_M_CYCLES - 1
    );

    // Mode 3 (Drawing): dots 80-251, M-cycles 20-62
    step_n(&mut cpu, &mut bus, 1); // entra no Mode 3
    assert_eq!(
        stat_mode(&bus),
        MODE_DRAW,
        "transição para Mode 3 no M-cycle {}",
        MODE_2_M_CYCLES
    );

    // Mode 0 (HBlank): dots 252-455, M-cycles 63-113
    step_n(&mut cpu, &mut bus, MODE_3_M_CYCLES);
    assert_eq!(
        stat_mode(&bus),
        MODE_HBLANK,
        "transição para Mode 0 (HBlank) após Mode 3"
    );
}

#[test]
fn stat_reports_mode_1_during_vblank() {
    let (mut cpu, mut bus) = machine(&[]);

    // Avança até VBlank (LY ≥ 144)
    let m_cycles_to_vblank = VBLANK_FIRST_LY as u32 * M_CYCLES_PER_SCANLINE;
    step_n(&mut cpu, &mut bus, m_cycles_to_vblank);

    assert_eq!(
        bus.read(LY),
        VBLANK_FIRST_LY,
        "LY deve ser {} (primeira linha do VBlank)",
        VBLANK_FIRST_LY
    );
    assert_eq!(
        stat_mode(&bus),
        MODE_VBLANK,
        "STAT deve reportar Mode 1 (VBlank) quando LY ≥ 144"
    );
}

#[test]
fn stat_lyc_eq_ly_flag_is_set_when_ly_equals_lyc() {
    let (mut cpu, mut bus) = machine(&[]);

    // LYC e LY começam em 0 — a flag deve estar setada
    assert!(
        lyc_eq_ly(&bus),
        "STAT bit 2 (LYC=LY) deve estar setado quando LY=LYC=0"
    );

    // Avança uma scanline — LY vira 1, LYC continua 0, flag deve limpar
    step_n(&mut cpu, &mut bus, M_CYCLES_PER_SCANLINE);
    assert!(
        !lyc_eq_ly(&bus),
        "STAT bit 2 deve estar limpo quando LY=1 e LYC=0"
    );

    // Seta LYC=1 — flag deve voltar
    bus.write(LYC, 0x01);
    assert!(
        lyc_eq_ly(&bus),
        "STAT bit 2 deve estar setado quando LY=1 e LYC=1"
    );
}

#[test]
fn stat_writable_bits_are_preserved() {
    let (_cpu, mut bus) = machine(&[]);

    bus.write(STAT, 0xD8);
    assert_eq!(
        bus.read(STAT) & 0xF8,
        0xD8,
        "bits 7-3 de STAT preservam o valor escrito"
    );
}

#[test]
fn ppu_disabled_via_lcdc_resets_ly_and_mode() {
    let (mut cpu, mut bus) = machine(&[]);

    step_n(&mut cpu, &mut bus, 3 * M_CYCLES_PER_SCANLINE);
    assert_ne!(bus.read(LY), 0, "LY não é mais 0 após algumas scanlines");

    // Desliga o PPU (limpa bit 7 do LCDC)
    bus.write(LCDC, 0x00);
    cpu.step(&mut bus);

    assert_eq!(
        bus.read(LY),
        0,
        "LY deve voltar a 0 quando o PPU é desligado"
    );
    assert_eq!(
        stat_mode(&bus),
        MODE_HBLANK,
        "STAT deve reportar Mode 0 quando o PPU está desligado"
    );
    assert!(
        !lyc_eq_ly(&bus),
        "STAT LYC=LY deve estar limpo quando o PPU está desligado"
    );
}

#[test]
fn ppu_stays_stuck_when_disabled() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(LCDC, 0x00);
    cpu.step(&mut bus);

    assert_eq!(bus.read(LY), 0, "LY fica em 0 com PPU desligado");

    // Mais M-cycles não devem alterar LY
    step_n(&mut cpu, &mut bus, M_CYCLES_PER_SCANLINE);
    assert_eq!(bus.read(LY), 0, "LY não avança quando PPU está desligado");
}

#[test]
fn re_enabling_ppu_resumes_from_ly_zero() {
    let (mut cpu, mut bus) = machine(&[]);

    // Desliga
    bus.write(LCDC, 0x00);
    cpu.step(&mut bus);

    // Liga de novo
    bus.write(LCDC, LCDC_PPU_ENABLE);
    cpu.step(&mut bus);

    step_n(&mut cpu, &mut bus, M_CYCLES_PER_SCANLINE);
    assert_eq!(
        bus.read(LY),
        1,
        "após religar o PPU, LY volta a incrementar a partir de 0"
    );
}

#[test]
fn stat_int_select_bits_are_writable_individually() {
    let (_cpu, mut bus) = machine(&[]);

    for bit in 3..=6 {
        let mask = 1u8 << bit;
        bus.write(STAT, mask);
        assert_eq!(
            bus.read(STAT) & mask,
            mask,
            "STAT bit {bit} (interrupt select) deve ser setável"
        );
        bus.write(STAT, 0x00);
        assert_eq!(bus.read(STAT) & mask, 0, "STAT bit {bit} deve ser limpável");
    }
}

#[test]
fn lyc_is_readable_and_writable() {
    let (_cpu, mut bus) = machine(&[]);

    assert_eq!(bus.read(LYC), 0x00, "LYC começa em 0 no hand-off");

    bus.write(LYC, 0x5A);
    assert_eq!(bus.read(LYC), 0x5A, "LYC aceita escrita e devolve o valor");
}

#[test]
fn lcdc_is_readable_and_writable() {
    let (_cpu, mut bus) = machine(&[]);

    assert_eq!(
        bus.read(LCDC),
        0x91,
        "LCDC vale $91 no hand-off da boot ROM"
    );

    bus.write(LCDC, 0x83);
    assert_eq!(
        bus.read(LCDC),
        0x83,
        "LCDC aceita escrita e devolve o valor"
    );
}
