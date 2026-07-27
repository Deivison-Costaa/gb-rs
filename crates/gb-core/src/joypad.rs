//! spec: `docs/reference/09-joypad-serial.md` § Joypad Input.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Right,
    Left,
    Up,
    Down,
    A,
    B,
    Start,
    Select,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Joypad {
    select: u8,
    dpad: u8,
    buttons: u8,
    pub(crate) interrupt: bool,
}

impl Default for Joypad {
    fn default() -> Self {
        Self {
            select: 0x00,
            dpad: 0x0F,
            buttons: 0x0F,
            interrupt: false,
        }
    }
}

impl Joypad {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn read(&self) -> u8 {
        let mut value = 0xC0;
        value |= self.select;
        let mut low = 0x0F;
        if self.select & 0x20 == 0 {
            low &= self.buttons;
        }
        if self.select & 0x10 == 0 {
            low &= self.dpad;
        }
        value |= low & 0x0F;
        value
    }

    pub fn write(&mut self, value: u8) {
        self.select = value & 0x30;
    }

    pub fn key_down(&mut self, key: Key) {
        match key {
            Key::Right => self.dpad &= !0x01,
            Key::Left => self.dpad &= !0x02,
            Key::Up => self.dpad &= !0x04,
            Key::Down => self.dpad &= !0x08,
            Key::A => self.buttons &= !0x01,
            Key::B => self.buttons &= !0x02,
            Key::Select => self.buttons &= !0x04,
            Key::Start => self.buttons &= !0x08,
        }
        self.interrupt = true;
    }

    pub fn key_up(&mut self, key: Key) {
        match key {
            Key::Right => self.dpad |= 0x01,
            Key::Left => self.dpad |= 0x02,
            Key::Up => self.dpad |= 0x04,
            Key::Down => self.dpad |= 0x08,
            Key::A => self.buttons |= 0x01,
            Key::B => self.buttons |= 0x02,
            Key::Select => self.buttons |= 0x04,
            Key::Start => self.buttons |= 0x08,
        }
    }
}
