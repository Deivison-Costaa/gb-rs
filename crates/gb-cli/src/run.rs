use std::path::Path;
use std::process::ExitCode;

use gb_core::bus::Bus;
use gb_core::cart::{self, CartridgeHeader};
use gb_core::cpu::Cpu;
use sha2::{Digest, Sha256};

use crate::exit;

pub fn execute(path: &Path, max_cycles: u64) -> ExitCode {
    execute_inner(path, max_cycles, None)
}

pub fn execute_with_fb_hash(path: &Path, max_cycles: u64, expected_hash: &str) -> ExitCode {
    execute_inner(path, max_cycles, Some(expected_hash))
}

fn execute_inner(path: &Path, max_cycles: u64, expected_fb_hash: Option<&str>) -> ExitCode {
    let rom = match std::fs::read(path) {
        Ok(rom) => rom,
        Err(error) => {
            eprintln!("não consegui ler {}: {error}", path.display());
            return ExitCode::from(exit::NO_INPUT);
        }
    };

    let checksum = match CartridgeHeader::parse(&rom) {
        Ok(header) => header.checksum(),
        Err(error) => {
            eprintln!("{}: {error}", path.display());
            return ExitCode::from(exit::DATA_ERROR);
        }
    };

    let sav_path = path.with_extension("sav");
    let sav_data = std::fs::read(&sav_path).ok();

    let mut cartridge = match cart::load(rom) {
        Ok(cart) => cart,
        Err(error) => {
            eprintln!("{}: {error}", path.display());
            return ExitCode::from(exit::DATA_ERROR);
        }
    };

    if let Some(ref data) = sav_data {
        cartridge.load_ram(data);
    }

    let mut bus = Bus::new(cartridge);
    let mut cpu = Cpu::after_boot_rom(checksum);

    let mut cycle_count: u64 = 0;
    while cycle_count < max_cycles {
        if cpu.lockup().is_some() || cpu.is_stopped() {
            break;
        }
        cpu.step(&mut bus);
        cycle_count = cycle_count.saturating_add(1);
    }

    let result = if let Some(expected) = expected_fb_hash {
        let fb = bus.framebuffer();
        let mut hasher = Sha256::new();
        hasher.update(fb);
        let hash = format!("{:x}", hasher.finalize());

        println!("fb-hash={hash}");
        println!("cycles={cycle_count}");

        if hash == expected {
            ExitCode::SUCCESS
        } else {
            ExitCode::from(1)
        }
    } else {
        let serial_output = bus.take_serial_output();
        let serial_text = String::from_utf8_lossy(&serial_output);

        print!("{serial_text}");
        println!("cycles={cycle_count}");

        if serial_text.contains("Passed") {
            ExitCode::SUCCESS
        } else if serial_text.contains("Failed") {
            ExitCode::from(1)
        } else {
            ExitCode::from(exit::NOT_IMPLEMENTED)
        }
    };

    if let Some(ram) = bus.cartridge_ram() {
        let _ = std::fs::write(&sav_path, ram);
    }

    result
}
