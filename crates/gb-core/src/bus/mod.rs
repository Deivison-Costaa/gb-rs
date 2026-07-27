//! spec: `docs/reference/01-memory-map.md`. Bus é struct, não trait — vtable
//! no caminho mais quente do emulador não compra nada (ver docs/iterations/0010).

pub(crate) mod boot;

use crate::apu::Apu;
use crate::cart::{Cartridge, OPEN_BUS};
use crate::joypad::{Joypad, Key};
use crate::ppu::Ppu;
use crate::serial::Serial;

const P1_ADDR: u16 = 0xFF00;

const DIV_ADDR: u16 = 0xFF04;
const TIMA_ADDR: u16 = 0xFF05;
const TAC_ADDR: u16 = 0xFF07;

const SB_ADDR: u16 = 0xFF01;

const SC_ADDR: u16 = 0xFF02;

const TIMA_IDX: usize = 0x05;
const TMA_IDX: usize = 0x06;
const TAC_IDX: usize = 0x07;
const IF_IDX: usize = 0x0F;

const WRAM_LEN: usize = 8 * 1024;

const VRAM_LEN: usize = 8 * 1024;

const OAM_LEN: usize = 40 * 4;

// Só os 13 bits baixos do endereço chegam à WRAM (§ Echo RAM descrevendo a fiação).
const WRAM_ADDRESS_MASK: usize = 0x1FFF;

// $FF80–$FFFE: 127 bytes, não 128. $FFFF é o IE.
const HRAM_LEN: usize = 0x7F;

const HRAM_BASE: usize = 0xFF80;

// $FEA0–$FEFF: $00 no DMG sem bloqueio de OAM (sem PPU até M3). $FF só com OAM bloqueada.
const NOT_USABLE_READ: u8 = 0x00;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Region {
    CartridgeRom,
    VideoRam,
    ExternalRam,
    WorkRam,
    EchoRam,
    ObjectAttributeMemory,
    NotUsable,
    IoRegisters,
    HighRam,
    InterruptEnable,
}

impl Region {
    #[must_use]
    pub const fn of(addr: u16) -> Self {
        match addr {
            0x0000..=0x7FFF => Self::CartridgeRom,
            0x8000..=0x9FFF => Self::VideoRam,
            0xA000..=0xBFFF => Self::ExternalRam,
            0xC000..=0xDFFF => Self::WorkRam,
            0xE000..=0xFDFF => Self::EchoRam,
            0xFE00..=0xFE9F => Self::ObjectAttributeMemory,
            0xFEA0..=0xFEFF => Self::NotUsable,
            0xFF00..=0xFF7F => Self::IoRegisters,
            0xFF80..=0xFFFE => Self::HighRam,
            0xFFFF => Self::InterruptEnable,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimerOverflow {
    Idle,
    Overflowed,
    Reloading,
}

pub struct Bus {
    cartridge: Box<dyn Cartridge>,
    vram: [u8; VRAM_LEN],
    oam: [u8; OAM_LEN],
    wram: [u8; WRAM_LEN],
    hram: [u8; HRAM_LEN],
    io: [u8; boot::IO_LEN],
    ie: u8,
    serial: Serial,
    joypad: Joypad,
    sys_counter: u16,
    prev_and_result: bool,
    tima_overflow: TimerOverflow,
    ppu: Ppu,
    apu: Apu,
}

impl Bus {
    #[must_use]
    pub fn new(cartridge: Box<dyn Cartridge>) -> Self {
        Self {
            cartridge,
            vram: [0x00; VRAM_LEN],
            oam: [0x00; OAM_LEN],
            wram: [0x00; WRAM_LEN],
            hram: [0x00; HRAM_LEN],
            io: boot::IO,
            ie: boot::INTERRUPT_ENABLE,
            serial: Serial::new(),
            joypad: Joypad::new(),
            sys_counter: 0xAB00,
            prev_and_result: false,
            tima_overflow: TimerOverflow::Idle,
            ppu: Ppu::new(),
            apu: Apu::new(),
        }
    }

    #[must_use]
    pub fn read(&self, addr: u16) -> u8 {
        match Region::of(addr) {
            Region::CartridgeRom | Region::ExternalRam => self.cartridge.read(addr),
            Region::WorkRam | Region::EchoRam => self.wram[wram_index(addr)],
            Region::HighRam => self.hram[hram_index(addr)],
            Region::NotUsable => NOT_USABLE_READ,
            Region::IoRegisters => match addr {
                P1_ADDR => self.joypad.read(),
                DIV_ADDR => (self.sys_counter >> 8) as u8,
                SB_ADDR | SC_ADDR => self.serial.read(addr),
                0xFF10..=0xFF26 => self.apu.read(addr),
                0xFF40..=0xFF4B => self.ppu.read(addr),
                _ => {
                    let index = io_index(addr);
                    if boot::IO_HAS_OWNER[index] {
                        self.io[index]
                    } else {
                        OPEN_BUS
                    }
                }
            },
            Region::InterruptEnable => self.ie,
            Region::VideoRam => {
                if self.ppu.is_vram_accessible() {
                    self.vram[vram_index(addr)]
                } else {
                    OPEN_BUS
                }
            }
            Region::ObjectAttributeMemory => {
                if self.ppu.is_oam_accessible() {
                    self.oam[oam_index(addr)]
                } else {
                    OPEN_BUS
                }
            }
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match Region::of(addr) {
            Region::CartridgeRom | Region::ExternalRam => self.cartridge.write(addr, value),
            Region::WorkRam | Region::EchoRam => self.wram[wram_index(addr)] = value,
            Region::HighRam => self.hram[hram_index(addr)] = value,
            Region::IoRegisters => match addr {
                P1_ADDR => self.joypad.write(value),
                DIV_ADDR => {
                    let old_and = self.and_result();
                    self.sys_counter = 0;
                    let new_and = self.and_result();
                    if old_and && !new_and {
                        self.increment_tima();
                    }
                    self.prev_and_result = new_and;
                }
                TIMA_ADDR => {
                    let index = io_index(addr);
                    match self.tima_overflow {
                        TimerOverflow::Overflowed => {
                            self.io[index] = value;
                            self.tima_overflow = TimerOverflow::Idle;
                        }
                        TimerOverflow::Reloading => {}
                        TimerOverflow::Idle => {
                            self.io[index] = value;
                        }
                    }
                }
                TAC_ADDR => {
                    let old_and = self.prev_and_result;
                    self.io[TAC_IDX] = value;
                    let new_and = self.and_result();
                    if old_and && !new_and {
                        self.increment_tima();
                    }
                    self.prev_and_result = new_and;
                }
                SB_ADDR | SC_ADDR => self.serial.write(addr, value),
                0xFF10..=0xFF26 => self.apu.write(addr, value),
                0xFF40..=0xFF4B => self.ppu.write(addr, value),
                _ => {
                    let index = io_index(addr);
                    if boot::IO_HAS_OWNER[index] {
                        self.io[index] = value;
                    }
                }
            },
            Region::InterruptEnable => self.ie = value,
            Region::VideoRam => {
                if self.ppu.is_vram_accessible() {
                    self.vram[vram_index(addr)] = value;
                }
            }
            Region::NotUsable => {}
            Region::ObjectAttributeMemory => {
                if self.ppu.is_oam_accessible() {
                    self.oam[oam_index(addr)] = value;
                }
            }
        }
    }

    #[must_use]
    pub fn take_serial_output(&mut self) -> Vec<u8> {
        self.serial.take_output()
    }

    pub fn tick_ppu(&mut self) {
        let signals = self.ppu.tick();
        if signals.begin_mode3 {
            self.ppu.render_scanline(&self.vram, &self.oam);
        }
        if signals.vblank_interrupt {
            self.io[IF_IDX] |= 0x01;
        }
        if signals.stat_interrupt {
            self.io[IF_IDX] |= 0x02;
        }
    }

    #[must_use]
    pub fn cartridge_ram(&self) -> Option<&[u8]> {
        self.cartridge.ram_data()
    }

    #[must_use]
    pub fn framebuffer(&self) -> &[u8; 160 * 144] {
        self.ppu.framebuffer()
    }

    pub fn key_down(&mut self, key: Key) {
        self.joypad.key_down(key);
    }

    pub fn key_up(&mut self, key: Key) {
        self.joypad.key_up(key);
    }

    pub fn tick_joypad_interrupt(&mut self) {
        if self.joypad.interrupt {
            self.joypad.interrupt = false;
            self.io[IF_IDX] |= 0x10;
        }
    }

    pub fn tick_timer(&mut self) {
        if self.tima_overflow == TimerOverflow::Reloading {
            self.tima_overflow = TimerOverflow::Idle;
        }

        if self.tima_overflow == TimerOverflow::Overflowed {
            self.io[TIMA_IDX] = self.io[TMA_IDX];
            self.io[IF_IDX] |= 0x04;
            self.tima_overflow = TimerOverflow::Reloading;
        }

        let old_and = self.prev_and_result;
        self.sys_counter = self.sys_counter.wrapping_add(4);
        let new_and = self.and_result();

        if old_and && !new_and {
            self.increment_tima();
        }

        self.prev_and_result = new_and;
    }

    pub fn tick_apu(&mut self) {
        self.apu.tick();
    }

    #[must_use]
    pub fn apu_frame_sequencer_step(&self) -> u8 {
        self.apu.frame_sequencer_step()
    }

    #[must_use]
    pub fn ch1_enabled(&self) -> bool {
        self.apu.ch1_enabled()
    }

    #[must_use]
    pub fn ch1_sweep_pace(&self) -> u8 {
        self.apu.ch1_sweep_pace()
    }

    #[must_use]
    pub fn ch1_sweep_direction(&self) -> u8 {
        self.apu.ch1_sweep_direction()
    }

    #[must_use]
    pub fn ch1_sweep_step(&self) -> u8 {
        self.apu.ch1_sweep_step()
    }

    #[must_use]
    pub fn ch1_sweep_shadow(&self) -> u16 {
        self.apu.ch1_sweep_shadow()
    }

    #[must_use]
    pub fn ch1_sweep_enabled(&self) -> bool {
        self.apu.ch1_sweep_enabled()
    }

    #[must_use]
    pub fn ch1_sweep_timer(&self) -> u8 {
        self.apu.ch1_sweep_timer()
    }

    #[must_use]
    pub fn ch1_duty_pattern(&self) -> u8 {
        self.apu.ch1_duty_pattern()
    }

    #[must_use]
    pub fn ch1_initial_volume(&self) -> u8 {
        self.apu.ch1_initial_volume()
    }

    #[must_use]
    pub fn ch1_envelope_pace(&self) -> u8 {
        self.apu.ch1_envelope_pace()
    }

    #[must_use]
    pub fn ch1_dac_enabled(&self) -> bool {
        self.apu.ch1_dac_enabled()
    }

    #[must_use]
    pub fn ch1_period(&self) -> u16 {
        self.apu.ch1_period()
    }

    #[must_use]
    pub fn ch1_frequency_timer(&self) -> u16 {
        self.apu.ch1_frequency_timer()
    }

    #[must_use]
    pub fn ch1_duty_step(&self) -> u8 {
        self.apu.ch1_duty_step()
    }

    #[must_use]
    pub fn ch1_envelope_volume(&self) -> u8 {
        self.apu.ch1_envelope_volume()
    }

    #[must_use]
    pub fn ch2_enabled(&self) -> bool {
        self.apu.ch2_enabled()
    }

    #[must_use]
    pub fn ch2_duty_pattern(&self) -> u8 {
        self.apu.ch2_duty_pattern()
    }

    #[must_use]
    pub fn ch2_initial_volume(&self) -> u8 {
        self.apu.ch2_initial_volume()
    }

    #[must_use]
    pub fn ch2_envelope_pace(&self) -> u8 {
        self.apu.ch2_envelope_pace()
    }

    #[must_use]
    pub fn ch2_dac_enabled(&self) -> bool {
        self.apu.ch2_dac_enabled()
    }

    #[must_use]
    pub fn ch2_period(&self) -> u16 {
        self.apu.ch2_period()
    }

    #[must_use]
    pub fn ch2_frequency_timer(&self) -> u16 {
        self.apu.ch2_frequency_timer()
    }

    #[must_use]
    pub fn ch2_duty_step(&self) -> u8 {
        self.apu.ch2_duty_step()
    }

    #[must_use]
    pub fn ch2_envelope_volume(&self) -> u8 {
        self.apu.ch2_envelope_volume()
    }

    fn and_result(&self) -> bool {
        let tac = self.io[TAC_IDX];
        let enable = (tac & 0x04) != 0;
        if !enable {
            return false;
        }
        let bit = clock_bit(tac & 0x03);
        ((self.sys_counter >> bit) & 1) != 0
    }

    fn increment_tima(&mut self) {
        let (tima, overflowed) = self.io[TIMA_IDX].overflowing_add(1);
        self.io[TIMA_IDX] = tima;
        if overflowed {
            self.tima_overflow = TimerOverflow::Overflowed;
        }
    }
}

const fn clock_bit(select: u8) -> u32 {
    match select {
        0 => 9,
        1 => 3,
        2 => 5,
        3 => 7,
        _ => 9,
    }
}

const fn wram_index(addr: u16) -> usize {
    (addr as usize) & WRAM_ADDRESS_MASK
}

const fn vram_index(addr: u16) -> usize {
    (addr as usize) - 0x8000
}

const fn oam_index(addr: u16) -> usize {
    (addr as usize) - 0xFE00
}

const fn hram_index(addr: u16) -> usize {
    (addr as usize) - HRAM_BASE
}

const fn io_index(addr: u16) -> usize {
    (addr as usize) - boot::IO_BASE
}

impl std::fmt::Debug for Bus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bus")
            .field("wram_len", &self.wram.len())
            .field("hram_len", &self.hram.len())
            .finish_non_exhaustive()
    }
}
