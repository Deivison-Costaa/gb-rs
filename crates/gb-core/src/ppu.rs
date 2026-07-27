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

const MAX_SPRITES: usize = 10;

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
    window_line: u8,
    window_y_condition: bool,
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
            window_line: 0,
            window_y_condition: false,
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

        if self.dots == 0 && self.ly == self.wy && self.ly < VBLANK_START_LY {
            self.window_y_condition = true;
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
                self.window_y_condition = false;
                self.window_line = 0;
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

    pub(crate) fn render_scanline(&mut self, vram: &[u8], oam: &[u8]) {
        self.render_background_scanline(vram);
        if self.lcdc & 0x02 != 0 {
            self.render_sprites(oam, vram);
        }
    }

    fn render_sprites(&mut self, oam: &[u8], vram: &[u8]) {
        let ly = self.ly as i32;
        if ly >= SCREEN_H as i32 {
            return;
        }
        let row_offset = ly as usize * SCREEN_W;
        let obj_height: i32 = if self.lcdc & 0x04 != 0 { 16 } else { 8 };

        let mut selected: [(u8, u8, u8, u8, u8); MAX_SPRITES] = [(0, 0, 0, 0, 0); MAX_SPRITES];
        let mut count = 0;

        for i in 0..40 {
            let y = oam[i * 4];
            let sprite_y = (y as i32) - 16;
            if sprite_y <= ly && ly < sprite_y + obj_height && count < MAX_SPRITES {
                let x = oam[i * 4 + 1];
                let tile = oam[i * 4 + 2];
                let attr = oam[i * 4 + 3];
                selected[count] = (x, y, tile, attr, i as u8);
                count += 1;
            }
        }

        selected[..count].sort_by(|a, b| a.0.cmp(&b.0).then(a.4.cmp(&b.4)));

        for pixel_x in 0..SCREEN_W {
            let mut sprite_pixel: Option<(u8, u8, u8)> = None;

            for &(x, y, tile_idx, attr, _oam_idx) in &selected[..count] {
                let screen_x = (x as i32) - 8;
                let right = screen_x + 8;
                if pixel_x as i32 >= right || (pixel_x as i32) < screen_x {
                    continue;
                }

                let sprite_y = (y as i32) - 16;
                let current_line = ly - sprite_y;

                let flipped_line = if attr & 0x40 != 0 {
                    (obj_height - 1 - current_line) as u8
                } else {
                    current_line as u8
                };

                let (effective_tile, line_in_tile) = if obj_height == 16 {
                    let base = tile_idx & 0xFE;
                    if flipped_line < 8 {
                        (base, flipped_line)
                    } else {
                        (base | 1, flipped_line - 8)
                    }
                } else {
                    (tile_idx, flipped_line)
                };

                let pixels_into_sprite = pixel_x as i32 - screen_x;
                let pixel_in_tile = if attr & 0x20 != 0 {
                    (pixels_into_sprite) as u8
                } else {
                    (7 - pixels_into_sprite) as u8
                };

                let tile_addr = 0x8000u16 + effective_tile as u16 * 16 + line_in_tile as u16 * 2;
                let vram_idx = tile_addr as usize - 0x8000;
                let byte0 = vram[vram_idx];
                let byte1 = vram[vram_idx + 1];

                let color = ((byte1 >> pixel_in_tile) & 1) << 1 | ((byte0 >> pixel_in_tile) & 1);

                if color != 0 {
                    let palette_bit = (attr >> 4) & 1;
                    let bg_priority = (attr >> 7) & 1;
                    sprite_pixel = Some((color, palette_bit, bg_priority));
                    break;
                }
            }

            if let Some((color, palette_bit, bg_priority)) = sprite_pixel {
                let bg_shade = self.framebuffer[row_offset + pixel_x];
                let sprite_wins = !(bg_priority != 0 && bg_shade != 0 && self.lcdc & 0x01 != 0);

                if sprite_wins {
                    let palette_reg = if palette_bit != 0 {
                        self.obp1
                    } else {
                        self.obp0
                    };
                    let shade = (palette_reg >> (color * 2)) & 0x03;
                    self.framebuffer[row_offset + pixel_x] = shade;
                }
            }
        }
    }

    #[allow(clippy::too_many_lines)]
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

        let bg_tilemap_base = if self.lcdc & 0x08 != 0 {
            0x9C00
        } else {
            0x9800
        };
        let win_tilemap_base = if self.lcdc & 0x40 != 0 {
            0x9C00
        } else {
            0x9800
        };
        let unsigned = self.lcdc & 0x10 != 0;

        let win_left = self.window_left();

        let wx166_bug = win_left.is_some() && self.wx == 166;
        let wx0_special = win_left.is_some() && self.wx == 0;

        let mut window_rendered = false;

        for pixel_x in 0..SCREEN_W {
            if let Some(left) = win_left {
                if (pixel_x as i16) >= left {
                    window_rendered = true;
                    let mut win_x = (pixel_x as i16).wrapping_sub(left) as u16;

                    if wx0_special {
                        win_x = win_x.wrapping_add((self.scx & 7) as u16);
                    }

                    let tile_col = win_x / 8;
                    let tile_row_base = if wx166_bug {
                        self.window_line.wrapping_add(1) as u16
                    } else {
                        self.window_line as u16
                    };
                    let tile_row = tile_row_base / 8;
                    let tilemap_addr = win_tilemap_base + tile_row * 32 + tile_col;
                    let tile_index = vram[tilemap_addr as usize - 0x8000];

                    let tile_addr = if unsigned {
                        0x8000u16 + tile_index as u16 * 16
                    } else {
                        0x9000u16.wrapping_add_signed(tile_index as i8 as i16 * 16)
                    };

                    let line_in_tile = (tile_row_base as usize) & 7;
                    let pixel_in_tile = 7usize.wrapping_sub((win_x & 7) as usize);
                    let byte0_addr = tile_addr as usize - 0x8000 + line_in_tile * 2;
                    let byte0 = vram[byte0_addr];
                    let byte1 = vram[byte0_addr + 1];

                    let color_idx =
                        ((byte1 >> pixel_in_tile) & 1) << 1 | ((byte0 >> pixel_in_tile) & 1);
                    let shade = (self.bgp >> (color_idx * 2)) & 0x03;
                    self.framebuffer[row_offset + pixel_x] = shade;
                    continue;
                }
            }

            let bg_x = (pixel_x as u16).wrapping_add(self.scx as u16) & 0xFF;
            let bg_y = (ly as u16).wrapping_add(self.scy as u16) & 0xFF;

            let tile_col = bg_x / 8;
            let tile_row = bg_y / 8;

            let tilemap_addr = bg_tilemap_base + tile_row * 32 + tile_col;
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

        if window_rendered {
            self.window_line = self.window_line.wrapping_add(1);
        }
    }

    fn window_left(&self) -> Option<i16> {
        if (self.lcdc & 0x20) == 0 || !self.window_y_condition {
            return None;
        }
        if self.wx > 166 {
            return None;
        }
        if self.wx == 0 || self.wx == 166 {
            return Some(0);
        }
        Some((self.wx as i16) - 7)
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
