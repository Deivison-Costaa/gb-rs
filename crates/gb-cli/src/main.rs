//! `gb-cli` — runner headless e leitor de cabeçalho.
//! Contrato com scoreboard.sh: `run <rom> --headless --max-cycles <n>`, exit 0/1/124/outro.

#![forbid(unsafe_code)]

mod exit;
mod info;
mod run;

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
            let mut rom_path: Option<PathBuf> = None;
            let mut headless = false;
            let mut max_cycles: Option<u64> = None;
            let mut check_fb_hash: Option<String> = None;

            while let Some(arg) = args.next() {
                let s = arg.to_string_lossy();
                match s.as_ref() {
                    "--headless" => headless = true,
                    "--max-cycles" => {
                        let Some(val) = args.next() else {
                            return usage("`--max-cycles` precisa de um valor numérico");
                        };
                        let val_str = val.to_string_lossy();
                        max_cycles = match val_str.parse() {
                            Ok(n) => Some(n),
                            Err(_) => {
                                return usage(&format!(
                                    "`--max-cycles` espera um número, recebeu `{val_str}`"
                                ));
                            }
                        };
                    }
                    "--check-fb-hash" => {
                        let Some(val) = args.next() else {
                            return usage("`--check-fb-hash` precisa de um hash SHA-256 em hex");
                        };
                        check_fb_hash = Some(val.to_string_lossy().into_owned());
                    }
                    other if other.starts_with('-') => {
                        return usage(&format!("flag desconhecida: `{other}`"));
                    }
                    _ => {
                        if rom_path.is_some() {
                            return usage("`run` recebe exatamente uma ROM");
                        }
                        rom_path = Some(PathBuf::from(arg));
                    }
                }
            }

            let Some(path) = rom_path else {
                return usage("`run` precisa do caminho de uma ROM");
            };

            if !headless {
                return usage("`run` exige `--headless` (modo gráfico ainda não existe)");
            }

            let Some(cycles) = max_cycles else {
                return usage("`run` exige `--max-cycles <n>`");
            };

            if let Some(expected_hash) = check_fb_hash {
                run::execute_with_fb_hash(&path, cycles, &expected_hash)
            } else {
                run::execute(&path, cycles)
            }
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
