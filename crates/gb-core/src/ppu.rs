//! spec: `docs/reference/06-ppu.md`. ROADMAP 3.1a + 3.1b + 3.2 + 3.3 — registradores, modos, interrupções e background por scanline.

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

pub(crate) const SCREEN_W: usize = 160;
pub(crate) const SCREEN_H: usize = 144;

#[derive(Debug, Default)]
pub(crate) struct PpuSignals {
    pub vblank_interrupt: bool,
    pub stat_interrupt: bool,
    pub begin_mode3: bool,
}

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
    framebuffer: [u8; SCREEN_W * SCREEN_H],
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
            framebuffer: [0x00; SCREEN_W * SCREEN_H],
        }
    }

    pub(crate) fn framebuffer(&self) -> &[u8; SCREEN_W * SCREEN_H] {
        &self.framebuffer
    }

    pub(crate) fn tick(&mut self) -> PpuSignals {
        if self.lcdc & 0x80 == 0 {
            self.dots = 0;
            self.ly = 0;
            return PpuSignals::default();
        }

        let old_stat_line = self.compute_stat_line();
        let old_ly = self.ly;

        let old_dots = self.dots;
        self.dots += 4;

        let mut begin_mode3 = false;
        let mut vblank_fired = false;
        if self.dots >= DOTS_PER_SCANLINE {
            self.dots = 0;
            let new_ly = (self.ly + 1) % TOTAL_LINES;
            if old_ly == VBLANK_START_LY - 1 && new_ly == VBLANK_START_LY {
                vblank_fired = true;
            }
            self.ly = new_ly;
        }

        if old_dots < MODE_2_END && self.dots >= MODE_2_END && self.ly < VBLANK_START_LY {
            begin_mode3 = true;
        }

        let new_stat_line = self.compute_stat_line();
        let stat_interrupt = new_stat_line && !old_stat_line;

        PpuSignals {
            vblank_interrupt: vblank_fired,
            stat_interrupt,
            begin_mode3,
        }
    }

    pub(crate) fn render_background_scanline(&mut self, vram: &[u8]) {
        let ly = self.ly as usize;
        if ly >= SCREEN_H {
            return;
        }

        let row_offset = ly * SCREEN_W;

        if self.lcdc & 0x01 == 0 {
            let shade = self.bgp & 0x03;
            for x in 0..SCREEN_W {
                self.framebuffer[row_offset + x] = shade;
            }
            return;
        }

        let tilemap_base = if self.lcdc & 0x08 != 0 {
            0x9C00
        } else {
            0x9800
        };
        let unsigned = self.lcdc & 0x10 != 0;

        for pixel_x in 0..SCREEN_W {
            let bg_x = (pixel_x as u16).wrapping_add(self.scx as u16) & 0xFF;
            let bg_y = (ly as u16).wrapping_add(self.scy as u16) & 0xFF;

            let tile_col = bg_x / 8;
            let tile_row = bg_y / 8;

            let tilemap_addr = tilemap_base + tile_row * 32 + tile_col;
            let tile_index = vram[tilemap_addr as usize - 0x8000];

            let tile_addr = if unsigned {
                0x8000u16 + tile_index as u16 * 16
            } else {
                0x9000u16.wrapping_add_signed(tile_index as i8 as i16 * 16)
            };

            let line_in_tile = (bg_y & 7) as usize;
            let byte0_addr = tile_addr as usize - 0x8000 + line_in_tile * 2;
            let byte0 = vram[byte0_addr];
            let byte1 = vram[byte0_addr + 1];

            let pixel_in_tile = 7 - (bg_x & 7);
            let color_idx = ((byte1 >> pixel_in_tile) & 1) << 1 | ((byte0 >> pixel_in_tile) & 1);

            let shade = (self.bgp >> (color_idx * 2)) & 0x03;
            self.framebuffer[row_offset + pixel_x] = shade;
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

    fn lyc_eq_ly(&self) -> bool {
        self.lcdc & 0x80 != 0 && self.ly == self.lyc
    }

    fn compute_stat_line(&self) -> bool {
        if self.lcdc & 0x80 == 0 {
            return false;
        }
        let mode = self.current_mode();
        let lyc_match = self.lyc_eq_ly();

        let stat = self.stat_writable;
        let sel_mode0 = (stat >> 3) & 1 != 0;
        let sel_mode1 = (stat >> 4) & 1 != 0;
        let sel_mode2 = (stat >> 5) & 1 != 0;
        let sel_lyc = (stat >> 6) & 1 != 0;

        (mode == MODE_HBLANK && sel_mode0)
            || (mode == MODE_VBLANK && sel_mode1)
            || (mode == MODE_OAM_SCAN && sel_mode2)
            || (lyc_match && sel_lyc)
    }
}
