//! `gb-cli info <rom>` — ROADMAP 0.3b. Casca de I/O do parser do cartucho.
//! spec: `docs/reference/08-cartridges-mbc.md` § The Cartridge Header.

use std::path::Path;
use std::process::ExitCode;

use gb_core::cart::{CartridgeHeader, CartridgeType, HeaderChecksum, RamSize, RomSize};

use crate::exit;

const LABEL_WIDTH: usize = 10;
const UNKNOWN_SIZE: &str = "tamanho desconhecido";

pub fn report(path: &Path) -> ExitCode {
    let rom = match std::fs::read(path) {
        Ok(rom) => rom,
        Err(error) => {
            eprintln!("não consegui ler {}: {error}", path.display());
            return ExitCode::from(exit::NO_INPUT);
        }
    };

    let header = match CartridgeHeader::parse(&rom) {
        Ok(header) => header,
        Err(error) => {
            eprintln!("{}: {error}", path.display());
            return ExitCode::from(exit::DATA_ERROR);
        }
    };

    print!("{}", render(path, rom.len(), &header));
    ExitCode::SUCCESS
}

fn render(path: &Path, rom_len: usize, header: &CartridgeHeader) -> String {
    let title = if header.title().is_empty() {
        "(vazio)".to_string()
    } else {
        header.title().to_string()
    };

    [
        line("arquivo", &path.display().to_string()),
        line("tamanho", &format!("{rom_len} bytes")),
        line("título", &title),
        line("tipo", &cartridge_type(header.cartridge_type())),
        line("ROM", &rom_size(header.rom_size())),
        line("RAM", &ram_size(header.ram_size())),
        line("checksum", &checksum(header.checksum())),
    ]
    .concat()
}

fn line(label: &str, value: &str) -> String {
    let label = format!("{label}:");
    format!("{label:<LABEL_WIDTH$}{value}\n")
}

fn cartridge_type(kind: CartridgeType) -> String {
    let name = kind.name().unwrap_or("desconhecido");
    format!("${:02X} {name}", kind.code())
}

fn rom_size(size: RomSize) -> String {
    let described = match (size.bytes(), size.banks()) {
        (Some(bytes), Some(banks)) => format!("{} ({banks} bancos)", human(bytes)),
        _ => UNKNOWN_SIZE.to_string(),
    };
    format!("${:02X} {described}", size.code())
}

fn ram_size(size: RamSize) -> String {
    let described = size.bytes().map_or_else(|| UNKNOWN_SIZE.to_string(), human);
    format!("${:02X} {described}", size.code())
}

fn checksum(checksum: HeaderChecksum) -> String {
    let verdict = if checksum.is_valid() {
        "confere"
    } else {
        "NÃO CONFERE"
    };
    format!(
        "${:02X} (calculado ${:02X}) — {verdict}",
        checksum.stored(),
        checksum.computed()
    )
}

// Tamanho em KiB ou MiB, só quando a divisão é exata.
// % e não is_multiple_of: MSRV é 1.85 e is_multiple_of entrou em 1.87.
fn human(bytes: u32) -> String {
    const KIB: u32 = 1024;
    const MIB: u32 = 1024 * KIB;

    if bytes >= MIB && bytes % MIB == 0 {
        format!("{} MiB", bytes / MIB)
    } else if bytes >= KIB && bytes % KIB == 0 {
        format!("{} KiB", bytes / KIB)
    } else {
        format!("{bytes} B")
    }
}
