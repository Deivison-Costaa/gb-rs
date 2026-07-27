#![forbid(unsafe_code)]

pub mod bus;
pub mod cart;
pub mod cpu;
mod ppu;
mod serial;

pub const MODEL: &str = "DMG";
