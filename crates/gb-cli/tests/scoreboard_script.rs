//! Guarda comportamental do `scripts/scoreboard.sh` — ROADMAP 0.2b.
//!
//! O `scoreboard.csv` é a série temporal que vira gráfico na apresentação
//! (ROADMAP 8.2). O modo de falha que interessa não é o script explodir — isso
//! `set -e` já resolve, e a CI pinta de vermelho. É o script **terminar
//! bem-sucedido sem ter anexado nada**: nenhuma ROM encontrada, `find` mudo,
//! laço que não executou nenhuma volta. O job fica verde, o artefato sobe com o
//! CSV de ontem, e a série para de crescer sem que ninguém receba um sinal.
//!
//! Foi exatamente esse o cenário da nota 7 do `STATUS.md` visto do outro lado:
//! lá o script morria em silêncio, aqui ele poderia sair `0` em silêncio. Os
//! dois se detectam do mesmo jeito — perguntando ao CSV se ele cresceu.
//!
//! Os testes rodam o script de verdade, num diretório temporário próprio, com
//! `SCOREBOARD_CSV` e `ROMS_DIR` apontados para lá. **Nunca** tocam o
//! `scoreboard.csv` versionado na raiz: contaminá-lo com linhas de teste
//! corromperia o dado da apresentação.
//!
//! `GB_CLI` aponta para o binário que o próprio `cargo test` acabou de
//! construir. Sem isso, `find_gb_cli` cairia no `cargo build --release` de
//! best-effort — um cargo aninhado dentro de um teste do cargo, disputando o
//! lock do diretório de build.
//!
//! `unwrap`/`expect` são permitidos aqui: R6 proíbe fora de teste.

#![cfg(unix)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// `crates/gb-cli` → `crates` → raiz do workspace.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("gb-cli deve morar em <raiz>/crates/gb-cli")
        .to_path_buf()
}

/// Diretório de trabalho exclusivo deste caso de teste, sempre zerado antes de
/// usar. Mora sob `target/` para sair no `cargo clean` e já estar no
/// `.gitignore`.
fn sandbox(name: &str) -> PathBuf {
    let dir = workspace_root().join("target/tests-tmp").join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("limpar sandbox");
    }
    std::fs::create_dir_all(&dir).expect("criar sandbox");
    dir
}

/// Roda o `scoreboard.sh` contra um sandbox com as ROMs falsas dadas.
///
/// As "ROMs" são arquivos vazios: o `gb-cli` de hoje sai `2` sem sequer abrir o
/// arquivo, e o que este teste mede é o script, não o emulador.
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

/// Linhas de dado do CSV — o cabeçalho não conta.
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

/// ROADMAP 0.2b — o caso que importa: nenhuma linha anexada é **erro**.
///
/// Sem ROMs, o laço não roda e o CSV fica só com o cabeçalho. Sair `0` aqui é
/// afirmar "medi 0 de 0 ROMs e está tudo bem", quando o que houve foi não medir
/// nada.
///
/// O teste exige a **mensagem**, não só o código de saída — e não por
/// preciosismo: na primeira versão ele passou verde contra o script antigo,
/// que morria três linhas antes com `total: variável não associada`. Falhar não
/// é a mesma coisa que falhar pelo motivo certo, e um teste que não distingue
/// os dois estava medindo um bug com o outro.
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

/// Controle positivo. Sem ele, um script que falhasse *sempre* passaria no
/// teste acima — a vacuidade da nota 8 do `STATUS.md`, de cabeça para baixo.
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

/// O CSV é acumulativo (invariante do projeto): a segunda execução anexa à
/// primeira, não a substitui. É também o que garante que a checagem de
/// crescimento compara com o tamanho **anterior**, e não com zero.
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
