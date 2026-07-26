use std::path::Path;
use std::process::ExitCode;

use gb_core::bus::Bus;
use gb_core::cart::{self, CartridgeHeader};
use gb_core::cpu::Cpu;

use crate::exit;

pub fn execute(path: &Path, max_cycles: u64) -> ExitCode {
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

    let cartridge = match cart::load(rom) {
        Ok(cart) => cart,
        Err(error) => {
            eprintln!("{}: {error}", path.display());
            return ExitCode::from(exit::DATA_ERROR);
        }
    };

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
}
