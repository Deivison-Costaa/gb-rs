use std::path::PathBuf;
use std::process::ExitCode;

mod gamepad;
mod run;

fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);

    let Some(rom_arg) = args.next() else {
        eprintln!("uso: gb-desktop <rom>");
        eprintln!("gb-desktop {}", env!("CARGO_PKG_VERSION"));
        eprintln!("Frontend gráfico do gb-rs (ROADMAP 4.4)");
        return ExitCode::from(64);
    };

    let path = PathBuf::from(rom_arg);

    run::execute(&path);

    ExitCode::SUCCESS
}
