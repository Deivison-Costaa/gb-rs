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

// ── Interrupções PPU — ROADMAP 3.2 ─────────────────────────────────────

const IE: u16 = 0xFFFF;
const IF: u16 = 0xFF0F;

#[test]
fn vblank_interrupt_fires_when_entering_vblank() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(IE, 0x01);
    bus.write(IF, 0x00);

    // Entra no VBlank: LY=144 após 144 scanlines de 114 M-cycles
    let m_cycles_to_vblank = VBLANK_FIRST_LY as u32 * M_CYCLES_PER_SCANLINE;
    step_n(&mut cpu, &mut bus, m_cycles_to_vblank);

    assert_eq!(
        bus.read(LY),
        VBLANK_FIRST_LY,
        "LY deve ser 144 no início do VBlank"
    );
    assert_eq!(
        stat_mode(&bus),
        MODE_VBLANK,
        "STAT deve reportar Mode 1 (VBlank)"
    );
    assert_eq!(
        bus.read(IF) & 0x01,
        0x01,
        "IF bit 0 (VBlank) deve estar setado ao entrar no VBlank"
    );
}

#[test]
fn vblank_interrupt_does_not_fire_again_within_vblank() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(IE, 0x01);
    bus.write(IF, 0x00);

    // Entra no VBlank (LY=144)
    let m_cycles_to_vblank = VBLANK_FIRST_LY as u32 * M_CYCLES_PER_SCANLINE;
    step_n(&mut cpu, &mut bus, m_cycles_to_vblank);

    // Limpa o bit de VBlank de IF
    bus.write(IF, 0x00);

    // Avança mais uma scanline dentro do VBlank (LY=145)
    step_n(&mut cpu, &mut bus, M_CYCLES_PER_SCANLINE);

    assert_eq!(
        bus.read(LY),
        145,
        "LY deve ser 145 (uma scanline após o início do VBlank)"
    );
    assert_eq!(
        bus.read(IF) & 0x01,
        0x00,
        "IF bit 0 (VBlank) não deve ser setado de novo dentro do VBlank"
    );
}

#[test]
fn stat_interrupt_fires_on_mode_2_transition_when_enabled() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(STAT, 0x20); // bit 5 = Mode 2 int select
    bus.write(IE, 0x02);
    bus.write(IF, 0x00);

    // Avança até o início da scanline 1: Mode 0 (fim da scanline 0) → Mode 2
    step_n(&mut cpu, &mut bus, M_CYCLES_PER_SCANLINE);

    assert_eq!(bus.read(LY), 1, "LY deve ser 1");
    assert_eq!(
        stat_mode(&bus),
        MODE_OAM_SCAN,
        "STAT deve reportar Mode 2 no início da scanline"
    );
    assert_eq!(
        bus.read(IF) & 0x02,
        0x02,
        "IF bit 1 (LCD/STAT) deve estar setado na transição para Mode 2"
    );
}

#[test]
fn stat_interrupt_fires_on_mode_0_transition_when_enabled() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(STAT, 0x08); // bit 3 = Mode 0 int select
    bus.write(IE, 0x02);
    bus.write(IF, 0x00);

    // Avança até o fim do Mode 3 (M-cycles 20-62) — transição Mode 3→0
    // Mode 3 começa em dots=80 (M-cycle 20), dura 172 dots = 43 M-cycles
    // Transição Mode 3→0: dots=252 (M-cycle 63)
    step_n(&mut cpu, &mut bus, MODE_2_M_CYCLES + MODE_3_M_CYCLES);

    assert_eq!(bus.read(LY), 0, "ainda na scanline 0");
    assert_eq!(
        stat_mode(&bus),
        MODE_HBLANK,
        "STAT deve reportar Mode 0 (HBlank)"
    );
    assert_eq!(
        bus.read(IF) & 0x02,
        0x02,
        "IF bit 1 (LCD/STAT) deve estar setado na transição para Mode 0"
    );
}

#[test]
fn stat_interrupt_fires_on_lyc_equality_when_enabled() {
    let (mut cpu, mut bus) = machine(&[]);

    // LYC=5, LY=0 estão diferentes. Bit 6 = LYC int select.
    bus.write(LYC, 0x05);
    bus.write(STAT, 0x40); // bit 6 = LYC int select
    bus.write(IE, 0x02);
    bus.write(IF, 0x00);

    // Avança 5 scanlines: LY vai de 0 a 5
    step_n(&mut cpu, &mut bus, 5 * M_CYCLES_PER_SCANLINE);

    assert_eq!(bus.read(LY), 5, "LY deve ser 5");
    assert!(
        lyc_eq_ly(&bus),
        "STAT bit 2 (LYC=LY) deve estar setado quando LY=5 e LYC=5"
    );
    assert_eq!(
        bus.read(IF) & 0x02,
        0x02,
        "IF bit 1 (LCD/STAT) deve estar setado quando LY torna-se igual a LYC"
    );
}

#[test]
fn stat_interrupt_does_not_fire_when_no_int_select_bits_are_set() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(STAT, 0x00); // nenhum bit de int select
    bus.write(IE, 0x02);
    bus.write(IF, 0x00);

    // Avança uma scanline inteira (passa por Mode 2, 3, 0)
    step_n(&mut cpu, &mut bus, M_CYCLES_PER_SCANLINE);

    assert_eq!(
        bus.read(IF) & 0x02,
        0x00,
        "IF bit 1 (LCD/STAT) não deve ser setado sem int select"
    );
}

#[test]
fn stat_interrupt_does_not_fire_on_lyc_mismatch() {
    let (mut cpu, mut bus) = machine(&[]);

    bus.write(LYC, 0x10);
    bus.write(STAT, 0x40); // bit 6 = LYC int select
    bus.write(IE, 0x02);
    bus.write(IF, 0x00);

    // Avança 3 scanlines (LY=3, LYC=16 — sem coincidência)
    step_n(&mut cpu, &mut bus, 3 * M_CYCLES_PER_SCANLINE);

    assert_eq!(
        bus.read(IF) & 0x02,
        0x00,
        "IF bit 1 (LCD/STAT) não deve ser setado sem coincidência LYC=LY"
    );
}

#[test]
fn stat_blocking_mode_0_to_mode_1_prevents_vblank_stat_interrupt() {
    let (mut cpu, mut bus) = machine(&[]);

    // Habilita Mode 0 (bit 3) e Mode 1 (bit 4)
    bus.write(STAT, 0x18); // bits 3 e 4 = Mode 0 e Mode 1 int select
    bus.write(IE, 0x02);
    bus.write(IF, 0x00);

    // Avança até o fim da scanline 143 (entra em Mode 0 após Mode 3)
    // Mode 3→0 acontece por scanline. Precisamos passar do fim do Mode 3 em LY=143.
    // Avança: 143 scanlines completas + Mode 3 em LY=143 = 143*114 + 63 M-cycles
    step_n(
        &mut cpu,
        &mut bus,
        143 * M_CYCLES_PER_SCANLINE + MODE_2_M_CYCLES + MODE_3_M_CYCLES,
    );

    // Agora estamos em Mode 0 em LY=143. O primeiro STAT (Mode 0) deve ter disparado.
    let if_before = bus.read(IF) & 0x02;
    assert_eq!(
        if_before, 0x02,
        "STAT interrupt (Mode 0) deve ter disparado em LY=143"
    );

    // Limpa IF e avança até entrar em VBlank (LY=144)
    bus.write(IF, 0x00);

    // Avança o restante da scanline 143: Mode 0 termina, dots vira 0, LY=144, mode=1
    // Faltam: DOTS_PER_SCANLINE/4 - (MODE_2_M_CYCLES + MODE_3_M_CYCLES) = 114 - (20+43) = 51
    let remaining_mode0 = M_CYCLES_PER_SCANLINE - MODE_2_M_CYCLES - MODE_3_M_CYCLES;
    step_n(&mut cpu, &mut bus, remaining_mode0);

    assert_eq!(
        bus.read(LY),
        VBLANK_FIRST_LY,
        "LY deve ser 144 após terminar a scanline 143"
    );
    assert_eq!(
        stat_mode(&bus),
        MODE_VBLANK,
        "STAT deve reportar Mode 1 (VBlank)"
    );
    assert_eq!(
        bus.read(IF) & 0x02,
        0x00,
        "IF bit 1 (LCD/STAT) não deve ser setado para Mode 1 — STAT blocking"
    );
}
