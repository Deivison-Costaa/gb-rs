//! ROADMAP 3.1b — registradores restantes da PPU: SCY, SCX, DMA, BGP, OBP0, OBP1, WY, WX.
//! spec: docs/reference/06-ppu.md § FF42–FF4B.

use gb_core::bus::Bus;
use gb_core::cart::Cartridge;
use gb_core::cart::OPEN_BUS;

const SCY: u16 = 0xFF42;
const SCX: u16 = 0xFF43;
const DMA: u16 = 0xFF46;
const BGP: u16 = 0xFF47;
const OBP0: u16 = 0xFF48;
const OBP1: u16 = 0xFF49;
const WY: u16 = 0xFF4A;
const WX: u16 = 0xFF4B;

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

#[test]
fn scy_starts_at_zero_and_is_writable() {
    let mut bus = bus();

    assert_eq!(
        bus.read(SCY),
        0x00,
        "SCY começa em 0 no hand-off da boot ROM"
    );

    bus.write(SCY, 0x7F);
    assert_eq!(bus.read(SCY), 0x7F, "SCY aceita escrita de $7F");

    bus.write(SCY, 0xFF);
    assert_eq!(bus.read(SCY), 0xFF, "SCY aceita escrita de $FF");

    bus.write(SCY, 0x00);
    assert_eq!(bus.read(SCY), 0x00, "SCY volta a 0");
}

#[test]
fn scx_starts_at_zero_and_is_writable() {
    let mut bus = bus();

    assert_eq!(
        bus.read(SCX),
        0x00,
        "SCX começa em 0 no hand-off da boot ROM"
    );

    bus.write(SCX, 0xAA);
    assert_eq!(bus.read(SCX), 0xAA, "SCX aceita escrita de $AA");

    bus.write(SCX, 0x00);
    assert_eq!(bus.read(SCX), 0x00, "SCX volta a 0");
}

#[test]
fn dma_starts_at_0xff_and_is_writable() {
    let mut bus = bus();

    assert_eq!(bus.read(DMA), 0xFF, "DMA começa em $FF (DMG, não CGB/AGB)");

    bus.write(DMA, 0xC0);
    assert_eq!(bus.read(DMA), 0xC0, "stub de DMA devolve o valor escrito");

    bus.write(DMA, 0x00);
    assert_eq!(bus.read(DMA), 0x00, "stub de DMA aceita $00");
}

#[test]
fn bgp_starts_at_0xfc_and_is_writable() {
    let mut bus = bus();

    assert_eq!(
        bus.read(BGP),
        0xFC,
        "BGP começa em $FC no hand-off da boot ROM"
    );

    bus.write(BGP, 0xE4);
    assert_eq!(bus.read(BGP), 0xE4, "BGP aceita escrita de $E4");

    bus.write(BGP, 0x00);
    assert_eq!(bus.read(BGP), 0x00, "BGP aceita $00");
}

#[test]
fn obp0_starts_at_zero_and_is_writable() {
    let mut bus = bus();

    assert_eq!(
        bus.read(OBP0),
        0x00,
        "OBP0 começa em $00 (?? na spec, escolha do emulador)"
    );

    bus.write(OBP0, 0xE4);
    assert_eq!(bus.read(OBP0), 0xE4, "OBP0 aceita escrita de $E4");

    bus.write(OBP0, 0x00);
    assert_eq!(bus.read(OBP0), 0x00, "OBP0 volta a 0");
}

#[test]
fn obp1_starts_at_zero_and_is_writable() {
    let mut bus = bus();

    assert_eq!(
        bus.read(OBP1),
        0x00,
        "OBP1 começa em $00 (?? na spec, escolha do emulador)"
    );

    bus.write(OBP1, 0x1B);
    assert_eq!(bus.read(OBP1), 0x1B, "OBP1 aceita escrita de $1B");

    bus.write(OBP1, 0x00);
    assert_eq!(bus.read(OBP1), 0x00, "OBP1 volta a 0");
}

#[test]
fn wy_starts_at_zero_and_is_writable() {
    let mut bus = bus();

    assert_eq!(bus.read(WY), 0x00, "WY começa em 0 no hand-off da boot ROM");

    bus.write(WY, 0x10);
    assert_eq!(bus.read(WY), 0x10, "WY aceita escrita de $10");

    bus.write(WY, 0xFF);
    assert_eq!(bus.read(WY), 0xFF, "WY aceita escrita de $FF");
}

#[test]
fn wx_starts_at_zero_and_is_writable() {
    let mut bus = bus();

    assert_eq!(bus.read(WX), 0x00, "WX começa em 0 no hand-off da boot ROM");

    bus.write(WX, 0xA7);
    assert_eq!(bus.read(WX), 0xA7, "WX aceita escrita de $A7");

    bus.write(WX, 0x00);
    assert_eq!(bus.read(WX), 0x00, "WX volta a 0");
}

#[test]
fn ppu_scroll_registers_are_independent() {
    let mut bus = bus();

    bus.write(SCY, 0x42);
    bus.write(SCX, 0x43);

    assert_eq!(bus.read(SCY), 0x42, "SCY conserva o valor escrito");
    assert_eq!(bus.read(SCX), 0x43, "SCX não interfere em SCY");
}

#[test]
fn ppu_palette_registers_are_independent() {
    let mut bus = bus();

    bus.write(BGP, 0xE4);
    bus.write(OBP0, 0x1B);
    bus.write(OBP1, 0xA5);

    assert_eq!(bus.read(BGP), 0xE4, "BGP conserva o valor escrito");
    assert_eq!(bus.read(OBP0), 0x1B, "OBP0 conserva o valor escrito");
    assert_eq!(bus.read(OBP1), 0xA5, "OBP1 conserva o valor escrito");
}

#[test]
fn ppu_window_registers_are_independent() {
    let mut bus = bus();

    bus.write(WY, 0x50);
    bus.write(WX, 0x60);

    assert_eq!(bus.read(WY), 0x50, "WY conserva o valor escrito");
    assert_eq!(bus.read(WX), 0x60, "WX não interfere em WY");
}

#[test]
fn writes_to_ppu_registers_do_not_interfere_across_groups() {
    let mut bus = bus();

    bus.write(SCY, 0x42);
    bus.write(BGP, 0xE4);
    bus.write(WY, 0x30);
    bus.write(DMA, 0xC0);

    assert_eq!(bus.read(SCY), 0x42, "escrever em BGP/WY/DMA não altera SCY");
    assert_eq!(bus.read(BGP), 0xE4, "escrever em SCY/WY/DMA não altera BGP");
    assert_eq!(bus.read(WY), 0x30, "escrever em SCY/BGP/DMA não altera WY");
    assert_eq!(bus.read(DMA), 0xC0, "escrever em SCY/BGP/WY não altera DMA");
}
