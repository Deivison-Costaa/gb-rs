//! ROADMAP 3.1c — VRAM ($8000–$9FFF) acessível pelo barramento.
//! spec: docs/reference/01-memory-map.md § VRAM memory map.

use gb_core::bus::Bus;
use gb_core::cart::{Cartridge, OPEN_BUS};

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

fn pattern(addr: u16) -> u8 {
    (addr as u8).wrapping_mul(17)
}

fn spread(addr: u16) -> u8 {
    addr.wrapping_mul(17) as u8
}

#[test]
fn vram_guarda_escrita_e_devolve_na_leitura() {
    let mut bus = bus();

    for addr in [0x8000u16, 0x8800, 0x9000, 0x9800, 0x9FFF] {
        bus.write(addr, pattern(addr));
        assert_eq!(
            bus.read(addr),
            pattern(addr),
            "${addr:04X} devia guardar o valor escrito"
        );
    }
}

#[test]
fn vram_tem_8_kib_sem_aliasing() {
    let mut bus = bus();

    for addr in 0x8000..=0x9FFFu16 {
        bus.write(addr, pattern(addr));
    }
    for addr in 0x8000..=0x9FFFu16 {
        assert_eq!(
            bus.read(addr),
            pattern(addr),
            "${addr:04X} devia guardar o próprio byte: VRAM são 8 KiB contíguos"
        );
    }
}

#[test]
fn vram_comeca_zerada_por_escolha() {
    let bus = bus();

    for addr in [0x8000u16, 0x8800, 0x9000, 0x9800, 0x9FFF] {
        assert_eq!(
            bus.read(addr),
            0x00,
            "${addr:04X}: VRAM começa zerada neste emulador"
        );
    }
}

#[test]
fn vram_nao_vaza_para_fora_da_regiao() {
    let mut bus = bus();

    bus.write(0x8000, 0xAB);
    assert_eq!(
        bus.read(0x7FFF),
        OPEN_BUS,
        "escrita em $8000 vazou para $7FFF"
    );
    assert_eq!(bus.read(0x7FFF), OPEN_BUS, "$7FFF é ROM, não VRAM");

    bus.write(0x9FFF, 0xCD);
    assert_eq!(
        bus.read(0xA000),
        OPEN_BUS,
        "escrita em $9FFF vazou para $A000"
    );
}

#[test]
fn vram_aceita_escrita_e_leitura_em_todo_o_range() {
    let mut bus = bus();

    for addr in (0x8000..=0x9FFF).step_by(0x511) {
        bus.write(addr, spread(addr));
    }
    for addr in (0x8000..=0x9FFF).step_by(0x511) {
        assert_eq!(bus.read(addr), spread(addr), "${addr:04X} perdeu o valor");
    }
}
