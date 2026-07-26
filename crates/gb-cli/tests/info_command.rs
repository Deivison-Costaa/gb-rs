//! ROADMAP 0.3b — `gb-cli info <rom>`: a casca de I/O do parser da 0005.
//!
//! O parser já está testado em `gb-core/tests/cart_header.rs` contra ROMs
//! sintéticas. O que **falta** testar é tudo que o `gb-core` não pode ter (R3):
//! ler arquivo, interpretar argumentos, escolher código de saída, imprimir.
//! Por isso estes testes executam o **binário**, e não uma função — código de
//! saída e stream de destino são o produto do item, e função nenhuma os expõe.
//!
//! Os códigos de saída seguem `sysexits.h` (`64` uso, `65` dado inválido,
//! `66` entrada ilegível) e a razão é o contrato do `scoreboard.sh`: `0` e `1`
//! pertencem ao veredito da ROM e `124` ao `timeout(1)`. Um `info` que saísse
//! `1` ao não achar o arquivo estaria dizendo "a ROM reprovou", que é falso e
//! acabaria no `scoreboard.csv` como medição.
//!
//! As ROMs aqui são **sintéticas**, montadas byte a byte, pelo mesmo motivo da
//! 0005: `tests/roms/` é gitignored e baixado por script, e um teste que
//! dependesse dele passaria vazio na máquina de quem não rodou o download —
//! o modo de falha da nota 8 do `STATUS.md`.
//!
//! `unwrap`/`expect` são permitidos aqui: a R6 proíbe fora de teste.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Menor ROM que contém o cabeçalho inteiro — `$014F` é o último byte dele.
const ROM_LEN: usize = 0x0150;

const TITLE: usize = 0x0134;
const CARTRIDGE_TYPE: usize = 0x0147;
const ROM_SIZE: usize = 0x0148;
const RAM_SIZE: usize = 0x0149;
const HEADER_CHECKSUM: usize = 0x014D;

/// `crates/gb-cli` → `crates` → raiz do workspace.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("gb-cli deve morar em <raiz>/crates/gb-cli")
        .to_path_buf()
}

/// Diretório exclusivo do caso de teste, sob `target/` para sair no
/// `cargo clean` e já estar no `.gitignore`.
fn sandbox(name: &str) -> PathBuf {
    let dir = workspace_root().join("target/tests-tmp").join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("limpar sandbox");
    }
    std::fs::create_dir_all(&dir).expect("criar sandbox");
    dir
}

/// O checksum como o boot ROM o calcula, transcrito da spec (Pan Docs § 014D):
///
/// ```c
/// uint8_t checksum = 0;
/// for (uint16_t address = 0x0134; address <= 0x014C; address++) {
///     checksum = checksum - rom[address] - 1;
/// }
/// ```
fn boot_rom_checksum(rom: &[u8]) -> u8 {
    let mut checksum: u8 = 0;
    for &byte in &rom[0x0134..=0x014C] {
        checksum = checksum.wrapping_sub(byte).wrapping_sub(1);
    }
    checksum
}

/// ROM sintética do tamanho mínimo, com título e códigos escolhidos, já com o
/// checksum que o boot ROM aceitaria gravado em `$014D`.
fn rom(title: &[u8], cartridge_type: u8, rom_size: u8, ram_size: u8) -> Vec<u8> {
    let mut rom = vec![0x00; ROM_LEN];
    rom[TITLE..TITLE + title.len()].copy_from_slice(title);
    rom[CARTRIDGE_TYPE] = cartridge_type;
    rom[ROM_SIZE] = rom_size;
    rom[RAM_SIZE] = ram_size;
    rom[HEADER_CHECKSUM] = boot_rom_checksum(&rom);
    rom
}

/// Grava a ROM no sandbox e devolve o caminho.
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

/// O conteúdo do campo `label:` da saída, sem o rótulo e sem o alinhamento.
///
/// Os testes afirmam o **conteúdo** dos campos, não a largura das colunas:
/// mudar o alinhamento é cosmético e não deveria pintar nada de vermelho;
/// mudar o que o campo diz é o que interessa.
fn field(text: &str, label: &str) -> String {
    let prefix = format!("{label}:");
    let line = text
        .lines()
        .find(|line| line.starts_with(&prefix))
        .unwrap_or_else(|| panic!("a saída não tem o campo `{label}`:\n{text}"));
    line[prefix.len()..].trim().to_string()
}

/// O código de saída como número, ou `None` se o processo morreu por sinal.
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

/// O relatório vai para **stdout**. Se fosse para stderr, `gb-cli info rom.gb >
/// relatorio.txt` gravaria arquivo vazio e o dado se perderia no terminal.
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

/// O caminho e o tamanho do arquivo são do `gb-cli`, não do cabeçalho — e são
/// eles que dizem *qual* ROM produziu o resto do relatório quando alguém rodar
/// isto sobre as 121 de `tests/roms/`.
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

/// Código fora da tabela **não** é erro: sai `0` e o relatório diz o que sabe.
/// Um `info` que se recusa a falar sobre a ROM esquisita é justamente o que não
/// serve para diagnosticar ROM esquisita.
#[test]
fn info_reports_an_unknown_cartridge_type_without_failing() {
    let dir = sandbox("info-tipo-desconhecido");
    let path = write_rom(&dir, "estranha.gb", &rom(b"ESTRANHA", 0x04, 0x00, 0x00));
    let out = gb_cli(&["info", path.to_str().unwrap()]);

    assert_eq!(code(&out), Some(0), "{}", describe(&out));
    assert_eq!(field(&stdout(&out), "tipo"), "$04 desconhecido");
}

/// A RAM `$01` é o erro #1 da 0005 visto pela ponta que o usuário lê. O Pan
/// Docs diz "Unused"; meio mundo documenta 2 KiB. Se `2 KiB` aparecer aqui, a
/// invariante vazou do `gb-core` para o texto impresso — e texto impresso é
/// lido como medição.
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

/// `$52` é um dos três tamanhos de ROM que a spec só vê em documentação não
/// oficial e considera provavelmente errados.
#[test]
fn info_reports_an_unattested_rom_size_as_unknown() {
    let dir = sandbox("info-rom-52");
    let path = write_rom(&dir, "estranha.gb", &rom(b"ESTRANHA", 0x00, 0x52, 0x00));
    let out = gb_cli(&["info", path.to_str().unwrap()]);

    assert_eq!(code(&out), Some(0), "{}", describe(&out));
    assert_eq!(field(&stdout(&out), "ROM"), "$52 tamanho desconhecido");
}

/// Checksum que não bate trava o boot ROM — mas é exatamente a ROM que alguém
/// quer diagnosticar. Relatar, e sair `0`: o veredito é do leitor.
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

/// ROM que acaba dentro do cabeçalho: `HeaderError::TooShort`. Sai `65`
/// (`EX_DATAERR`) — o arquivo existe e foi lido, o conteúdo é que não serve.
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

/// Arquivo inexistente: `66` (`EX_NOINPUT`). Distinguir isto de `65` é o que
/// separa "errei o caminho" de "a ROM está corrompida".
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

/// Diretório no lugar de arquivo é o mesmo tipo de engano de caminho — e o
/// `read` falha com `IsADirectory`, não com `NotFound`. Se o código de saída
/// dependesse da variante do erro, este caso escaparia.
#[test]
fn directory_instead_of_rom_exits_with_no_input() {
    let dir = sandbox("info-diretorio");
    let out = gb_cli(&["info", dir.to_str().unwrap()]);

    assert_eq!(code(&out), Some(66), "{}", describe(&out));
}

// ---------------------------------------------------------------------------
// Argumentos
// ---------------------------------------------------------------------------

/// `64` é `EX_USAGE`. Sem argumento nenhum não há o que fazer, e sair `0`
/// deixaria um `gb-cli` mudo passando por sucesso em qualquer script.
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

/// Argumento a mais é engano de quem chamou — provavelmente um glob que casou
/// com duas ROMs. Aceitar em silêncio e ler só a primeira esconde o engano.
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

/// Contrato do `scoreboard.sh`: `run` ainda não existe (é o ROADMAP 1.12) e sai
/// `2`, que o script classifica como `crash`. Se esta iteração tivesse
/// transformado o `2` em `64` ao reescrever o despacho de argumentos, as 121
/// linhas do CSV mudariam de categoria sem que nada no emulador tivesse mudado.
#[test]
fn run_is_still_unimplemented_and_exits_two() {
    let out = gb_cli(&["run", "tetris.gb", "--headless", "--max-cycles", "1000"]);

    assert_eq!(
        code(&out),
        Some(2),
        "o `run` é do 1.12; `2` é o `crash` que o scoreboard espera\n{}",
        describe(&out)
    );
}
