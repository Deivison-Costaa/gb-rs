//! spec: `docs/reference/06-ppu.md`. ROADMAP 3.1a + 3.1b — registradores da PPU.

use crate::bus::boot::STAT_BOOT_WRITABLE;

const LCDC_ADDR: u16 = 0xFF40;
const STAT_ADDR: u16 = 0xFF41;
const SCY_ADDR: u16 = 0xFF42;
const SCX_ADDR: u16 = 0xFF43;
const LY_ADDR: u16 = 0xFF44;
const LYC_ADDR: u16 = 0xFF45;
const DMA_ADDR: u16 = 0xFF46;
const BGP_ADDR: u16 = 0xFF47;
const OBP0_ADDR: u16 = 0xFF48;
const OBP1_ADDR: u16 = 0xFF49;
const WY_ADDR: u16 = 0xFF4A;
const WX_ADDR: u16 = 0xFF4B;

const MODE_HBLANK: u8 = 0;
const MODE_VBLANK: u8 = 1;
const MODE_OAM_SCAN: u8 = 2;
const MODE_DRAW: u8 = 3;

const DOTS_PER_SCANLINE: u32 = 456;

const MODE_2_END: u32 = 80;

const MODE_3_BASE: u32 = 172;

const VBLANK_START_LY: u8 = 144;

const TOTAL_LINES: u8 = 154;

pub(crate) struct Ppu {
    dots: u32,
    scy: u8,
    scx: u8,
    ly: u8,
    lcdc: u8,
    lyc: u8,
    dma: u8,
    bgp: u8,
    obp0: u8,
    obp1: u8,
    wy: u8,
    wx: u8,
    stat_writable: u8,
}

impl Ppu {
    pub(crate) fn new() -> Self {
        Self {
            dots: 0,
            scy: 0x00,
            scx: 0x00,
            ly: 0,
            lcdc: 0x91,
            lyc: 0x00,
            dma: 0xFF,
            bgp: 0xFC,
            obp0: 0x00,
            obp1: 0x00,
            wy: 0x00,
            wx: 0x00,
            stat_writable: STAT_BOOT_WRITABLE,
        }
    }

    pub(crate) fn tick(&mut self) {
        if self.lcdc & 0x80 == 0 {
            self.dots = 0;
            self.ly = 0;
            return;
        }

        self.dots += 4;

        if self.dots >= DOTS_PER_SCANLINE {
            self.dots = 0;
            self.ly = (self.ly + 1) % TOTAL_LINES;
        }
    }

    pub(crate) fn read(&self, addr: u16) -> u8 {
        match addr {
            LCDC_ADDR => self.lcdc,
            STAT_ADDR => {
                let mode = self.current_mode();
                let lyc_flag = if self.ly == self.lyc && self.lcdc & 0x80 != 0 {
                    0x04
                } else {
                    0x00
                };
                self.stat_writable | lyc_flag | mode
            }
            SCY_ADDR => self.scy,
            SCX_ADDR => self.scx,
            LY_ADDR => self.ly,
            LYC_ADDR => self.lyc,
            DMA_ADDR => self.dma,
            BGP_ADDR => self.bgp,
            OBP0_ADDR => self.obp0,
            OBP1_ADDR => self.obp1,
            WY_ADDR => self.wy,
            WX_ADDR => self.wx,
            _ => unreachable!(
                "ppu só atende ${:04X}-${:04X}, recebeu ${:04X}",
                LCDC_ADDR, WX_ADDR, addr
            ),
        }
    }

    pub(crate) fn write(&mut self, addr: u16, value: u8) {
        match addr {
            LCDC_ADDR => self.lcdc = value,
            STAT_ADDR => self.stat_writable = value & 0xF8,
            SCY_ADDR => self.scy = value,
            SCX_ADDR => self.scx = value,
            LY_ADDR => {}
            LYC_ADDR => self.lyc = value,
            DMA_ADDR => self.dma = value,
            BGP_ADDR => self.bgp = value,
            OBP0_ADDR => self.obp0 = value,
            OBP1_ADDR => self.obp1 = value,
            WY_ADDR => self.wy = value,
            WX_ADDR => self.wx = value,
            _ => unreachable!(
                "ppu só atende ${:04X}-${:04X}, recebeu ${:04X}",
                LCDC_ADDR, WX_ADDR, addr
            ),
        }
    }

    fn current_mode(&self) -> u8 {
        if self.lcdc & 0x80 == 0 {
            return MODE_HBLANK;
        }
        if self.ly >= VBLANK_START_LY {
            return MODE_VBLANK;
        }
        if self.dots < MODE_2_END {
            return MODE_OAM_SCAN;
        }
        let mode3_end = MODE_2_END + MODE_3_BASE;
        if self.dots < mode3_end {
            return MODE_DRAW;
        }
        MODE_HBLANK
    }
}
