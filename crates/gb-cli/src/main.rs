//! `gb-cli` — runner headless das ROMs de teste.
//!
//! O contrato com `scripts/scoreboard.sh` já está escrito (R5: o teste vem
//! antes da implementação) e é este:
//!
//! ```text
//! gb-cli run <rom> --headless --max-cycles <n>
//!   exit 0    a ROM reportou sucesso
//!   exit 1    a ROM reportou falha
//!   exit 124  reservado ao timeout(1)
//!   outro     erro do emulador
//!   stdout    saída da porta serial, contendo o token `cycles=<n>`
//! ```
//!
//! Cumprir esse contrato é dos itens 0.3 (`info`) e 1.12 (`run`) do ROADMAP.
//! O 0.1 entrega só o binário.

#![forbid(unsafe_code)]

use std::process::ExitCode;

/// "O emulador não conseguiu executar a ROM."
///
/// `scoreboard.sh` classifica isto como `crash` — que é exatamente a verdade
/// hoje. `0` e `1` pertencem ao veredito da ROM e `124` ao `timeout(1)`;
/// reaproveitar qualquer um deles aqui plantaria um `pass` ou um `fail`
/// falso no `scoreboard.csv`, que é a série temporal da apresentação.
const EXIT_NOT_IMPLEMENTED: u8 = 2;

fn main() -> ExitCode {
    eprintln!(
        "gb-cli {} (modelo {})",
        env!("CARGO_PKG_VERSION"),
        gb_core::MODEL,
    );
    eprintln!("Nenhum subcomando implementado ainda: `info` chega no ROADMAP 0.3, `run` no 1.12.");
    ExitCode::from(EXIT_NOT_IMPLEMENTED)
}
