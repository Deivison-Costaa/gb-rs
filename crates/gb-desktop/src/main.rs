//! `gb-desktop` — frontend gráfico. ROADMAP 4.4.

#![forbid(unsafe_code)]

use std::process::ExitCode;

fn main() -> ExitCode {
    eprintln!(
        "gb-desktop {} (modelo {})",
        env!("CARGO_PKG_VERSION"),
        gb_core::MODEL
    );
    eprintln!("Frontend gráfico ainda não implementado: ROADMAP 4.4.");
    ExitCode::FAILURE
}
