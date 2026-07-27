//! ROADMAP 3.1d — OAM ($FE00–$FE9F) acessível pelo barramento.
//! spec: docs/reference/01-memory-map.md § Object attribute memory (OAM).
//! spec: docs/reference/06-ppu.md § Accessing VRAM and OAM — OAM só é
//! acessível em Mode 0/1 ou com PPU desligado; o barramento bloqueia nos demais.

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

fn bus() -> Bus {
    Bus::new(Box::new(MinimalCart))
}

fn bus_oam_accessible() -> Bus {
    let mut bus = bus();
    bus.write(LCDC, 0x00);
    bus.tick_ppu();
    bus
}

fn pattern(addr: u16) -> u8 {
    (addr as u8).wrapping_mul(17)
}

fn spread(addr: u16) -> u8 {
    addr.wrapping_mul(17) as u8
}

#[test]
fn oam_guarda_escrita_e_devolve_na_leitura() {
    let mut bus = bus_oam_accessible();

    for addr in [0xFE00u16, 0xFE40, 0xFE80, 0xFE9F] {
        bus.write(addr, pattern(addr));
        assert_eq!(
            bus.read(addr),
            pattern(addr),
            "${addr:04X} devia guardar o valor escrito"
        );
    }
}

#[test]
fn oam_tem_160_bytes_sem_aliasing() {
    let mut bus = bus_oam_accessible();

    for addr in 0xFE00..=0xFE9Fu16 {
        bus.write(addr, pattern(addr));
    }
    for addr in 0xFE00..=0xFE9Fu16 {
        assert_eq!(
            bus.read(addr),
            pattern(addr),
            "${addr:04X} devia guardar o próprio byte: OAM são 160 bytes contíguos"
        );
    }
}

#[test]
fn oam_comeca_zerada_por_escolha() {
    let bus = bus_oam_accessible();

    for addr in [0xFE00u16, 0xFE40, 0xFE80, 0xFE9F] {
        assert_eq!(
            bus.read(addr),
            0x00,
            "${addr:04X}: OAM começa zerada neste emulador"
        );
    }
}

#[test]
fn oam_nao_vaza_para_fora_da_regiao() {
    let mut bus = bus_oam_accessible();

    for addr in 0xFE00..=0xFE9Fu16 {
        bus.write(addr, 0x37);
    }

    assert_eq!(
        bus.read(0xFDFF),
        0x00,
        "$FDFF (Echo RAM) não devia receber escrita de OAM"
    );

    assert_eq!(
        bus.read(0xFEA0),
        0x00,
        "$FEA0 (NotUsable) não devia receber escrita de OAM"
    );
}

#[test]
fn oam_aceita_escrita_e_leitura_em_todo_o_range() {
    let mut bus = bus_oam_accessible();

    for addr in (0xFE00..=0xFE9F).step_by(0x3B) {
        bus.write(addr, spread(addr));
    }
    for addr in (0xFE00..=0xFE9F).step_by(0x3B) {
        assert_eq!(bus.read(addr), spread(addr), "${addr:04X} perdeu o valor");
    }
}
