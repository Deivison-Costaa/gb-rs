//! ROADMAP 3.6 — Bloqueio de acesso a VRAM e OAM por modo da PPU.
//! spec: docs/reference/06-ppu.md § Accessing VRAM and OAM.

use gb_core::bus::Bus;
use gb_core::cart::{Cartridge, OPEN_BUS};

struct MinimalCart;

impl Cartridge for MinimalCart {
    fn read(&self, _addr: u16) -> u8 {
        OPEN_BUS
    }

    fn write(&mut self, _addr: u16, _value: u8) {}
}

const LCDC: u16 = 0xFF40;
const VRAM_ADDR: u16 = 0x8000;
const OAM_ADDR: u16 = 0xFE00;

const MODE_2_TICKS: u32 = 80 / 4; // 20 ticks para sair do Mode 2
const MODE_3_TICKS: u32 = 172 / 4; // 43 ticks no Mode 3
const ENTER_MODE_3: u32 = MODE_2_TICKS + 1; // um tick dentro do Mode 3
const ENTER_MODE_0: u32 = MODE_2_TICKS + MODE_3_TICKS; // primeiro tick do Mode 0

fn bus() -> Bus {
    Bus::new(Box::new(MinimalCart))
}

fn tick_n(bus: &mut Bus, n: u32) {
    for _ in 0..n {
        bus.tick_ppu();
    }
}

fn disable_ppu(bus: &mut Bus) {
    bus.write(LCDC, 0x00);
    bus.tick_ppu();
}

fn enable_ppu(bus: &mut Bus) {
    bus.write(LCDC, 0x91);
    bus.tick_ppu();
}

// ── VRAM ─────────────────────────────────────────────────────────────────

#[test]
fn vram_read_returns_ff_during_mode_3() {
    let mut bus = bus();
    bus.write(VRAM_ADDR, 0x5A);

    tick_n(&mut bus, ENTER_MODE_3);

    assert_eq!(
        bus.read(VRAM_ADDR),
        0xFF,
        "VRAM devolve $FF durante Mode 3 (PPU está renderizando)"
    );
}

#[test]
fn vram_write_is_ignored_during_mode_3() {
    let mut bus = bus();
    bus.write(VRAM_ADDR, 0x42);

    tick_n(&mut bus, ENTER_MODE_3);
    bus.write(VRAM_ADDR, 0xFF);

    tick_n(&mut bus, ENTER_MODE_0 - ENTER_MODE_3);

    assert_eq!(
        bus.read(VRAM_ADDR),
        0x42,
        "escrita em VRAM durante Mode 3 deve ser ignorada; o valor antigo ($42) persiste"
    );
}

#[test]
fn vram_is_accessible_during_mode_2() {
    let mut bus = bus();
    bus.write(VRAM_ADDR, 0x7C);

    assert_eq!(
        bus.read(VRAM_ADDR),
        0x7C,
        "VRAM está acessível em Mode 2 (OAM scan)"
    );
}

#[test]
fn vram_is_accessible_during_mode_0() {
    let mut bus = bus();
    bus.write(VRAM_ADDR, 0xAB);

    tick_n(&mut bus, ENTER_MODE_0);

    assert_eq!(
        bus.read(VRAM_ADDR),
        0xAB,
        "VRAM está acessível em Mode 0 (HBlank)"
    );
}

#[test]
fn vram_is_accessible_when_ppu_disabled() {
    let mut bus = bus();
    bus.write(VRAM_ADDR, 0x33);

    disable_ppu(&mut bus);

    assert_eq!(
        bus.read(VRAM_ADDR),
        0x33,
        "VRAM está acessível quando o PPU está desligado (LCDC.7=0)"
    );
}

// ── OAM ──────────────────────────────────────────────────────────────────

#[test]
fn oam_read_returns_ff_during_mode_2() {
    let bus = bus();

    assert_eq!(
        bus.read(OAM_ADDR),
        0xFF,
        "OAM devolve $FF durante Mode 2 (PPU está lendo OAM)"
    );
}

#[test]
fn oam_read_returns_ff_during_mode_3() {
    let mut bus = bus();

    tick_n(&mut bus, ENTER_MODE_3);

    assert_eq!(
        bus.read(OAM_ADDR),
        0xFF,
        "OAM devolve $FF durante Mode 3 (PPU está renderizando)"
    );
}

#[test]
fn oam_write_is_ignored_during_mode_2() {
    let mut bus = bus();

    disable_ppu(&mut bus);
    bus.write(OAM_ADDR, 0x11);
    enable_ppu(&mut bus);

    bus.write(OAM_ADDR, 0x77);

    disable_ppu(&mut bus);

    assert_eq!(
        bus.read(OAM_ADDR),
        0x11,
        "escrita em OAM durante Mode 2 deve ser ignorada; valor anterior ($11) persiste"
    );
}

#[test]
fn oam_write_is_ignored_during_mode_3() {
    let mut bus = bus();

    disable_ppu(&mut bus);
    bus.write(OAM_ADDR, 0x11);
    enable_ppu(&mut bus);

    tick_n(&mut bus, ENTER_MODE_3);
    bus.write(OAM_ADDR, 0xEE);

    tick_n(&mut bus, ENTER_MODE_0 - ENTER_MODE_3);

    assert_eq!(
        bus.read(OAM_ADDR),
        0x11,
        "escrita em OAM durante Mode 3 deve ser ignorada; valor anterior ($11) persiste"
    );
}

#[test]
fn oam_is_accessible_during_mode_0() {
    let mut bus = bus();

    tick_n(&mut bus, ENTER_MODE_0);
    bus.write(OAM_ADDR, 0xCD);

    assert_eq!(
        bus.read(OAM_ADDR),
        0xCD,
        "OAM está acessível em Mode 0 (HBlank)"
    );
}

#[test]
fn oam_is_accessible_when_ppu_disabled() {
    let mut bus = bus();

    disable_ppu(&mut bus);
    bus.write(OAM_ADDR, 0x5A);

    assert_eq!(
        bus.read(OAM_ADDR),
        0x5A,
        "OAM está acessível quando o PPU está desligado (LCDC.7=0)"
    );
}
