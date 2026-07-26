//! `gb-cli` — runner headless das ROMs de teste e leitor de cabeçalho.
//!
//! O contrato com `scripts/scoreboard.sh` foi escrito antes do binário (R5) e
//! é este:
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
//! O `run` é o ROADMAP 1.12 e ainda sai [`exit::NOT_IMPLEMENTED`]. O `info`
//! (ROADMAP 0.3b) está em [`info`]; os códigos de saída, em [`exit`].
//!
//! **Não há `--help`.** O uso vai para `stderr` junto com o erro que o
//! provocou, e é o mínimo que o item pediu; uma flag de ajuda seria
//! generalização que o ROADMAP não pediu (passo 5 do protocolo).

#![forbid(unsafe_code)]

mod exit;
mod info;

use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    // `args_os` e não `args`: este último **entra em pânico** com argumento que
    // não seja UTF-8 válido, e caminho de arquivo não tem obrigação de ser.
    // Pânico ao receber caminho esquisito seria sair com código de sinal, que
    // o scoreboard leria como categoria própria.
    let mut args = std::env::args_os().skip(1);

    let Some(subcommand) = args.next() else {
        return usage("nenhum subcomando");
    };

    match subcommand.as_os_str().to_str() {
        Some("info") => {
            let Some(rom) = args.next() else {
                return usage("`info` precisa do caminho de uma ROM");
            };
            // Argumento a mais costuma ser um glob que casou com duas ROMs.
            // Ler a primeira em silêncio esconderia o engano de quem chamou.
            if args.next().is_some() {
                return usage("`info` recebe exatamente uma ROM");
            }
            info::report(&PathBuf::from(rom))
        }
        Some("run") => {
            eprintln!("`run` chega no ROADMAP 1.12: ainda não há CPU para executar a ROM.");
            ExitCode::from(exit::NOT_IMPLEMENTED)
        }
        _ => usage(&format!(
            "subcomando desconhecido: {}",
            subcommand.as_os_str().to_string_lossy()
        )),
    }
}

/// Reclama do uso em `stderr` e sai com [`exit::USAGE`].
///
/// Vai para `stderr`, e não `stdout`, para não contaminar a saída que um
/// `gb-cli info rom.gb > relatorio.txt` estaria capturando.
fn usage(problem: &str) -> ExitCode {
    let program = std::env::args_os()
        .next()
        .unwrap_or_else(|| OsStr::new("gb-cli").to_os_string());

    eprintln!("{}: {problem}", program.to_string_lossy());
    eprintln!();
    eprintln!("uso:");
    eprintln!("  gb-cli info <rom>");
    eprintln!("  gb-cli run <rom> --headless --max-cycles <n>   (ROADMAP 1.12)");
    ExitCode::from(exit::USAGE)
}
