//! ROADMAP 0.3b — `gb-cli info <rom>`: a casca de I/O do parser da 0005.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const ROM_LEN: usize = 0x0150;

const TITLE: usize = 0x0134;
const CARTRIDGE_TYPE: usize = 0x0147;
const ROM_SIZE: usize = 0x0148;
const RAM_SIZE: usize = 0x0149;
const HEADER_CHECKSUM: usize = 0x014D;

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

fn boot_rom_checksum(rom: &[u8]) -> u8 {
    let mut checksum: u8 = 0;
    for &byte in &rom[0x0134..=0x014C] {
        checksum = checksum.wrapping_sub(byte).wrapping_sub(1);
    }
    checksum
}

fn rom(title: &[u8], cartridge_type: u8, rom_size: u8, ram_size: u8) -> Vec<u8> {
    let mut rom = vec![0x00; ROM_LEN];
    rom[TITLE..TITLE + title.len()].copy_from_slice(title);
    rom[CARTRIDGE_TYPE] = cartridge_type;
    rom[ROM_SIZE] = rom_size;
    rom[RAM_SIZE] = ram_size;
    rom[HEADER_CHECKSUM] = boot_rom_checksum(&rom);
    rom
}

fn write_rom(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, bytes).expect("gravar ROM sintética");
    path
}

fn gb_cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_gb-cli"))
        .args(args)
        .output()
        .expect("executar gb-cli")
}

fn describe(out: &Output) -> String {
    format!(
        "saída {:?}\n--- stdout ---\n{}\n--- stderr ---\n{}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    )
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn field(text: &str, label: &str) -> String {
    let prefix = format!("{label}:");
    let line = text
        .lines()
        .find(|line| line.starts_with(&prefix))
        .unwrap_or_else(|| panic!("a saída não tem o campo `{label}`:\n{text}"));
    line[prefix.len()..].trim().to_string()
}

fn code(out: &Output) -> Option<i32> {
    out.status.code()
}

// ---------------------------------------------------------------------------
// O caminho feliz
// ---------------------------------------------------------------------------

#[test]
fn info_prints_every_header_field_of_a_well_formed_rom() {
    let dir = sandbox("info-feliz");
    let bytes = rom(b"TETRIS", 0x01, 0x01, 0x02);
    let path = write_rom(&dir, "tetris.gb", &bytes);
    let out = gb_cli(&["info", path.to_str().unwrap()]);

    assert_eq!(code(&out), Some(0), "ROM íntegra\n{}", describe(&out));

    let text = stdout(&out);
    assert_eq!(field(&text, "título"), "TETRIS");
    assert_eq!(field(&text, "tipo"), "$01 MBC1");
    assert_eq!(field(&text, "ROM"), "$01 64 KiB (4 bancos)");
    assert_eq!(field(&text, "RAM"), "$02 8 KiB");
    assert_eq!(
        field(&text, "checksum"),
        format!(
            "${0:02X} (calculado ${0:02X}) — confere",
            bytes[HEADER_CHECKSUM]
        )
    );
}

#[test]
fn info_writes_the_report_to_stdout_not_stderr() {
    let dir = sandbox("info-stdout");
    let path = write_rom(&dir, "tetris.gb", &rom(b"TETRIS", 0x00, 0x00, 0x00));
    let out = gb_cli(&["info", path.to_str().unwrap()]);

    assert!(stdout(&out).contains("TETRIS"), "{}", describe(&out));
    assert!(
        stderr(&out).is_empty(),
        "ROM íntegra não deveria escrever nada em stderr\n{}",
        describe(&out)
    );
}

#[test]
fn info_reports_the_file_it_read_and_its_size() {
    let dir = sandbox("info-arquivo");
    let bytes = rom(b"TETRIS", 0x00, 0x00, 0x00);
    let path = write_rom(&dir, "tetris.gb", &bytes);
    let out = gb_cli(&["info", path.to_str().unwrap()]);

    assert_eq!(field(&stdout(&out), "arquivo"), path.display().to_string());
    assert_eq!(
        field(&stdout(&out), "tamanho"),
        format!("{} bytes", bytes.len())
    );
}

#[test]
fn info_says_the_title_is_empty_instead_of_printing_a_blank_line() {
    let dir = sandbox("info-sem-titulo");
    let path = write_rom(&dir, "anonima.gb", &rom(b"", 0x00, 0x00, 0x00));
    let out = gb_cli(&["info", path.to_str().unwrap()]);

    assert_eq!(
        field(&stdout(&out), "título"),
        "(vazio)",
        "campo em branco é indistinguível de campo faltando\n{}",
        describe(&out)
    );
}

// ---------------------------------------------------------------------------
// Códigos desconhecidos: relatar, nunca inventar (invariante da 0005)
// ---------------------------------------------------------------------------

#[test]
fn info_reports_an_unknown_cartridge_type_without_failing() {
    let dir = sandbox("info-tipo-desconhecido");
    let path = write_rom(&dir, "estranha.gb", &rom(b"ESTRANHA", 0x04, 0x00, 0x00));
    let out = gb_cli(&["info", path.to_str().unwrap()]);

    assert_eq!(code(&out), Some(0), "{}", describe(&out));
    assert_eq!(field(&stdout(&out), "tipo"), "$04 desconhecido");
}

#[test]
fn info_never_invents_two_kib_for_ram_code_one() {
    let dir = sandbox("info-ram-01");
    let path = write_rom(&dir, "homebrew.gb", &rom(b"HOMEBREW", 0x00, 0x00, 0x01));
    let out = gb_cli(&["info", path.to_str().unwrap()]);

    assert_eq!(code(&out), Some(0), "{}", describe(&out));
    assert_eq!(field(&stdout(&out), "RAM"), "$01 tamanho desconhecido");
    // Escopado ao campo da RAM de propósito: a versão que varria a saída
    // inteira reprovava a implementação **correta**, porque a linha da ROM diz
    // `32 KiB` e `"32 KiB".contains("2 KiB")` é verdadeiro.
    assert!(
        !field(&stdout(&out), "RAM").contains("2 KiB"),
        "o Pan Docs marca $01 como Unused; 2 KiB é número inventado\n{}",
        describe(&out)
    );
}

#[test]
fn info_reports_an_unattested_rom_size_as_unknown() {
    let dir = sandbox("info-rom-52");
    let path = write_rom(&dir, "estranha.gb", &rom(b"ESTRANHA", 0x00, 0x52, 0x00));
    let out = gb_cli(&["info", path.to_str().unwrap()]);

    assert_eq!(code(&out), Some(0), "{}", describe(&out));
    assert_eq!(field(&stdout(&out), "ROM"), "$52 tamanho desconhecido");
}

#[test]
fn info_reports_a_broken_checksum_and_still_succeeds() {
    let dir = sandbox("info-checksum-quebrado");
    let mut bytes = rom(b"CORROMPIDA", 0x00, 0x00, 0x00);
    let computed = bytes[HEADER_CHECKSUM];
    bytes[HEADER_CHECKSUM] = computed.wrapping_add(1);
    let path = write_rom(&dir, "corrompida.gb", &bytes);
    let out = gb_cli(&["info", path.to_str().unwrap()]);

    assert_eq!(
        code(&out),
        Some(0),
        "cabeçalho corrompido é diagnóstico, não erro de execução\n{}",
        describe(&out)
    );
    assert_eq!(
        field(&stdout(&out), "checksum"),
        format!(
            "${:02X} (calculado ${:02X}) — NÃO CONFERE",
            computed.wrapping_add(1),
            computed
        )
    );
}

// ---------------------------------------------------------------------------
// Erros: cada um com seu código de saída
// ---------------------------------------------------------------------------

#[test]
fn truncated_rom_exits_with_data_error() {
    let dir = sandbox("info-truncada");
    let path = write_rom(&dir, "truncada.gb", &vec![0x00; ROM_LEN - 1]);
    let out = gb_cli(&["info", path.to_str().unwrap()]);

    assert_eq!(code(&out), Some(65), "{}", describe(&out));
    assert!(
        stderr(&out).contains("cabeçalho"),
        "o erro tem de dizer que o cabeçalho não coube\n{}",
        describe(&out)
    );
    assert!(
        stdout(&out).is_empty(),
        "relatório parcial de ROM inválida é pior que relatório nenhum\n{}",
        describe(&out)
    );
}

#[test]
fn missing_file_exits_with_no_input() {
    let dir = sandbox("info-inexistente");
    let path = dir.join("nao-existe.gb");
    let out = gb_cli(&["info", path.to_str().unwrap()]);

    assert_eq!(code(&out), Some(66), "{}", describe(&out));
    assert!(
        stderr(&out).contains("nao-existe.gb"),
        "a mensagem tem de dizer qual caminho falhou\n{}",
        describe(&out)
    );
}

#[test]
fn directory_instead_of_rom_exits_with_no_input() {
    let dir = sandbox("info-diretorio");
    let out = gb_cli(&["info", dir.to_str().unwrap()]);

    assert_eq!(code(&out), Some(66), "{}", describe(&out));
}

// ---------------------------------------------------------------------------
// Argumentos
// ---------------------------------------------------------------------------

#[test]
fn no_arguments_exits_with_usage() {
    let out = gb_cli(&[]);

    assert_eq!(code(&out), Some(64), "{}", describe(&out));
    assert!(
        stderr(&out).contains("info"),
        "o uso tem de listar os subcomandos\n{}",
        describe(&out)
    );
}

#[test]
fn info_without_a_rom_exits_with_usage() {
    let out = gb_cli(&["info"]);

    assert_eq!(code(&out), Some(64), "{}", describe(&out));
}

#[test]
fn info_with_extra_arguments_exits_with_usage() {
    let dir = sandbox("info-argumento-extra");
    let path = write_rom(&dir, "tetris.gb", &rom(b"TETRIS", 0x00, 0x00, 0x00));
    let out = gb_cli(&["info", path.to_str().unwrap(), "sobrando"]);

    assert_eq!(code(&out), Some(64), "{}", describe(&out));
    assert!(
        stdout(&out).is_empty(),
        "não deveria ter lido a ROM antes de reclamar\n{}",
        describe(&out)
    );
}

#[test]
fn unknown_subcommand_exits_with_usage() {
    let out = gb_cli(&["informacao", "tetris.gb"]);

    assert_eq!(code(&out), Some(64), "{}", describe(&out));
}

#[test]
fn run_with_relative_path_that_does_not_point_to_an_existing_rom_exits_with_no_input() {
    let out = gb_cli(&["run", "tetris.gb", "--headless", "--max-cycles", "1000"]);

    assert_eq!(
        code(&out),
        Some(66),
        "arquivo inexistente tem de sair com NO_INPUT, não com 2\n{}",
        describe(&out)
    );
}
