//! spec: docs/reference/07-apu.md. ROADMAP 6.1 — frame sequencer 512 Hz.

use crate::cart::OPEN_BUS;

const NR10_ADDR: u16 = 0xFF10;
const NR11_ADDR: u16 = 0xFF11;
const NR12_ADDR: u16 = 0xFF12;
const NR13_ADDR: u16 = 0xFF13;
const NR14_ADDR: u16 = 0xFF14;

const NR21_ADDR: u16 = 0xFF16;
const NR22_ADDR: u16 = 0xFF17;
const NR23_ADDR: u16 = 0xFF18;
const NR24_ADDR: u16 = 0xFF19;

const NR30_ADDR: u16 = 0xFF1A;
const NR31_ADDR: u16 = 0xFF1B;
const NR32_ADDR: u16 = 0xFF1C;
const NR33_ADDR: u16 = 0xFF1D;
const NR34_ADDR: u16 = 0xFF1E;

const NR41_ADDR: u16 = 0xFF20;
const NR42_ADDR: u16 = 0xFF21;
const NR43_ADDR: u16 = 0xFF22;
const NR44_ADDR: u16 = 0xFF23;

const NR50_ADDR: u16 = 0xFF24;
const NR51_ADDR: u16 = 0xFF25;
const NR52_ADDR: u16 = 0xFF26;

const WAVE_RAM_BASE: u16 = 0xFF30;
const WAVE_RAM_END: u16 = 0xFF3F;

const T_CYCLES_PER_FRAME_SEQUENCER_TICK: u32 = 8192;

const NR52_POWER_BIT: u8 = 0x80;

const NRX4_TRIGGER_BIT: u8 = 0x80;

const FREQ_MAX: u16 = 2048;

#[derive(Debug, Clone)]
struct Channel2 {
    enabled: bool,
    freq_timer: u16,
    duty_step: u8,
    envelope_volume: u8,
    envelope_timer: u8,
}

impl Channel2 {
    const fn new() -> Self {
        Self {
            enabled: false,
            freq_timer: 0,
            duty_step: 0,
            envelope_volume: 0,
            envelope_timer: 0,
        }
    }

    fn trigger(&mut self, nr22: u8, nr23: u8, nr24: u8) {
        self.enabled = true;
        self.freq_timer = period(nr23, nr24);
        self.envelope_volume = nr22 >> 4;
        let pace = nr22 & 0x07;
        self.envelope_timer = if pace == 0 { 8 } else { pace };
    }

    fn tick_freq(&mut self, nr23: u8, nr24: u8) {
        self.freq_timer = self.freq_timer.wrapping_add(4);
        if self.freq_timer >= FREQ_MAX {
            self.freq_timer = period(nr23, nr24);
            self.duty_step = (self.duty_step + 1) & 0x07;
        }
    }

    fn tick_envelope(&mut self, nr22: u8) {
        let pace = nr22 & 0x07;
        if pace == 0 {
            return;
        }

        if self.envelope_timer > 0 {
            self.envelope_timer -= 1;
        }

        if self.envelope_timer == 0 {
            self.envelope_timer = pace;

            if (nr22 >> 3) & 1 == 0 {
                if self.envelope_volume > 0 {
                    self.envelope_volume -= 1;
                }
            } else if self.envelope_volume < 15 {
                self.envelope_volume += 1;
            }
        }
    }
}

const fn period(nr23: u8, nr24: u8) -> u16 {
    u16::from_le_bytes([nr23, nr24 & 0x07])
}

pub(crate) struct Apu {
    nr10: u8,
    nr11: u8,
    nr12: u8,
    nr13: u8,
    nr14: u8,
    nr21: u8,
    nr22: u8,
    nr23: u8,
    nr24: u8,
    nr30: u8,
    nr31: u8,
    nr32: u8,
    nr33: u8,
    nr34: u8,
    nr41: u8,
    nr42: u8,
    nr43: u8,
    nr44: u8,
    nr50: u8,
    nr51: u8,
    nr52: u8,
    wave_ram: [u8; 16],
    div_apu: u32,
    frame_sequencer_step: u8,
    prev_frame_sequencer_step: u8,
    ch2: Channel2,
}

impl Apu {
    pub(crate) fn new() -> Self {
        Self {
            nr10: 0x80,
            nr11: 0xBF,
            nr12: 0xF3,
            nr13: 0xFF,
            nr14: 0xBF,
            nr21: 0x3F,
            nr22: 0x00,
            nr23: 0xFF,
            nr24: 0xBF,
            nr30: 0x7F,
            nr31: 0xFF,
            nr32: 0x9F,
            nr33: 0xFF,
            nr34: 0xBF,
            nr41: 0xFF,
            nr42: 0x00,
            nr43: 0x00,
            nr44: 0xBF,
            nr50: 0x77,
            nr51: 0xF3,
            nr52: 0xF1,
            wave_ram: [0x00; 16],
            div_apu: 0,
            frame_sequencer_step: 0,
            prev_frame_sequencer_step: 0,
            ch2: Channel2::new(),
        }
    }

    pub(crate) fn read(&self, addr: u16) -> u8 {
        match addr {
            NR10_ADDR => self.nr10,
            NR11_ADDR => self.nr11,
            NR12_ADDR => self.nr12,
            NR13_ADDR => self.nr13,
            NR14_ADDR => self.nr14,
            NR21_ADDR => self.nr21,
            NR22_ADDR => self.nr22,
            NR23_ADDR => self.nr23,
            NR24_ADDR => self.nr24,
            NR30_ADDR => self.nr30,
            NR31_ADDR => self.nr31,
            NR32_ADDR => self.nr32,
            NR33_ADDR => self.nr33,
            NR34_ADDR => self.nr34,
            NR41_ADDR => self.nr41,
            NR42_ADDR => self.nr42,
            NR43_ADDR => self.nr43,
            NR44_ADDR => self.nr44,
            NR50_ADDR => self.nr50,
            NR51_ADDR => self.nr51,
            NR52_ADDR => self.nr52,
            WAVE_RAM_BASE..=WAVE_RAM_END => self.wave_ram[(addr - WAVE_RAM_BASE) as usize],
            _ => OPEN_BUS,
        }
    }

    pub(crate) fn write(&mut self, addr: u16, value: u8) {
        match addr {
            NR10_ADDR => self.nr10 = value,
            NR11_ADDR => self.nr11 = value,
            NR12_ADDR => self.nr12 = value,
            NR13_ADDR => self.nr13 = value,
            NR14_ADDR => self.nr14 = value,
            NR21_ADDR => self.nr21 = value,
            NR22_ADDR => self.nr22 = value,
            NR23_ADDR => self.nr23 = value,
            NR24_ADDR => {
                self.nr24 = value;
                if value & NRX4_TRIGGER_BIT != 0 {
                    self.ch2.trigger(self.nr22, self.nr23, self.nr24);
                }
            }
            NR30_ADDR => self.nr30 = value,
            NR31_ADDR => self.nr31 = value,
            NR32_ADDR => self.nr32 = value,
            NR33_ADDR => self.nr33 = value,
            NR34_ADDR => self.nr34 = value,
            NR41_ADDR => self.nr41 = value,
            NR42_ADDR => self.nr42 = value,
            NR43_ADDR => self.nr43 = value,
            NR44_ADDR => self.nr44 = value,
            NR50_ADDR => self.nr50 = value,
            NR51_ADDR => self.nr51 = value,
            NR52_ADDR => {
                self.nr52 = (self.nr52 & !NR52_POWER_BIT) | (value & NR52_POWER_BIT);
            }
            WAVE_RAM_BASE..=WAVE_RAM_END => {
                self.wave_ram[(addr - WAVE_RAM_BASE) as usize] = value;
            }
            _ => {}
        }
    }

    pub(crate) fn tick(&mut self) {
        self.div_apu = self.div_apu.wrapping_add(4);

        if self.nr52 & NR52_POWER_BIT != 0 && self.ch2.enabled {
            self.ch2.tick_freq(self.nr23, self.nr24);
        }

        if self.div_apu >= T_CYCLES_PER_FRAME_SEQUENCER_TICK {
            self.div_apu -= T_CYCLES_PER_FRAME_SEQUENCER_TICK;
            self.prev_frame_sequencer_step = self.frame_sequencer_step;
            self.frame_sequencer_step = (self.frame_sequencer_step + 1) & 0x07;

            if self.nr52 & NR52_POWER_BIT != 0 && self.ch2.enabled {
                let step = self.frame_sequencer_step;
                if step == 2 || step == 6 {
                    self.ch2.tick_envelope(self.nr22);
                }
            }
        }
    }

    pub(crate) const fn frame_sequencer_step(&self) -> u8 {
        self.frame_sequencer_step
    }

    pub(crate) const fn ch2_enabled(&self) -> bool {
        self.ch2.enabled
    }

    pub(crate) const fn ch2_duty_pattern(&self) -> u8 {
        self.nr21 >> 6
    }

    pub(crate) const fn ch2_initial_volume(&self) -> u8 {
        self.nr22 >> 4
    }

    pub(crate) const fn ch2_envelope_pace(&self) -> u8 {
        self.nr22 & 0x07
    }

    pub(crate) const fn ch2_dac_enabled(&self) -> bool {
        self.nr22 & 0xF8 != 0
    }

    pub(crate) const fn ch2_period(&self) -> u16 {
        period(self.nr23, self.nr24)
    }

    pub(crate) const fn ch2_frequency_timer(&self) -> u16 {
        self.ch2.freq_timer
    }

    pub(crate) const fn ch2_duty_step(&self) -> u8 {
        self.ch2.duty_step
    }

    pub(crate) const fn ch2_envelope_volume(&self) -> u8 {
        self.ch2.envelope_volume
    }
}
