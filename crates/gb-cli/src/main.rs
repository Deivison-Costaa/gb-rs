//! `gb-cli` — runner headless e leitor de cabeçalho.
//! Contrato com scoreboard.sh: `run <rom> --headless --max-cycles <n>`, exit 0/1/124/outro.

#![forbid(unsafe_code)]

mod exit;
mod info;

use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    // args_os e não args: args entra em pânico com argumento não UTF-8.
    let mut args = std::env::args_os().skip(1);

    let Some(subcommand) = args.next() else {
        return usage("nenhum subcomando");
    };

    match subcommand.as_os_str().to_str() {
        Some("info") => {
            let Some(rom) = args.next() else {
                return usage("`info` precisa do caminho de uma ROM");
            };
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
