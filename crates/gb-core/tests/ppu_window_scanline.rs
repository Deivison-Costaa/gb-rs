//! ROADMAP 3.4 — Window por scanline: contador interno de linha, WX/WY, LCDC.5/LCDC.6.
//! spec: docs/reference/06-ppu.md § Window, § Window behavior.

use gb_core::bus::Bus;
use gb_core::cart::{Cartridge, OPEN_BUS};

const VRAM_BASE: u16 = 0x8000;
const TILEMAP_BASE: u16 = 0x9800;
const TILEMAP2_BASE: u16 = 0x9C00;

const LCDC: u16 = 0xFF40;
const SCX: u16 = 0xFF43;
const WY: u16 = 0xFF4A;
const WX: u16 = 0xFF4B;
const BGP: u16 = 0xFF47;

const LCDC_PPU_ENABLE: u8 = 0x80;
const LCDC_BG_ENABLE: u8 = 0x01;
const LCDC_WIN_ENABLE: u8 = 0x20;
const LCDC_WIN_MAP: u8 = 0x40;
const LCDC_TILE_SELECT: u8 = 0x10;

const DOTS_PER_M_CYCLE: u32 = 4;
const MODE_3_ENTRY_DOTS: u32 = 80;

struct MinimalCart;

impl Cartridge for MinimalCart {
    fn read(&self, _addr: u16) -> u8 {
        OPEN_BUS
    }

    fn write(&mut self, _addr: u16, _value: u8) {}
}

fn bus() -> Bus {
    Bus::new(Box::new(MinimalCart))
}

fn step_into_mode3(bus: &mut Bus) {
    let ticks_needed = MODE_3_ENTRY_DOTS / DOTS_PER_M_CYCLE;
    for _ in 0..ticks_needed {
        bus.tick_ppu();
        bus.tick_timer();
    }
}

fn step_scanline(bus: &mut Bus) {
    for _ in 0..(456 / DOTS_PER_M_CYCLE) {
        bus.tick_ppu();
        bus.tick_timer();
    }
}

fn set_tile_data(bus: &mut Bus, tile_index: u8, bytes: &[u8; 16]) {
    let addr = VRAM_BASE + tile_index as u16 * 16;
    for (i, &b) in bytes.iter().enumerate() {
        bus.write(addr + i as u16, b);
    }
}

fn fill_tilemap(bus: &mut Bus, base: u16, tile_index: u8) {
    for offset in 0..1024 {
        bus.write(base + offset, tile_index);
    }
}

fn write_lcdc(bus: &mut Bus, bits: u8) {
    bus.write(LCDC, bits);
}

fn write_bgp(bus: &mut Bus, value: u8) {
    bus.write(BGP, value);
}

fn black_tile() -> [u8; 16] {
    [0xFFu8; 16]
}

const BGP_IDENTITY: u8 = 0xE4;

// ── testes ──────────────────────────────────────────────────────────────

#[test]
fn window_covers_entire_screen_when_wx7_wy0_and_lcdc5_set() {
    let mut bus = bus();

    set_tile_data(&mut bus, 0, &black_tile());
    fill_tilemap(&mut bus, TILEMAP_BASE, 0);

    set_tile_data(&mut bus, 1, &[0x00u8; 16]);
    fill_tilemap(&mut bus, TILEMAP2_BASE, 1);

    write_lcdc(
        &mut bus,
        LCDC_PPU_ENABLE | LCDC_BG_ENABLE | LCDC_WIN_ENABLE | LCDC_WIN_MAP | LCDC_TILE_SELECT,
    );
    write_bgp(&mut bus, BGP_IDENTITY);
    bus.write(WX, 7);
    bus.write(WY, 0);

    step_into_mode3(&mut bus);

    let fb = bus.framebuffer();
    assert_eq!(fb[0], 0, "WX=7 WY=0: window tilemap $9C00 tile 1 shade 0");
    assert_eq!(fb[8], 0, "WX=7 WY=0: pixel 8 também é window shade 0");
}

#[test]
fn window_uses_its_own_tilemap_when_lcdc6_set() {
    let mut bus = bus();

    set_tile_data(&mut bus, 0, &black_tile());
    fill_tilemap(&mut bus, TILEMAP_BASE, 0);

    set_tile_data(&mut bus, 1, &[0x00u8; 16]);
    fill_tilemap(&mut bus, TILEMAP2_BASE, 1);

    write_lcdc(
        &mut bus,
        LCDC_PPU_ENABLE | LCDC_BG_ENABLE | LCDC_WIN_ENABLE | LCDC_WIN_MAP | LCDC_TILE_SELECT,
    );
    write_bgp(&mut bus, BGP_IDENTITY);
    bus.write(WX, 7);
    bus.write(WY, 0);

    step_into_mode3(&mut bus);

    let fb = bus.framebuffer();
    assert_eq!(fb[0], 0, "LCDC.6=1: window tilemap $9C00 tile 1 shade 0");
}

#[test]
fn window_disabled_when_lcdc5_clear_shows_background_only() {
    let mut bus = bus();

    set_tile_data(&mut bus, 0, &black_tile());
    fill_tilemap(&mut bus, TILEMAP_BASE, 0);

    write_lcdc(
        &mut bus,
        LCDC_PPU_ENABLE | LCDC_BG_ENABLE | LCDC_TILE_SELECT,
    );
    write_bgp(&mut bus, BGP_IDENTITY);
    bus.write(WX, 7);
    bus.write(WY, 0);

    step_into_mode3(&mut bus);

    let fb = bus.framebuffer();
    assert_eq!(fb[0], 3, "LCDC.5=0: BG renderiza tile 0 shade 3");
}

#[test]
fn window_not_visible_when_wy_gt_ly_shows_background() {
    let mut bus = bus();

    set_tile_data(&mut bus, 0, &black_tile());
    fill_tilemap(&mut bus, TILEMAP_BASE, 0);

    write_lcdc(
        &mut bus,
        LCDC_PPU_ENABLE | LCDC_BG_ENABLE | LCDC_WIN_ENABLE | LCDC_TILE_SELECT,
    );
    write_bgp(&mut bus, BGP_IDENTITY);
    bus.write(WX, 7);
    bus.write(WY, 5);

    step_into_mode3(&mut bus);

    let fb = bus.framebuffer();
    assert_eq!(fb[0], 3, "WY=5 > LY=0: Y condition false, BG shade 3");
}

#[test]
fn window_starts_at_wx_minus_7_position() {
    let mut bus = bus();

    set_tile_data(&mut bus, 0, &black_tile());
    fill_tilemap(&mut bus, TILEMAP_BASE, 0);

    set_tile_data(&mut bus, 1, &[0x00u8; 16]);
    fill_tilemap(&mut bus, TILEMAP2_BASE, 1);

    write_lcdc(
        &mut bus,
        LCDC_PPU_ENABLE | LCDC_BG_ENABLE | LCDC_WIN_ENABLE | LCDC_WIN_MAP | LCDC_TILE_SELECT,
    );
    write_bgp(&mut bus, BGP_IDENTITY);
    bus.write(WX, 7 + 8);
    bus.write(WY, 0);

    step_into_mode3(&mut bus);

    let fb = bus.framebuffer();
    assert_eq!(fb[0], 3, "WX=15: pixels 0..7 são BG (tile 0, shade 3)");
    assert_eq!(
        fb[8], 0,
        "WX=15: pixel 8 (WX-7) entra na window (tile 1, shade 0)"
    );
}

#[test]
fn window_line_counter_increments_within_tile_when_visible() {
    let mut bus = bus();

    // Tile com linha 0 = shade 3 (0xFF), linha 1 = shade 0 (0x00)
    let striped: [u8; 16] = [
        0xFF, 0xFF, // linha 0: shade 3
        0x00, 0x00, // linha 1: shade 0
        0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00,
    ];
    set_tile_data(&mut bus, 0, &striped);
    fill_tilemap(&mut bus, TILEMAP_BASE, 0);

    write_lcdc(
        &mut bus,
        LCDC_PPU_ENABLE | LCDC_BG_ENABLE | LCDC_WIN_ENABLE | LCDC_TILE_SELECT,
    );
    write_bgp(&mut bus, BGP_IDENTITY);
    bus.write(WX, 7);
    bus.write(WY, 0);

    // scanline LY=0: window_line=0
    step_into_mode3(&mut bus);
    assert_eq!(
        bus.framebuffer()[0],
        3,
        "LY=0 window_line=0: linha 0 do tile shade 3"
    );

    // Avança para LY=1
    step_scanline(&mut bus);
    step_into_mode3(&mut bus);
    assert_eq!(
        bus.framebuffer()[160],
        0,
        "LY=1 window_line=1: linha 1 do tile shade 0"
    );
}

#[test]
fn window_line_counter_does_not_increment_when_wy_not_reached() {
    let mut bus = bus();

    set_tile_data(&mut bus, 0, &black_tile());
    fill_tilemap(&mut bus, TILEMAP_BASE, 0);

    write_lcdc(
        &mut bus,
        LCDC_PPU_ENABLE | LCDC_BG_ENABLE | LCDC_WIN_ENABLE | LCDC_TILE_SELECT,
    );
    write_bgp(&mut bus, BGP_IDENTITY);
    bus.write(WX, 7);
    bus.write(WY, 2);

    step_scanline(&mut bus);
    step_into_mode3(&mut bus);

    assert_eq!(
        bus.framebuffer()[160],
        3,
        "LY=1 WY=2: window não ativa, BG shade 3, pixel na linha 1 da tela"
    );
}

#[test]
fn wx_zero_covers_entire_screen_without_scx_shift() {
    let mut bus = bus();

    set_tile_data(&mut bus, 0, &black_tile());
    fill_tilemap(&mut bus, TILEMAP_BASE, 0);

    set_tile_data(&mut bus, 1, &[0x00u8; 16]);
    fill_tilemap(&mut bus, TILEMAP2_BASE, 1);

    write_lcdc(
        &mut bus,
        LCDC_PPU_ENABLE | LCDC_BG_ENABLE | LCDC_WIN_ENABLE | LCDC_WIN_MAP | LCDC_TILE_SELECT,
    );
    write_bgp(&mut bus, BGP_IDENTITY);
    bus.write(WX, 0);
    bus.write(WY, 0);
    bus.write(SCX, 0);

    step_into_mode3(&mut bus);

    let fb = bus.framebuffer();
    assert_eq!(
        fb[0], 0,
        "WX=0 SCX=0: window cobre tela inteira, tile 1 shade 0"
    );
}

#[test]
fn wx_166_bug_covers_entire_screen_instead_of_one_pixel() {
    let mut bus = bus();

    set_tile_data(&mut bus, 0, &black_tile());
    fill_tilemap(&mut bus, TILEMAP_BASE, 0);

    set_tile_data(&mut bus, 1, &[0x00u8; 16]);
    fill_tilemap(&mut bus, TILEMAP2_BASE, 1);

    write_lcdc(
        &mut bus,
        LCDC_PPU_ENABLE | LCDC_BG_ENABLE | LCDC_WIN_ENABLE | LCDC_WIN_MAP | LCDC_TILE_SELECT,
    );
    write_bgp(&mut bus, BGP_IDENTITY);
    bus.write(WX, 166);
    bus.write(WY, 0);

    step_into_mode3(&mut bus);

    let fb = bus.framebuffer();
    assert_eq!(
        fb[0], 0,
        "WX=166 bug: window cobre tela inteira, pixel 0 é window shade 0"
    );
    assert_eq!(fb[8], 0, "WX=166 bug: pixel 8 também é window");
}
