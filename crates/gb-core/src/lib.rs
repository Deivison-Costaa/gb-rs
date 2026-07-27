#![forbid(unsafe_code)]

mod apu;
pub mod bus;
pub mod cart;
pub mod cpu;
pub mod joypad;
mod ppu;
mod serial;

pub const MODEL: &str = "DMG";
