//! ROADMAP 3.5b — DMA de OAM ($FF46).
//! spec: docs/reference/06-ppu.md § OAM DMA Transfer.

use gb_core::bus::Bus;
use gb_core::cart::{Cartridge, OPEN_BUS};

const DMA: u16 = 0xFF46;
const LCDC: u16 = 0xFF40;
const OAM_BASE: u16 = 0xFE00;
const WRAM_BASE: u16 = 0xC000;

const OAM_LEN: u16 = 0xA0;
const DMA_M_CYCLES: usize = 160;

struct RomCart {
    rom: Vec<u8>,
}

impl Cartridge for RomCart {
    fn read(&self, addr: u16) -> u8 {
        self.rom.get(addr as usize).copied().unwrap_or(OPEN_BUS)
    }

    fn write(&mut self, _addr: u16, _value: u8) {}
}

fn bus() -> Bus {
    let mut bus = Bus::new(Box::new(RomCart {
        rom: vec![0; 0x8000],
    }));
    bus.write(LCDC, 0x00);
    bus
}

fn fill_wram(bus: &mut Bus) {
    for i in 0..OAM_LEN {
        bus.write(WRAM_BASE + i, (i as u8).wrapping_mul(3).wrapping_add(1));
    }
}

fn expected(i: u16) -> u8 {
    (i as u8).wrapping_mul(3).wrapping_add(1)
}

#[test]
fn dma_copia_os_160_bytes_da_fonte_para_a_oam() {
    let mut bus = bus();
    fill_wram(&mut bus);

    bus.write(DMA, 0xC0);
    for _ in 0..DMA_M_CYCLES {
        bus.tick_dma();
    }

    for i in 0..OAM_LEN {
        assert_eq!(
            bus.read(OAM_BASE + i),
            expected(i),
            "byte {i} da OAM não veio da DMA"
        );
    }
}

#[test]
fn dma_transfere_um_byte_por_m_cycle() {
    let mut bus = bus();
    fill_wram(&mut bus);

    bus.write(DMA, 0xC0);
    for _ in 0..80 {
        bus.tick_dma();
    }

    assert!(
        bus.dma_active(),
        "na metade dos 160 M-cycles a DMA ainda deveria estar ativa"
    );

    for _ in 0..80 {
        bus.tick_dma();
    }

    assert!(
        !bus.dma_active(),
        "depois de 160 M-cycles a DMA deveria ter terminado"
    );
}

#[test]
fn oam_fica_inacessivel_a_cpu_enquanto_a_dma_roda() {
    let mut bus = bus();
    fill_wram(&mut bus);

    bus.write(OAM_BASE, 0x11);
    bus.write(DMA, 0xC0);
    bus.tick_dma();

    assert_eq!(
        bus.read(OAM_BASE + 0x50),
        OPEN_BUS,
        "com a DMA rodando a CPU lê open bus na OAM"
    );

    bus.write(OAM_BASE + 0x50, 0x99);
    for _ in 0..DMA_M_CYCLES {
        bus.tick_dma();
    }

    assert_eq!(
        bus.read(OAM_BASE + 0x50),
        expected(0x50),
        "escrita da CPU durante a DMA não pode sobreviver"
    );
}

const HRAM_BASE: u16 = 0xFF80;
const ROM_ADDR: u16 = 0x0100;
const VRAM_ADDR: u16 = 0x8000;

#[test]
fn durante_a_dma_a_cpu_so_enxerga_hram() {
    let mut bus = bus();
    fill_wram(&mut bus);
    bus.write(HRAM_BASE, 0x42);

    bus.write(DMA, 0xC0);
    bus.tick_dma();

    assert_eq!(
        bus.read(HRAM_BASE),
        0x42,
        "a HRAM é o único caminho que sobra para a CPU durante a DMA"
    );
    for blocked in [ROM_ADDR, VRAM_ADDR, WRAM_BASE, 0xE000, OAM_BASE] {
        assert_eq!(
            bus.read(blocked),
            OPEN_BUS,
            "${blocked:04X} está no barramento que a DMA tomou"
        );
    }
}

#[test]
fn a_dma_nao_bloqueia_a_faixa_de_io() {
    let mut bus = bus();

    bus.write(0xFF07, 0x05);
    bus.write(DMA, 0xC0);
    bus.tick_dma();

    assert_eq!(
        bus.read(0xFF07),
        0x05,
        "TAC segue legível: IF/IE e companhia são internos à CPU, não passam pelo barramento tomado"
    );
}

#[test]
fn escrita_fora_da_hram_nao_pega_durante_a_dma() {
    let mut bus = bus();
    fill_wram(&mut bus);

    bus.write(DMA, 0xC0);
    bus.tick_dma();
    bus.write(0xD000, 0x5E);

    for _ in 0..DMA_M_CYCLES {
        bus.tick_dma();
    }

    assert_eq!(
        bus.read(0xD000),
        0x00,
        "escrita da CPU na WRAM durante a DMA é engolida"
    );
}

#[test]
fn ff46_continua_legivel_com_o_ultimo_valor_escrito() {
    let mut bus = bus();

    bus.write(DMA, 0xC0);

    assert_eq!(bus.read(DMA), 0xC0);
}

#[test]
fn dma_com_fonte_na_rom() {
    let mut rom = vec![0u8; 0x8000];
    for (i, byte) in rom.iter_mut().enumerate().take(0x4000).skip(0x1000) {
        *byte = (i as u8) ^ 0x5A;
    }
    let mut bus = Bus::new(Box::new(RomCart { rom }));
    bus.write(LCDC, 0x00);

    bus.write(DMA, 0x10);
    for _ in 0..DMA_M_CYCLES {
        bus.tick_dma();
    }

    for i in 0..OAM_LEN {
        assert_eq!(bus.read(OAM_BASE + i), (i as u8) ^ 0x5A);
    }
}

#[test]
fn escrita_em_ff46_durante_a_dma_reinicia_a_transferencia() {
    let mut bus = bus();
    fill_wram(&mut bus);
    for i in 0..OAM_LEN {
        bus.write(0xD000 + i, 0x77);
    }

    bus.write(DMA, 0xC0);
    for _ in 0..40 {
        bus.tick_dma();
    }
    bus.write(DMA, 0xD0);
    for _ in 0..DMA_M_CYCLES {
        bus.tick_dma();
    }

    for i in 0..OAM_LEN {
        assert_eq!(
            bus.read(OAM_BASE + i),
            0x77,
            "a segunda DMA sobrescreve tudo"
        );
    }
}
