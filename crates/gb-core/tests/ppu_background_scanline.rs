//! ROADMAP 3.3 — background por scanline: tilemap, tiledata, endereçamento signed/unsigned.
//! spec: docs/reference/06-ppu.md § VRAM Tile Data, § VRAM Tile Maps, § Scrolling.

use gb_core::bus::Bus;
use gb_core::cart::{Cartridge, OPEN_BUS};

const SCREEN_W: usize = 160;

const VRAM_BASE: u16 = 0x8000;
const TILEMAP_BASE: u16 = 0x9800;
const TILEMAP2_BASE: u16 = 0x9C00;

const LCDC: u16 = 0xFF40;
const SCY: u16 = 0xFF42;
const SCX: u16 = 0xFF43;
const BGP: u16 = 0xFF47;

const LCDC_PPU_ENABLE: u8 = 0x80;
const LCDC_BG_ENABLE: u8 = 0x01;
const LCDC_TILE_SELECT: u8 = 0x10; // 1=$8000 unsigned, 0=$8800 signed
const LCDC_BG_MAP: u8 = 0x08; // 1=$9C00, 0=$9800

const DOTS_PER_M_CYCLE: u32 = 4;
const MODE_3_ENTRY_DOTS: u32 = 80;

struct MinimalCart;

impl Cartridge for MinimalCart {
    fn read(&self, _addr: u16) -> u8 {
        OPEN_BUS
    }

    fn write(&mut self, _addr: u16, _value: u8) {}
}

/// Cria um Bus com PPU ligada e BG habilitado, sem dados em VRAM.
fn bus() -> Bus {
    Bus::new(Box::new(MinimalCart))
}

/// Avança a PPU até o início do Mode 3 da próxima scanline.
/// A PPU começa com dots=0, LY=0, no Mode 2. Depois de 20 M-cycles
/// (80 dots) entra no Mode 3 e renderiza o background da scanline.
fn step_into_mode3(bus: &mut Bus) {
    let ticks_needed = MODE_3_ENTRY_DOTS / DOTS_PER_M_CYCLE;
    for _ in 0..ticks_needed {
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

fn write_scx(bus: &mut Bus, value: u8) {
    bus.write(SCX, value);
}

fn write_scy(bus: &mut Bus, value: u8) {
    bus.write(SCY, value);
}

fn write_bgp(bus: &mut Bus, value: u8) {
    bus.write(BGP, value);
}

// ── tile de padrão conhecido ────────────────────────────────────────────

/// Cria um tile de 8×8 onde cada linha tem os mesmos 8 pixels:
/// cor 0, 3, 2, 1, 0, 3, 2, 1 (da esquerda para a direita).
/// byte0 (LSB) = 0x55, byte1 (MSB) = 0x66 para cada linha.
fn checker_tile() -> [u8; 16] {
    let lo: u8 = 0x55;
    let hi: u8 = 0x66;
    [
        lo, hi, lo, hi, lo, hi, lo, hi, lo, hi, lo, hi, lo, hi, lo, hi,
    ]
}

/// BGP = $E4: mapeia cor 0→0 (branco), 1→1 (cinza claro),
/// 2→2 (cinza escuro), 3→3 (preto) — identidade.
const BGP_IDENTITY: u8 = 0xE4;

// ── testes ──────────────────────────────────────────────────────────────

#[test]
fn renders_background_pixels_from_tilemap_with_unsigned_addressing() {
    let mut bus = bus();

    set_tile_data(&mut bus, 0, &checker_tile());
    fill_tilemap(&mut bus, TILEMAP_BASE, 0);
    write_lcdc(
        &mut bus,
        LCDC_PPU_ENABLE | LCDC_BG_ENABLE | LCDC_TILE_SELECT,
    );
    write_bgp(&mut bus, BGP_IDENTITY);
    write_scx(&mut bus, 0);
    write_scy(&mut bus, 0);

    step_into_mode3(&mut bus);

    let fb = bus.framebuffer();
    let row0 = &fb[0..SCREEN_W];

    assert_eq!(row0[0], 0, "pixel 0 deve ser shade 0 (branco)");
    assert_eq!(row0[1], 3, "pixel 1 deve ser shade 3 (preto)");
    assert_eq!(row0[2], 2, "pixel 2 deve ser shade 2 (cinza escuro)");
    assert_eq!(row0[3], 1, "pixel 3 deve ser shade 1 (cinza claro)");

    assert_eq!(row0[8], 0, "segundo tile repete padrão: pixel 8 = shade 0");
    assert_eq!(row0[9], 3, "segundo tile: pixel 9 = shade 3");
}

#[test]
fn signed_addressing_maps_tile_index_128_to_block_1() {
    let mut bus = bus();

    let pattern = [0xFFu8; 16];
    set_tile_data(&mut bus, 128, &pattern);

    fill_tilemap(&mut bus, TILEMAP_BASE, 128);
    write_lcdc(&mut bus, LCDC_PPU_ENABLE | LCDC_BG_ENABLE);
    write_bgp(&mut bus, BGP_IDENTITY);
    write_scx(&mut bus, 0);
    write_scy(&mut bus, 0);

    step_into_mode3(&mut bus);

    let fb = bus.framebuffer();
    assert_eq!(
        fb[0], 3,
        "tile 128 com endereçamento signed (−128) lê do bloco 1 ($8800),\
         tile de cor 3 em todos os pixels"
    );
}

#[test]
fn scx_shifts_pixels_horizontally() {
    let mut bus = bus();

    set_tile_data(&mut bus, 0, &checker_tile());
    fill_tilemap(&mut bus, TILEMAP_BASE, 0);
    write_lcdc(
        &mut bus,
        LCDC_PPU_ENABLE | LCDC_BG_ENABLE | LCDC_TILE_SELECT,
    );
    write_bgp(&mut bus, BGP_IDENTITY);
    write_scx(&mut bus, 1);
    write_scy(&mut bus, 0);

    step_into_mode3(&mut bus);

    let fb = bus.framebuffer();

    assert_eq!(
        fb[0], 3,
        "com SCX=1, pixel 0 da tela é o pixel 1 do tile (deslocamento de 1)"
    );
    assert_eq!(fb[1], 2, "com SCX=1, pixel 1 da tela é o pixel 2 do tile");
}

#[test]
fn lcdc0_disabled_fills_screen_with_bgp_color_0() {
    let mut bus = bus();

    set_tile_data(&mut bus, 0, &checker_tile());
    fill_tilemap(&mut bus, TILEMAP_BASE, 0);
    write_lcdc(&mut bus, LCDC_PPU_ENABLE | LCDC_TILE_SELECT);
    write_bgp(&mut bus, 0x1B);
    write_scx(&mut bus, 0);
    write_scy(&mut bus, 0);

    step_into_mode3(&mut bus);

    let fb = bus.framebuffer();
    for pixel in &fb[0..SCREEN_W] {
        assert_eq!(
            *pixel, 3,
            "com LCDC.0=0 e BGP cor 0=3, fundo desligado mostra shade 3 (cor 0 do BGP)"
        );
    }
}

#[test]
fn scy_shifts_pixels_vertically() {
    let mut bus0 = bus();

    let mut tile = [0x00u8; 16];
    tile[0] = 0xFF;
    tile[1] = 0xFF;
    set_tile_data(&mut bus0, 0, &tile);

    fill_tilemap(&mut bus0, TILEMAP_BASE, 0);
    write_lcdc(
        &mut bus0,
        LCDC_PPU_ENABLE | LCDC_BG_ENABLE | LCDC_TILE_SELECT,
    );
    write_bgp(&mut bus0, BGP_IDENTITY);
    write_scx(&mut bus0, 0);
    write_scy(&mut bus0, 0);

    step_into_mode3(&mut bus0);

    assert_eq!(
        bus0.framebuffer()[0],
        3,
        "SCY=0: linha 0 do tile (cor 3) na linha 0 da tela"
    );

    let mut bus1 = bus();
    set_tile_data(&mut bus1, 0, &tile);
    fill_tilemap(&mut bus1, TILEMAP_BASE, 0);
    write_lcdc(
        &mut bus1,
        LCDC_PPU_ENABLE | LCDC_BG_ENABLE | LCDC_TILE_SELECT,
    );
    write_bgp(&mut bus1, BGP_IDENTITY);
    write_scx(&mut bus1, 0);
    write_scy(&mut bus1, 1);

    step_into_mode3(&mut bus1);

    assert_eq!(
        bus1.framebuffer()[0],
        0,
        "SCY=1: pixel 0 da tela é linha 1 do tile (cor 0)"
    );
}

#[test]
fn lcdc3_selects_alternate_tilemap() {
    let mut bus = bus();

    set_tile_data(&mut bus, 0, &checker_tile());

    let alt_pattern = [0xFFu8; 16];
    set_tile_data(&mut bus, 1, &alt_pattern);

    fill_tilemap(&mut bus, TILEMAP_BASE, 0);
    bus.write(TILEMAP2_BASE, 1);
    fill_tilemap(&mut bus, TILEMAP2_BASE + 1, 0);

    write_lcdc(
        &mut bus,
        LCDC_PPU_ENABLE | LCDC_BG_ENABLE | LCDC_TILE_SELECT | LCDC_BG_MAP,
    );
    write_bgp(&mut bus, BGP_IDENTITY);
    write_scx(&mut bus, 0);
    write_scy(&mut bus, 0);

    step_into_mode3(&mut bus);

    let fb = bus.framebuffer();
    assert_eq!(
        fb[0], 3,
        "LCDC.3=1 usa tilemap $9C00 que tem tile 1 nos primeiros 8 pixels (cor 3)"
    );
    assert_eq!(
        fb[8], 0,
        "segundo tile do $9C00 é tile 0: pixel 8 = shade 0"
    );
}
