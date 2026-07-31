//! ROADMAP 3.5 — sprites: OAM scan, limite 10/linha, prioridade, flip X/Y, modo 8×16.
//! spec: docs/reference/06-ppu.md § Object Attribute Memory (OAM), § Rendering overview.

use gb_core::bus::Bus;
use gb_core::cart::{Cartridge, OPEN_BUS};

const VRAM_BASE: u16 = 0x8000;
const OAM_BASE: u16 = 0xFE00;

const LCDC: u16 = 0xFF40;
const BGP: u16 = 0xFF47;
const OBP0: u16 = 0xFF48;
const OBP1: u16 = 0xFF49;

const LCDC_PPU_ENABLE: u8 = 0x80;
const LCDC_BG_ENABLE: u8 = 0x01;
const LCDC_OBJ_ENABLE: u8 = 0x02;
const LCDC_OBJ_SIZE: u8 = 0x04;

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
    let mut bus = Bus::new(Box::new(MinimalCart));
    bus.write(LCDC, 0x00);
    bus.tick_ppu();
    bus
}

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

fn write_sprite(bus: &mut Bus, index: u8, y: u8, x: u8, tile: u8, attr: u8) {
    let base = OAM_BASE + index as u16 * 4;
    bus.write(base, y);
    bus.write(base + 1, x);
    bus.write(base + 2, tile);
    bus.write(base + 3, attr);
}

fn solid_tile(color_idx: u8) -> [u8; 16] {
    let (lo, hi) = match color_idx & 3 {
        0 => (0x00, 0x00),
        1 => (0xFF, 0x00),
        2 => (0x00, 0xFF),
        3 => (0xFF, 0xFF),
        _ => unreachable!(),
    };
    let mut tile = [0u8; 16];
    for i in 0..8 {
        tile[i * 2] = lo;
        tile[i * 2 + 1] = hi;
    }
    tile
}

const PAL_IDENTITY: u8 = 0xE4;
const PAL_OBP1_MAP_1_TO_2: u8 = 0xD8;

const LCDC_TILE_SELECT: u8 = 0x10;

#[test]
fn sprite_de_8x8_renderiza_pixels_nao_transparentes() {
    let mut bus = bus();

    set_tile_data(&mut bus, 0, &solid_tile(2));

    write_sprite(&mut bus, 0, 16, 8, 0, 0x00);

    bus.write(OBP0, PAL_IDENTITY);
    bus.write(LCDC, LCDC_PPU_ENABLE | LCDC_OBJ_ENABLE);

    step_into_mode3(&mut bus);

    let fb = bus.framebuffer();
    for (x, &shade) in fb.iter().enumerate().take(8) {
        assert_eq!(
            shade, 2,
            "pixel esquerdo x={} do sprite deveria ser shade 2, é {}",
            x, shade
        );
    }
}

#[test]
fn cor_0_do_sprite_eh_transparente() {
    let mut bus = bus();

    set_tile_data(&mut bus, 0, &solid_tile(0));

    write_sprite(&mut bus, 0, 16, 8, 0, 0x00);

    bus.write(BGP, PAL_IDENTITY);
    bus.write(OBP0, PAL_IDENTITY);
    bus.write(LCDC, LCDC_PPU_ENABLE | LCDC_OBJ_ENABLE);

    step_into_mode3(&mut bus);

    let fb = bus.framebuffer();
    assert_eq!(
        fb[4], 0,
        "cor 0 do sprite é transparente — pixel 4 deveria ser 0 (BG fill), é {}",
        fb[4]
    );
}

#[test]
fn atributo_palette_seleciona_obp0_ou_obp1() {
    let mut bus = bus();

    set_tile_data(&mut bus, 0, &solid_tile(1));

    write_sprite(&mut bus, 0, 16, 8, 0, 0x00);
    write_sprite(&mut bus, 1, 16, 16, 0, 0x10);

    bus.write(OBP0, PAL_IDENTITY);
    bus.write(OBP1, PAL_OBP1_MAP_1_TO_2);
    bus.write(LCDC, LCDC_PPU_ENABLE | LCDC_OBJ_ENABLE);

    step_into_mode3(&mut bus);

    let fb = bus.framebuffer();
    assert_eq!(
        fb[4], 1,
        "sprite 0 usa OBP0 — shade deveria ser 1, é {}",
        fb[4]
    );
    assert_eq!(
        fb[12], 2,
        "sprite 1 usa OBP1 ($D2 mapeia cor 1→2) — shade deveria ser 2, é {}",
        fb[12]
    );
}

#[test]
fn sprite_com_flip_x_espelha_horizontalmente() {
    let mut bus1 = bus();

    let mut tile = [0u8; 16];
    tile[0] = 0x80;
    tile[1] = 0x80;
    set_tile_data(&mut bus1, 0, &tile);

    write_sprite(&mut bus1, 0, 16, 8, 0, 0x00);

    bus1.write(OBP0, PAL_IDENTITY);
    bus1.write(LCDC, LCDC_PPU_ENABLE | LCDC_OBJ_ENABLE);

    step_into_mode3(&mut bus1);

    let fb = bus1.framebuffer();
    assert_eq!(fb[0], 3, "sem flip: pixel x=0 deveria ser 3, é {}", fb[0]);
    assert_eq!(
        fb[1], 0,
        "sem flip: pixel x=1 deveria ser 0 (transparente), é {}",
        fb[1]
    );

    let mut bus2 = bus();
    set_tile_data(&mut bus2, 0, &tile);
    write_sprite(&mut bus2, 0, 16, 8, 0, 0x20);
    bus2.write(OBP0, PAL_IDENTITY);
    bus2.write(LCDC, LCDC_PPU_ENABLE | LCDC_OBJ_ENABLE);
    step_into_mode3(&mut bus2);

    let fb2 = bus2.framebuffer();
    assert_eq!(
        fb2[0], 0,
        "com flip X: pixel x=0 deveria ser 0 (transparente), é {}",
        fb2[0]
    );
    assert_eq!(
        fb2[7], 3,
        "com flip X: pixel x=7 deveria ser 3 (espelhado), é {}",
        fb2[7]
    );
}

#[test]
fn sprite_com_flip_y_espelha_verticalmente() {
    let mut tile = [0u8; 16];
    tile[0] = 0x80;
    tile[1] = 0x00;
    tile[14] = 0x00;
    tile[15] = 0x80;

    let mut bus1 = bus();
    set_tile_data(&mut bus1, 0, &tile);
    write_sprite(&mut bus1, 0, 16, 8, 0, 0x00);
    bus1.write(OBP0, PAL_IDENTITY);
    bus1.write(LCDC, LCDC_PPU_ENABLE | LCDC_OBJ_ENABLE);
    step_into_mode3(&mut bus1);
    assert_eq!(
        bus1.framebuffer()[0],
        1,
        "sem flip Y: pixel x=0 usa linha 0 (cor 1), é {}",
        bus1.framebuffer()[0]
    );

    let mut bus2 = bus();
    set_tile_data(&mut bus2, 0, &tile);
    write_sprite(&mut bus2, 0, 16, 8, 0, 0x40);
    bus2.write(OBP0, PAL_IDENTITY);
    bus2.write(LCDC, LCDC_PPU_ENABLE | LCDC_OBJ_ENABLE);
    step_into_mode3(&mut bus2);
    assert_eq!(
        bus2.framebuffer()[0],
        2,
        "com flip Y: pixel x=0 usa linha 7 (cor 2), é {}",
        bus2.framebuffer()[0]
    );
}

#[test]
fn sprite_8x16_usa_dois_tiles_empilhados() {
    let mut bus1 = bus();

    set_tile_data(&mut bus1, 0, &solid_tile(1));
    set_tile_data(&mut bus1, 1, &solid_tile(2));

    write_sprite(&mut bus1, 0, 16, 8, 0, 0x00);

    bus1.write(OBP0, PAL_IDENTITY);
    bus1.write(LCDC, LCDC_PPU_ENABLE | LCDC_OBJ_ENABLE | LCDC_OBJ_SIZE);

    step_into_mode3(&mut bus1);

    let fb = bus1.framebuffer();
    assert_eq!(
        fb[4], 1,
        "8×16 LY=0 usa tile superior (0) — shade deveria ser 1, é {}",
        fb[4]
    );

    let mut bus2 = bus();
    set_tile_data(&mut bus2, 0, &solid_tile(1));
    set_tile_data(&mut bus2, 1, &solid_tile(2));
    write_sprite(&mut bus2, 0, 8, 8, 0, 0x00);
    bus2.write(OBP0, PAL_IDENTITY);
    bus2.write(LCDC, LCDC_PPU_ENABLE | LCDC_OBJ_ENABLE | LCDC_OBJ_SIZE);
    step_into_mode3(&mut bus2);

    let fb2 = bus2.framebuffer();
    assert_eq!(
        fb2[4], 2,
        "8×16 LY=0 com Y=8 usa tile inferior (1) — shade deveria ser 2, é {}",
        fb2[4]
    );
}

#[test]
fn sprite_menor_x_tem_prioridade() {
    let mut bus = bus();

    set_tile_data(&mut bus, 0, &solid_tile(1));
    set_tile_data(&mut bus, 1, &solid_tile(2));

    write_sprite(&mut bus, 0, 16, 12, 0, 0x00);
    write_sprite(&mut bus, 1, 16, 8, 1, 0x00);

    bus.write(OBP0, PAL_IDENTITY);
    bus.write(LCDC, LCDC_PPU_ENABLE | LCDC_OBJ_ENABLE);

    step_into_mode3(&mut bus);

    let fb = bus.framebuffer();
    assert_eq!(
        fb[6], 2,
        "sobreposição: sprite 1 (X=8, menor) tem prioridade sobre sprite 0 (X=12) — pixel 6 deveria ser 2, é {}",
        fb[6]
    );
}

#[test]
fn bg_over_obj_esconde_sprite_quando_bg_cores_1_3() {
    let mut bus = bus();

    let mut tile_data = [0u8; 16];
    tile_data[0] = 0xFF;
    tile_data[1] = 0xFF;
    set_tile_data(&mut bus, 0, &tile_data);
    set_tile_data(&mut bus, 1, &solid_tile(2));

    bus.write(0x9800, 0);

    write_sprite(&mut bus, 0, 16, 8, 1, 0x80);

    bus.write(BGP, PAL_IDENTITY);
    bus.write(OBP0, PAL_IDENTITY);
    bus.write(
        LCDC,
        LCDC_PPU_ENABLE | LCDC_BG_ENABLE | LCDC_OBJ_ENABLE | LCDC_TILE_SELECT,
    );

    step_into_mode3(&mut bus);

    let fb = bus.framebuffer();
    assert_eq!(
        fb[4], 3,
        "BG-over-OBJ: BG cor 3 sobrepõe sprite — pixel 4 deveria ser 3, é {}",
        fb[4]
    );
}

// A prioridade olha o índice de cor do BG, não o shade que a BGP produziu
// (ver docs/iterations/0087).
const PAL_COR_0_VIRA_SHADE_3: u8 = 0xE7;
const PAL_TUDO_SHADE_0: u8 = 0x00;

#[test]
fn bg_over_obj_nao_esconde_sprite_sobre_bg_cor_0() {
    let mut bus = bus();

    set_tile_data(&mut bus, 0, &solid_tile(0));
    set_tile_data(&mut bus, 1, &solid_tile(2));

    bus.write(0x9800, 0);

    write_sprite(&mut bus, 0, 16, 8, 1, 0x80);

    bus.write(BGP, PAL_COR_0_VIRA_SHADE_3);
    bus.write(OBP0, PAL_IDENTITY);
    bus.write(
        LCDC,
        LCDC_PPU_ENABLE | LCDC_BG_ENABLE | LCDC_OBJ_ENABLE | LCDC_TILE_SELECT,
    );

    step_into_mode3(&mut bus);

    let fb = bus.framebuffer();
    assert_eq!(
        fb[4], 2,
        "BG cor 0 pintada de shade 3 pela BGP ainda perde para o sprite — pixel 4 deveria ser 2, é {}",
        fb[4]
    );
}

#[test]
fn bg_over_obj_esconde_sprite_sobre_bg_cor_3_pintada_de_shade_0() {
    let mut bus = bus();

    set_tile_data(&mut bus, 0, &solid_tile(3));
    set_tile_data(&mut bus, 1, &solid_tile(2));

    bus.write(0x9800, 0);

    write_sprite(&mut bus, 0, 16, 8, 1, 0x80);

    bus.write(BGP, PAL_TUDO_SHADE_0);
    bus.write(OBP0, PAL_IDENTITY);
    bus.write(
        LCDC,
        LCDC_PPU_ENABLE | LCDC_BG_ENABLE | LCDC_OBJ_ENABLE | LCDC_TILE_SELECT,
    );

    step_into_mode3(&mut bus);

    let fb = bus.framebuffer();
    assert_eq!(
        fb[4], 0,
        "BG cor 3 vence o sprite mesmo pintada de shade 0 — pixel 4 deveria ser 0, é {}",
        fb[4]
    );
}

#[test]
fn objs_desabilitado_nao_renderiza_sprites() {
    let mut bus = bus();

    set_tile_data(&mut bus, 0, &solid_tile(2));
    write_sprite(&mut bus, 0, 16, 8, 0, 0x00);

    bus.write(OBP0, PAL_IDENTITY);
    bus.write(LCDC, LCDC_PPU_ENABLE);

    step_into_mode3(&mut bus);

    let fb = bus.framebuffer();
    assert_eq!(
        fb[4], 0,
        "OBJ desabilitado — pixel 4 deveria ser 0 (BG fill), é {}",
        fb[4]
    );
}

#[test]
fn sprite_x_igual_zero_nao_renderiza_mas_conta_no_limite() {
    let mut bus = bus();

    set_tile_data(&mut bus, 0, &solid_tile(2));

    for i in 0..11 {
        write_sprite(&mut bus, i, 16, 0, 0, 0x00);
    }
    write_sprite(&mut bus, 11, 16, 8, 0, 0x00);

    bus.write(OBP0, PAL_IDENTITY);
    bus.write(LCDC, LCDC_PPU_ENABLE | LCDC_OBJ_ENABLE);

    step_into_mode3(&mut bus);

    let fb = bus.framebuffer();
    assert_eq!(
        fb[4], 0,
        "11 sprites X=0 consomem o limite de 10 — sprite 11 (X=8) não renderiza, pixel 4 deveria ser 0, é {}",
        fb[4]
    );
}
