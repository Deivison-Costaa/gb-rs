//! ROADMAP 1.13 — `gb-cli run <rom> --headless --max-cycles <n>`.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const MIN_ROM_LEN: usize = 0x0150;

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

fn rom(title: &[u8], code_at_entry: &[u8]) -> Vec<u8> {
    let mut rom = vec![0x00; MIN_ROM_LEN];
    rom[TITLE..TITLE + title.len()].copy_from_slice(title);
    rom[CARTRIDGE_TYPE] = 0x00;
    rom[ROM_SIZE] = 0x00;
    rom[RAM_SIZE] = 0x00;

    let entry = 0x0100usize;
    let end = entry + code_at_entry.len();
    rom[entry..end].copy_from_slice(code_at_entry);

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

fn code(out: &Output) -> Option<i32> {
    out.status.code()
}

// ---------------------------------------------------------------------------
// argumentos inválidos
// ---------------------------------------------------------------------------

#[test]
fn run_without_rom_exits_with_usage() {
    let out = gb_cli(&["run"]);

    assert_eq!(code(&out), Some(64), "{}", describe(&out));
    assert!(
        stdout(&out).is_empty(),
        "erro de argumento não deveria escrever em stdout\n{}",
        describe(&out)
    );
}

#[test]
fn run_without_headless_exits_with_usage() {
    let dir = sandbox("run-sem-headless");
    let rom_bytes = rom(b"TEST", &[0x18, 0xFE]);
    let path = write_rom(&dir, "teste.gb", &rom_bytes);
    let out = gb_cli(&["run", path.to_str().unwrap(), "--max-cycles", "100"]);

    assert_eq!(code(&out), Some(64), "{}", describe(&out));
    assert!(
        stderr(&out).contains("headless"),
        "a mensagem tem de mencionar --headless\n{}",
        describe(&out)
    );
}

#[test]
fn run_without_max_cycles_exits_with_usage() {
    let dir = sandbox("run-sem-max-cycles");
    let rom_bytes = rom(b"TEST", &[0x18, 0xFE]);
    let path = write_rom(&dir, "teste.gb", &rom_bytes);
    let out = gb_cli(&["run", path.to_str().unwrap(), "--headless"]);

    assert_eq!(code(&out), Some(64), "{}", describe(&out));
    assert!(
        stderr(&out).contains("max-cycles"),
        "a mensagem tem de mencionar --max-cycles\n{}",
        describe(&out)
    );
}

#[test]
fn run_with_non_numeric_max_cycles_exits_with_usage() {
    let dir = sandbox("run-max-cycles-invalido");
    let rom_bytes = rom(b"TEST", &[0x18, 0xFE]);
    let path = write_rom(&dir, "teste.gb", &rom_bytes);
    let out = gb_cli(&[
        "run",
        path.to_str().unwrap(),
        "--headless",
        "--max-cycles",
        "doze",
    ]);

    assert_eq!(code(&out), Some(64), "{}", describe(&out));
}

// ---------------------------------------------------------------------------
// ROM ausente ou inválida
// ---------------------------------------------------------------------------

#[test]
fn run_with_missing_rom_file_exits_with_no_input() {
    let dir = sandbox("run-rom-inexistente");
    let path = dir.join("nao-existe.gb");
    let out = gb_cli(&[
        "run",
        path.to_str().unwrap(),
        "--headless",
        "--max-cycles",
        "100",
    ]);

    assert_eq!(code(&out), Some(66), "{}", describe(&out));
    assert!(
        stderr(&out).contains("nao-existe.gb"),
        "a mensagem tem de dizer qual caminho falhou\n{}",
        describe(&out)
    );
}

#[test]
fn run_with_truncated_rom_exits_with_data_error() {
    let dir = sandbox("run-rom-truncada");
    let path = write_rom(&dir, "truncada.gb", &vec![0x00; MIN_ROM_LEN - 1]);
    let out = gb_cli(&[
        "run",
        path.to_str().unwrap(),
        "--headless",
        "--max-cycles",
        "100",
    ]);

    assert_eq!(code(&out), Some(65), "{}", describe(&out));
}

// ---------------------------------------------------------------------------
// execução básica
// ---------------------------------------------------------------------------

#[test]
fn run_executes_a_valid_rom_and_prints_cycle_count() {
    let dir = sandbox("run-executa-rom");
    let rom_bytes = rom(b"TEST", &[0x18, 0xFE]);
    let path = write_rom(&dir, "teste.gb", &rom_bytes);
    let out = gb_cli(&[
        "run",
        path.to_str().unwrap(),
        "--headless",
        "--max-cycles",
        "50",
    ]);

    let text = stdout(&out);
    assert!(
        text.contains("cycles="),
        "a saída tem de conter cycles=<n>\n{}",
        describe(&out)
    );

    let cycles_line = text
        .lines()
        .find(|l| l.starts_with("cycles="))
        .expect("a linha cycles= não foi encontrada");
    let n: u64 = cycles_line
        .strip_prefix("cycles=")
        .unwrap()
        .parse()
        .expect("cycles= seguido de número inválido");
    assert!(
        n > 0,
        "o laço de step tem de consumir ciclos\n{}",
        describe(&out)
    );
}

#[test]
fn run_exits_2_when_rom_produces_no_serial_result() {
    let dir = sandbox("run-sem-resultado");
    let rom_bytes = rom(b"TEST", &[0x18, 0xFE]);
    let path = write_rom(&dir, "teste.gb", &rom_bytes);
    let out = gb_cli(&[
        "run",
        path.to_str().unwrap(),
        "--headless",
        "--max-cycles",
        "50",
    ]);

    assert_eq!(
        code(&out),
        Some(2),
        "ROM sem saída serial não tem como decidir pass/fail\n{}",
        describe(&out)
    );
}

#[test]
fn run_prints_serial_output_as_utf8_lossy() {
    let dir = sandbox("run-serial-output");
    // Programa que escreve 'P' no SB ($FF01) e dispara a transferência ($81 em SC, $FF02)
    // LD A,'P'; LDH ($01),A; LD A,$81; LDH ($02),A; JR -2 (loop infinito)
    let code: &[u8] = &[0x3E, 0x50, 0xE0, 0x01, 0x3E, 0x81, 0xE0, 0x02, 0x18, 0xFE];
    let rom_bytes = rom(b"TEST", code);
    let path = write_rom(&dir, "teste.gb", &rom_bytes);
    let out = gb_cli(&[
        "run",
        path.to_str().unwrap(),
        "--headless",
        "--max-cycles",
        "50",
    ]);

    let text = stdout(&out);
    assert!(
        text.contains('P'),
        "a saída serial tem de aparecer em stdout\n{}",
        describe(&out)
    );
}

#[test]
fn run_exits_zero_when_serial_output_contains_passed() {
    let dir = sandbox("run-passed");
    // Programa que escreve "Passed\n" no SB, byte a byte, e depois entra em loop
    // Cada byte: LD A,char; LDH ($01),A; LD A,$81; LDH ($02),A
    let msg = b"Passed\n";
    let mut prog: Vec<u8> = Vec::new();
    for &byte in msg {
        prog.extend_from_slice(&[0x3E, byte, 0xE0, 0x01, 0x3E, 0x81, 0xE0, 0x02]);
    }
    prog.extend_from_slice(&[0x18, 0xFE]);
    let rom_bytes = rom(b"TEST", &prog);
    let path = write_rom(&dir, "teste.gb", &rom_bytes);
    let out = gb_cli(&[
        "run",
        path.to_str().unwrap(),
        "--headless",
        "--max-cycles",
        "200",
    ]);

    assert_eq!(
        code(&out),
        Some(0),
        "conteúdo com 'Passed' tem de sair com 0\n{}",
        describe(&out)
    );
    let text = stdout(&out);
    assert!(
        text.contains("Passed"),
        "a saída serial com 'Passed' tem de aparecer em stdout\n{}",
        describe(&out)
    );
}

#[test]
fn run_exits_one_when_serial_output_contains_failed() {
    let dir = sandbox("run-failed");
    let msg = b"Failed\n";
    let mut prog: Vec<u8> = Vec::new();
    for &byte in msg {
        prog.extend_from_slice(&[0x3E, byte, 0xE0, 0x01, 0x3E, 0x81, 0xE0, 0x02]);
    }
    prog.extend_from_slice(&[0x18, 0xFE]);
    let rom_bytes = rom(b"TEST", &prog);
    let path = write_rom(&dir, "teste.gb", &rom_bytes);
    let out = gb_cli(&[
        "run",
        path.to_str().unwrap(),
        "--headless",
        "--max-cycles",
        "200",
    ]);

    assert_eq!(
        code(&out),
        Some(1),
        "conteúdo com 'Failed' tem de sair com 1\n{}",
        describe(&out)
    );
}

#[test]
fn run_stops_at_max_cycles_even_if_rom_does_not_halt() {
    let dir = sandbox("run-para-no-max");
    let rom_bytes = rom(b"TEST", &[0x18, 0xFE]);
    let path = write_rom(&dir, "teste.gb", &rom_bytes);
    let out = gb_cli(&[
        "run",
        path.to_str().unwrap(),
        "--headless",
        "--max-cycles",
        "20",
    ]);

    let text = stdout(&out);
    let cycles_line = text
        .lines()
        .find(|l| l.starts_with("cycles="))
        .expect("a linha cycles= não foi encontrada");
    let n: u64 = cycles_line
        .strip_prefix("cycles=")
        .unwrap()
        .parse()
        .unwrap();
    assert!(
        n <= 20,
        "o laço deve parar quando atinge max_cycles (pediu 20, rodou {n})\n{}",
        describe(&out)
    );
}

#[test]
fn run_stops_when_cpu_encounters_lockup() {
    let dir = sandbox("run-para-no-lockup");
    // $FD é um opcode inexistente — o CPU vai para Lockup::IllegalOpcode
    let rom_bytes = rom(b"TEST", &[0xFD]);
    let path = write_rom(&dir, "teste.gb", &rom_bytes);
    let out = gb_cli(&[
        "run",
        path.to_str().unwrap(),
        "--headless",
        "--max-cycles",
        "1000000",
    ]);

    let text = stdout(&out);
    let cycles_line = text
        .lines()
        .find(|l| l.starts_with("cycles="))
        .expect("a linha cycles= não foi encontrada");
    let n: u64 = cycles_line
        .strip_prefix("cycles=")
        .unwrap()
        .parse()
        .unwrap();
    assert!(
        n < 1000000,
        "o laço deve parar no lockup, não esperar max_cycles\n{}",
        describe(&out)
    );
}

#[test]
fn run_stops_when_cpu_is_stopped() {
    let dir = sandbox("run-para-no-stop");
    // STOP ($10) coloca a CPU em Stopped
    let rom_bytes = rom(b"TEST", &[0x10, 0x00]);
    let path = write_rom(&dir, "teste.gb", &rom_bytes);
    let out = gb_cli(&[
        "run",
        path.to_str().unwrap(),
        "--headless",
        "--max-cycles",
        "1000000",
    ]);

    let text = stdout(&out);
    let cycles_line = text
        .lines()
        .find(|l| l.starts_with("cycles="))
        .expect("a linha cycles= não foi encontrada");
    let n: u64 = cycles_line
        .strip_prefix("cycles=")
        .unwrap()
        .parse()
        .unwrap();
    assert!(
        n < 1000000,
        "o laço deve parar no STOP, não esperar max_cycles\n{}",
        describe(&out)
    );
}
