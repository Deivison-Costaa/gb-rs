//! Guarda comportamental do `scripts/scoreboard.sh` — ROADMAP 0.2b.

#![cfg(unix)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("gb-cli deve morar em <raiz>/crates/gb-cli")
        .to_path_buf()
}

fn sandbox(name: &str) -> PathBuf {
    let dir = workspace_root().join("target/tests-tmp").join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("limpar sandbox");
    }
    std::fs::create_dir_all(&dir).expect("criar sandbox");
    dir
}

fn run_scoreboard(dir: &Path, roms: &[&str]) -> Output {
    let roms_dir = dir.join("roms");
    for rom in roms {
        let path = roms_dir.join(rom);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("criar diretório da ROM falsa");
        }
        std::fs::write(&path, b"").expect("criar ROM falsa");
    }
    std::fs::create_dir_all(&roms_dir).expect("criar diretório de ROMs");

    Command::new(workspace_root().join("scripts/scoreboard.sh"))
        .env("ROMS_DIR", &roms_dir)
        .env("SCOREBOARD_CSV", dir.join("scoreboard.csv"))
        .env("GB_CLI", env!("CARGO_BIN_EXE_gb-cli"))
        .env("ROM_TIMEOUT", "10")
        .output()
        .expect("executar scripts/scoreboard.sh")
}

fn data_rows(dir: &Path) -> usize {
    let csv = dir.join("scoreboard.csv");
    let text = std::fs::read_to_string(&csv)
        .unwrap_or_else(|e| panic!("não consegui ler {}: {e}", csv.display()));
    text.lines()
        .filter(|l| !l.is_empty())
        .count()
        .saturating_sub(1)
}

fn describe(out: &Output) -> String {
    format!(
        "saída {:?}\n--- stdout ---\n{}\n--- stderr ---\n{}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    )
}

#[test]
fn scoreboard_fails_when_no_row_is_appended() {
    let dir = sandbox("scoreboard-vazio");
    let out = run_scoreboard(&dir, &[]);

    assert!(
        !out.status.success(),
        "ROADMAP 0.2b: o scoreboard não anexou nenhuma linha e mesmo assim saiu \
         com sucesso — a CI ficaria verde com a série congelada.\n{}",
        describe(&out)
    );

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("não anexou nenhuma linha"),
        "falhou, mas não por ter detectado o CSV parado\n{}",
        describe(&out)
    );
    assert_eq!(data_rows(&dir), 0, "não havia ROM para medir");
}

#[test]
fn scoreboard_succeeds_and_appends_one_row_per_rom() {
    let dir = sandbox("scoreboard-uma-rom");
    let out = run_scoreboard(&dir, &["blargg/cpu_instrs/01-especial.gb"]);

    assert!(
        out.status.success(),
        "havia ROM para medir e o script falhou\n{}",
        describe(&out)
    );
    assert_eq!(data_rows(&dir), 1, "uma ROM, uma linha\n{}", describe(&out));
}

#[test]
fn scoreboard_appends_to_an_existing_csv_without_truncating() {
    let dir = sandbox("scoreboard-acumula");
    let roms = ["blargg/cpu_instrs/01-especial.gb", "dmg-acid2/dmg-acid2.gb"];

    let first = run_scoreboard(&dir, &roms);
    assert!(first.status.success(), "{}", describe(&first));
    assert_eq!(data_rows(&dir), 2);

    let second = run_scoreboard(&dir, &roms);
    assert!(second.status.success(), "{}", describe(&second));
    assert_eq!(
        data_rows(&dir),
        4,
        "a segunda execução truncou o CSV em vez de anexar\n{}",
        describe(&second)
    );
}
