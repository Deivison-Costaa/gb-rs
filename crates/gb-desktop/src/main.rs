//! `gb-desktop` — frontend gráfico (janela, teclado, áudio).
//!
//! Chega de verdade no ROADMAP 4.4, quando já houver PPU (M3) e joypad (4.1)
//! e, portanto, algo para desenhar e alguém para receber as teclas. Por ora o
//! crate existe para que o workspace tenha a forma final desde o começo.

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
