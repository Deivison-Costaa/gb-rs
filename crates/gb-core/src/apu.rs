//! spec: docs/reference/07-apu.md. ROADMAP 6.5 — CH4 noise.

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

const M_CYCLES_PER_SECOND: u32 = 1_048_576;
const SAMPLE_RATE: u32 = 48_000;
const RING_BUFFER_SIZE: usize = 4096;

const FREQ_MAX: u16 = 2048;

const DUTY_WAVEFORMS: [u8; 4] = [0b00000001, 0b10000001, 0b10000111, 0b01111110];

#[derive(Debug, Clone)]
struct PulseChannel {
    enabled: bool,
    freq_timer: u16,
    duty_step: u8,
    envelope_volume: u8,
    envelope_timer: u8,
}

impl PulseChannel {
    const fn new() -> Self {
        Self {
            enabled: false,
            freq_timer: 0,
            duty_step: 0,
            envelope_volume: 0,
            envelope_timer: 0,
        }
    }

    fn trigger(&mut self, nrx2: u8, nrx3: u8, nrx4: u8) {
        self.enabled = true;
        self.freq_timer = period(nrx3, nrx4);
        self.envelope_volume = nrx2 >> 4;
        let pace = nrx2 & 0x07;
        self.envelope_timer = if pace == 0 { 8 } else { pace };
    }

    fn tick_freq(&mut self, nrx3: u8, nrx4: u8) {
        self.freq_timer = self.freq_timer.wrapping_add(4);
        if self.freq_timer >= FREQ_MAX {
            self.freq_timer = period(nrx3, nrx4);
            self.duty_step = (self.duty_step + 1) & 0x07;
        }
    }

    fn tick_envelope(&mut self, nrx2: u8) {
        let pace = nrx2 & 0x07;
        if pace == 0 {
            return;
        }

        if self.envelope_timer > 0 {
            self.envelope_timer -= 1;
        }

        if self.envelope_timer == 0 {
            self.envelope_timer = pace;

            if (nrx2 >> 3) & 1 == 0 {
                if self.envelope_volume > 0 {
                    self.envelope_volume -= 1;
                }
            } else if self.envelope_volume < 15 {
                self.envelope_volume += 1;
            }
        }
    }

    fn digital_output(&self, duty_pattern: u8) -> u8 {
        if !self.enabled {
            return 0;
        }
        let waveform = DUTY_WAVEFORMS[(duty_pattern & 0x03) as usize];
        if (waveform >> self.duty_step) & 1 != 0 {
            self.envelope_volume
        } else {
            0
        }
    }
}

#[derive(Debug, Clone)]
struct Channel1 {
    pulse: PulseChannel,
    sweep_shadow: u16,
    sweep_timer: u8,
    sweep_enabled: bool,
}

impl Channel1 {
    const fn new() -> Self {
        Self {
            pulse: PulseChannel::new(),
            sweep_shadow: 0,
            sweep_timer: 0,
            sweep_enabled: false,
        }
    }
}

#[derive(Debug, Clone)]
struct Channel3 {
    enabled: bool,
    freq_timer: u16,
    sample_index: u8,
    last_sample_buffer: u8,
}

impl Channel3 {
    const fn new() -> Self {
        Self {
            enabled: false,
            freq_timer: 0,
            sample_index: 1,
            last_sample_buffer: 0,
        }
    }

    fn digital_output(&self, nr32: u8) -> u8 {
        if !self.enabled {
            return 0;
        }
        let level = (nr32 >> 5) & 0x03;
        match level {
            0 => 0,
            1 => self.last_sample_buffer,
            2 => self.last_sample_buffer >> 1,
            _ => self.last_sample_buffer >> 2,
        }
    }
}

const fn period(nrx3: u8, nrx4: u8) -> u16 {
    u16::from_le_bytes([nrx3, nrx4 & 0x07])
}

fn sweep_calculate(nr10: u8, shadow: u16) -> u16 {
    let direction = (nr10 >> 3) & 1;
    let step = nr10 & 0x07;
    let shift = shadow >> step;
    if direction == 0 {
        shadow.wrapping_add(shift)
    } else {
        shadow.wrapping_sub(shift)
    }
}

fn sweep_write_back(new: u16, nr13: &mut u8, nr14: &mut u8) {
    *nr13 = (new & 0xFF) as u8;
    *nr14 = (*nr14 & 0xF8) | ((new >> 8) & 0x07) as u8;
}

#[derive(Debug, Clone)]
struct Channel4 {
    enabled: bool,
    lfsr: u16,
    freq_timer: u16,
    envelope_volume: u8,
    envelope_timer: u8,
}

impl Channel4 {
    const fn new() -> Self {
        Self {
            enabled: false,
            lfsr: 0x0000,
            freq_timer: 0,
            envelope_volume: 0,
            envelope_timer: 0,
        }
    }

    fn trigger(&mut self, nr42: u8) {
        self.enabled = true;
        self.lfsr = 0x0000;
        self.freq_timer = 0;
        self.envelope_volume = nr42 >> 4;
        let pace = nr42 & 0x07;
        self.envelope_timer = if pace == 0 { 8 } else { pace };
    }

    fn tick_envelope(&mut self, nr42: u8) {
        let pace = nr42 & 0x07;
        if pace == 0 {
            return;
        }

        if self.envelope_timer > 0 {
            self.envelope_timer -= 1;
        }

        if self.envelope_timer == 0 {
            self.envelope_timer = pace;

            if (nr42 >> 3) & 1 == 0 {
                if self.envelope_volume > 0 {
                    self.envelope_volume -= 1;
                }
            } else if self.envelope_volume < 15 {
                self.envelope_volume += 1;
            }
        }
    }

    fn tick_freq(&mut self, nr43: u8) {
        let threshold = noise_threshold(nr43);
        if threshold == u16::MAX {
            return;
        }
        self.freq_timer = self.freq_timer.wrapping_add(4);
        while self.freq_timer >= threshold {
            self.freq_timer -= threshold;
            let bit0 = self.lfsr & 1;
            let bit1 = (self.lfsr >> 1) & 1;
            let feedback = if bit0 == bit1 { 1 << 15 } else { 0 };
            let width_7bit = (nr43 >> 3) & 1 != 0;
            self.lfsr = (self.lfsr & 0x7FFF) | feedback;
            if width_7bit {
                self.lfsr = (self.lfsr & 0xFF7F) | (feedback >> 8);
            }
            self.lfsr >>= 1;
        }
    }

    fn digital_output(&self) -> u8 {
        if !self.enabled {
            return 0;
        }
        if self.lfsr & 1 != 0 {
            self.envelope_volume
        } else {
            0
        }
    }
}

const fn noise_threshold(nr43: u8) -> u16 {
    let shift = nr43 >> 4;
    if shift >= 14 {
        return u16::MAX;
    }
    let divider = (nr43 & 0x07) as u16;
    if divider == 0 {
        2u16.wrapping_shl(shift as u32)
    } else {
        (4 * divider).wrapping_shl(shift as u32)
    }
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
    ch1: Channel1,
    ch2: PulseChannel,
    ch3: Channel3,
    ch4: Channel4,
    sample_accum_left: f32,
    sample_accum_right: f32,
    sample_accum_count: u32,
    sample_phase: u32,
    sample_buffer: Box<[f32]>,
    sample_buffer_read: usize,
    sample_buffer_write: usize,
    sample_buffer_count: usize,
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
            ch1: Channel1::new(),
            ch2: PulseChannel::new(),
            ch3: Channel3::new(),
            ch4: Channel4::new(),
            sample_accum_left: 0.0,
            sample_accum_right: 0.0,
            sample_accum_count: 0,
            sample_phase: 0,
            sample_buffer: vec![0.0f32; RING_BUFFER_SIZE * 2].into_boxed_slice(),
            sample_buffer_read: 0,
            sample_buffer_write: 0,
            sample_buffer_count: 0,
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
            NR52_ADDR => {
                let status = if self.ch1.pulse.enabled { 0x01 } else { 0 }
                    | if self.ch2.enabled { 0x02 } else { 0 }
                    | if self.ch3.enabled { 0x04 } else { 0 }
                    | if self.ch4.enabled { 0x08 } else { 0 };
                (self.nr52 & 0xF0) | status
            }
            WAVE_RAM_BASE..=WAVE_RAM_END => {
                if self.ch3.enabled {
                    OPEN_BUS
                } else {
                    self.wave_ram[(addr - WAVE_RAM_BASE) as usize]
                }
            }
            _ => OPEN_BUS,
        }
    }

    #[allow(clippy::too_many_lines)]
    pub(crate) fn write(&mut self, addr: u16, value: u8) {
        match addr {
            NR10_ADDR => self.nr10 = value,
            NR11_ADDR => self.nr11 = value,
            NR12_ADDR => self.nr12 = value,
            NR13_ADDR => self.nr13 = value,
            NR14_ADDR => {
                self.nr14 = value;
                if value & NRX4_TRIGGER_BIT != 0 {
                    self.ch1.pulse.trigger(self.nr12, self.nr13, self.nr14);
                    self.trigger_ch1_sweep();
                }
            }
            NR21_ADDR => self.nr21 = value,
            NR22_ADDR => self.nr22 = value,
            NR23_ADDR => self.nr23 = value,
            NR24_ADDR => {
                self.nr24 = value;
                if value & NRX4_TRIGGER_BIT != 0 {
                    self.ch2.trigger(self.nr22, self.nr23, self.nr24);
                }
            }
            NR30_ADDR => {
                self.nr30 = value;
                if value & 0x80 == 0 {
                    self.ch3.enabled = false;
                }
            }
            NR31_ADDR => self.nr31 = value,
            NR32_ADDR => self.nr32 = value,
            NR33_ADDR => self.nr33 = value,
            NR34_ADDR => {
                self.nr34 = value;
                if value & NRX4_TRIGGER_BIT != 0 {
                    self.trigger_ch3();
                }
            }
            NR41_ADDR => self.nr41 = value,
            NR42_ADDR => self.nr42 = value,
            NR43_ADDR => self.nr43 = value,
            NR44_ADDR => {
                self.nr44 = value;
                if value & NRX4_TRIGGER_BIT != 0 {
                    self.ch4.trigger(self.nr42);
                }
            }
            NR50_ADDR => self.nr50 = value,
            NR51_ADDR => self.nr51 = value,
            NR52_ADDR => {
                let was_powered = self.nr52 & NR52_POWER_BIT != 0;
                self.nr52 = (self.nr52 & !NR52_POWER_BIT) | (value & NR52_POWER_BIT);
                if was_powered && value & NR52_POWER_BIT == 0 {
                    self.power_off();
                }
            }
            WAVE_RAM_BASE..=WAVE_RAM_END if !self.ch3.enabled => {
                self.wave_ram[(addr - WAVE_RAM_BASE) as usize] = value;
            }
            _ => {}
        }
    }

    fn power_off(&mut self) {
        self.nr10 = 0x80;
        self.nr11 = 0xBF;
        self.nr12 = 0xF3;
        self.nr13 = 0xFF;
        self.nr14 = 0xBF;
        self.nr21 = 0x3F;
        self.nr22 = 0x00;
        self.nr23 = 0xFF;
        self.nr24 = 0xBF;
        self.nr30 = 0x7F;
        self.nr31 = 0xFF;
        self.nr32 = 0x9F;
        self.nr33 = 0xFF;
        self.nr34 = 0xBF;
        self.nr41 = 0xFF;
        self.nr42 = 0x00;
        self.nr43 = 0x00;
        self.nr44 = 0xBF;
        self.nr50 = 0x77;
        self.nr51 = 0xF3;
        self.ch1 = Channel1::new();
        self.ch2 = PulseChannel::new();
        self.ch3 = Channel3::new();
        self.ch4 = Channel4::new();
        self.sample_accum_left = 0.0;
        self.sample_accum_right = 0.0;
        self.sample_accum_count = 0;
    }

    fn trigger_ch1_sweep(&mut self) {
        let pace = (self.nr10 >> 4) & 0x07;
        let step = self.nr10 & 0x07;
        self.ch1.sweep_shadow = period(self.nr13, self.nr14);
        self.ch1.sweep_timer = if pace == 0 { 8 } else { pace };
        self.ch1.sweep_enabled = pace != 0 || step != 0;

        if step != 0 {
            let new_period = sweep_calculate(self.nr10, self.ch1.sweep_shadow);
            if new_period > 2047 {
                self.ch1.pulse.enabled = false;
                return;
            }
            self.ch1.sweep_shadow = new_period;
            sweep_write_back(new_period, &mut self.nr13, &mut self.nr14);
        }
    }

    fn tick_ch1_sweep(&mut self) {
        let pace = (self.nr10 >> 4) & 0x07;

        if self.ch1.sweep_timer > 0 {
            self.ch1.sweep_timer -= 1;
        }

        if self.ch1.sweep_timer == 0 {
            self.ch1.sweep_timer = if pace == 0 { 8 } else { pace };

            if self.ch1.sweep_enabled && pace != 0 {
                let new_period = sweep_calculate(self.nr10, self.ch1.sweep_shadow);
                if new_period > 2047 {
                    self.ch1.pulse.enabled = false;
                    return;
                }

                let step = self.nr10 & 0x07;
                if step != 0 {
                    self.ch1.sweep_shadow = new_period;
                    sweep_write_back(new_period, &mut self.nr13, &mut self.nr14);

                    let second = sweep_calculate(self.nr10, new_period);
                    if second > 2047 {
                        self.ch1.pulse.enabled = false;
                    }
                }
            }
        }
    }

    fn trigger_ch3(&mut self) {
        if self.nr30 & 0x80 == 0 {
            return;
        }
        self.ch3.enabled = true;
        self.ch3.freq_timer = period(self.nr33, self.nr34);
        self.ch3.sample_index = 1;
    }

    fn tick_ch3_freq(&mut self) {
        self.ch3.freq_timer = self.ch3.freq_timer.wrapping_add(8);
        if self.ch3.freq_timer >= FREQ_MAX {
            self.ch3.freq_timer = period(self.nr33, self.nr34);
            let byte_idx = (self.ch3.sample_index as usize) >> 1;
            let nibble = if self.ch3.sample_index & 1 == 0 {
                self.wave_ram[byte_idx] >> 4
            } else {
                self.wave_ram[byte_idx] & 0x0F
            };
            self.ch3.last_sample_buffer = nibble;
            self.ch3.sample_index = (self.ch3.sample_index + 1) & 0x1F;
        }
    }

    pub(crate) fn tick(&mut self) {
        self.div_apu = self.div_apu.wrapping_add(4);

        let powered = self.nr52 & NR52_POWER_BIT != 0;

        if powered && self.ch1.pulse.enabled {
            self.ch1.pulse.tick_freq(self.nr13, self.nr14);
        }

        if powered && self.ch2.enabled {
            self.ch2.tick_freq(self.nr23, self.nr24);
        }

        if powered && self.ch3.enabled {
            self.tick_ch3_freq();
        }

        if powered && self.ch4.enabled {
            self.ch4.tick_freq(self.nr43);
        }

        if self.div_apu >= T_CYCLES_PER_FRAME_SEQUENCER_TICK {
            self.div_apu -= T_CYCLES_PER_FRAME_SEQUENCER_TICK;
            self.prev_frame_sequencer_step = self.frame_sequencer_step;
            self.frame_sequencer_step = (self.frame_sequencer_step + 1) & 0x07;

            if powered {
                let step = self.frame_sequencer_step;
                if step == 2 || step == 6 {
                    if self.ch1.pulse.enabled {
                        self.ch1.pulse.tick_envelope(self.nr12);
                    }
                    if self.ch2.enabled {
                        self.ch2.tick_envelope(self.nr22);
                    }
                    if self.ch4.enabled {
                        self.ch4.tick_envelope(self.nr42);
                    }
                    if self.ch1.pulse.enabled {
                        self.tick_ch1_sweep();
                    }
                }
            }
        }

        self.tick_downsample();
    }

    fn tick_downsample(&mut self) {
        let (left, right) = self.mixer_sample();
        self.sample_accum_left += left as f32;
        self.sample_accum_right += right as f32;
        self.sample_accum_count += 1;
        self.sample_phase += SAMPLE_RATE;

        while self.sample_phase >= M_CYCLES_PER_SECOND {
            self.sample_phase -= M_CYCLES_PER_SECOND;
            let divisor = self.sample_accum_count as f32;
            let sample_left = self.sample_accum_left / divisor;
            let sample_right = self.sample_accum_right / divisor;
            self.sample_accum_left = 0.0;
            self.sample_accum_right = 0.0;
            self.sample_accum_count = 0;
            self.push_sample(sample_left, sample_right);
        }
    }

    fn push_sample(&mut self, left: f32, right: f32) {
        if self.sample_buffer_count >= RING_BUFFER_SIZE {
            self.sample_buffer_read = (self.sample_buffer_read + 2) % (RING_BUFFER_SIZE * 2);
        } else {
            self.sample_buffer_count += 1;
        }
        self.sample_buffer[self.sample_buffer_write] = left;
        self.sample_buffer[self.sample_buffer_write + 1] = right;
        self.sample_buffer_write = (self.sample_buffer_write + 2) % (RING_BUFFER_SIZE * 2);
    }

    pub(crate) fn sample_buffer_len(&self) -> usize {
        self.sample_buffer_count
    }

    pub(crate) fn drain_sample_buffer(&mut self, output: &mut [(f32, f32)]) -> usize {
        let to_drain = output.len().min(self.sample_buffer_count);
        for entry in output.iter_mut().take(to_drain) {
            *entry = (
                self.sample_buffer[self.sample_buffer_read],
                self.sample_buffer[self.sample_buffer_read + 1],
            );
            self.sample_buffer_read = (self.sample_buffer_read + 2) % (RING_BUFFER_SIZE * 2);
        }
        self.sample_buffer_count -= to_drain;
        to_drain
    }

    pub(crate) fn mixer_sample(&self) -> (u16, u16) {
        let powered = self.nr52 & NR52_POWER_BIT != 0;
        if !powered {
            return (0, 0);
        }

        let ch1 = self.ch1_digital_output();
        let ch2 = self.ch2_digital_output();
        let ch3 = self.ch3_digital_output();
        let ch4 = self.ch4_digital_output();

        let mut left: u16 = 0;
        let mut right: u16 = 0;

        if self.nr51 & 0x10 != 0 {
            left += ch1 as u16;
        }
        if self.nr51 & 0x20 != 0 {
            left += ch2 as u16;
        }
        if self.nr51 & 0x40 != 0 {
            left += ch3 as u16;
        }
        if self.nr51 & 0x80 != 0 {
            left += ch4 as u16;
        }

        if self.nr51 & 0x01 != 0 {
            right += ch1 as u16;
        }
        if self.nr51 & 0x02 != 0 {
            right += ch2 as u16;
        }
        if self.nr51 & 0x04 != 0 {
            right += ch3 as u16;
        }
        if self.nr51 & 0x08 != 0 {
            right += ch4 as u16;
        }

        let left_vol = ((self.nr50 >> 4) & 0x07) as u16 + 1;
        let right_vol = (self.nr50 & 0x07) as u16 + 1;

        (left * left_vol, right * right_vol)
    }

    fn ch1_digital_output(&self) -> u8 {
        if !self.ch1_dac_enabled() || !self.ch1.pulse.enabled {
            return 0;
        }
        self.ch1.pulse.digital_output(self.nr11 >> 6)
    }

    fn ch2_digital_output(&self) -> u8 {
        if !self.ch2_dac_enabled() || !self.ch2.enabled {
            return 0;
        }
        self.ch2.digital_output(self.nr21 >> 6)
    }

    fn ch3_digital_output(&self) -> u8 {
        if !self.ch3_dac_enabled() || !self.ch3.enabled {
            return 0;
        }
        self.ch3.digital_output(self.nr32)
    }

    fn ch4_digital_output(&self) -> u8 {
        if !self.ch4_dac_enabled() || !self.ch4.enabled {
            return 0;
        }
        self.ch4.digital_output()
    }

    pub(crate) const fn frame_sequencer_step(&self) -> u8 {
        self.frame_sequencer_step
    }

    pub(crate) const fn ch1_enabled(&self) -> bool {
        self.ch1.pulse.enabled
    }

    pub(crate) const fn ch1_sweep_pace(&self) -> u8 {
        (self.nr10 >> 4) & 0x07
    }

    pub(crate) const fn ch1_sweep_direction(&self) -> u8 {
        (self.nr10 >> 3) & 1
    }

    pub(crate) const fn ch1_sweep_step(&self) -> u8 {
        self.nr10 & 0x07
    }

    pub(crate) const fn ch1_sweep_shadow(&self) -> u16 {
        self.ch1.sweep_shadow
    }

    pub(crate) const fn ch1_sweep_enabled(&self) -> bool {
        self.ch1.sweep_enabled
    }

    pub(crate) const fn ch1_sweep_timer(&self) -> u8 {
        self.ch1.sweep_timer
    }

    pub(crate) const fn ch1_duty_pattern(&self) -> u8 {
        self.nr11 >> 6
    }

    pub(crate) const fn ch1_initial_volume(&self) -> u8 {
        self.nr12 >> 4
    }

    pub(crate) const fn ch1_envelope_pace(&self) -> u8 {
        self.nr12 & 0x07
    }

    pub(crate) const fn ch1_dac_enabled(&self) -> bool {
        self.nr12 & 0xF8 != 0
    }

    pub(crate) const fn ch1_period(&self) -> u16 {
        period(self.nr13, self.nr14)
    }

    pub(crate) const fn ch1_frequency_timer(&self) -> u16 {
        self.ch1.pulse.freq_timer
    }

    pub(crate) const fn ch1_duty_step(&self) -> u8 {
        self.ch1.pulse.duty_step
    }

    pub(crate) const fn ch1_envelope_volume(&self) -> u8 {
        self.ch1.pulse.envelope_volume
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

    pub(crate) const fn ch3_enabled(&self) -> bool {
        self.ch3.enabled
    }

    pub(crate) const fn ch3_dac_enabled(&self) -> bool {
        self.nr30 & 0x80 != 0
    }

    pub(crate) const fn ch3_output_level(&self) -> u8 {
        (self.nr32 >> 5) & 0x03
    }

    pub(crate) const fn ch3_period(&self) -> u16 {
        period(self.nr33, self.nr34)
    }

    pub(crate) const fn ch3_frequency_timer(&self) -> u16 {
        self.ch3.freq_timer
    }

    pub(crate) const fn ch3_sample_index(&self) -> u8 {
        self.ch3.sample_index
    }

    pub(crate) const fn ch3_last_sample_buffer(&self) -> u8 {
        self.ch3.last_sample_buffer
    }

    pub(crate) const fn ch4_enabled(&self) -> bool {
        self.ch4.enabled
    }

    pub(crate) const fn ch4_dac_enabled(&self) -> bool {
        self.nr42 & 0xF8 != 0
    }

    pub(crate) const fn ch4_length_timer(&self) -> u8 {
        self.nr41 & 0x3F
    }

    pub(crate) const fn ch4_initial_volume(&self) -> u8 {
        self.nr42 >> 4
    }

    pub(crate) const fn ch4_envelope_pace(&self) -> u8 {
        self.nr42 & 0x07
    }

    pub(crate) const fn ch4_envelope_volume(&self) -> u8 {
        self.ch4.envelope_volume
    }

    pub(crate) const fn ch4_clock_shift(&self) -> u8 {
        self.nr43 >> 4
    }

    pub(crate) const fn ch4_lfsr_width_7bit(&self) -> bool {
        (self.nr43 >> 3) & 1 != 0
    }

    pub(crate) const fn ch4_clock_divider(&self) -> u8 {
        self.nr43 & 0x07
    }

    pub(crate) const fn ch4_lfsr_value(&self) -> u16 {
        self.ch4.lfsr
    }

    pub(crate) const fn ch4_noise_threshold(&self) -> u16 {
        noise_threshold(self.nr43)
    }

    pub(crate) const fn ch4_frequency_timer(&self) -> u16 {
        self.ch4.freq_timer
    }
}
