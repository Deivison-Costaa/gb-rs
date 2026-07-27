//! spec: `docs/reference/06-ppu.md` § FF44 LY, § FF41 STAT, § PPU modes.
//! ROADMAP 3.1a — LY e bits de modo do STAT.

use crate::bus::boot::STAT_BOOT_WRITABLE;

const LCDC_ADDR: u16 = 0xFF40;
const STAT_ADDR: u16 = 0xFF41;
const LY_ADDR: u16 = 0xFF44;
const LYC_ADDR: u16 = 0xFF45;

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
    ly: u8,
    lcdc: u8,
    lyc: u8,
    stat_writable: u8,
}

impl Ppu {
    pub(crate) fn new() -> Self {
        Self {
            dots: 0,
            ly: 0,
            lcdc: 0x91,
            lyc: 0x00,
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
            LY_ADDR => self.ly,
            LYC_ADDR => self.lyc,
            _ => unreachable!(
                "ppu só atende ${:04X}-${:04X}, recebeu ${:04X}",
                LCDC_ADDR, LYC_ADDR, addr
            ),
        }
    }

    pub(crate) fn write(&mut self, addr: u16, value: u8) {
        match addr {
            LCDC_ADDR => self.lcdc = value,
            STAT_ADDR => self.stat_writable = value & 0xF8,
            LY_ADDR => {}
            LYC_ADDR => self.lyc = value,
            _ => unreachable!(
                "ppu só atende ${:04X}-${:04X}, recebeu ${:04X}",
                LCDC_ADDR, LYC_ADDR, addr
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
