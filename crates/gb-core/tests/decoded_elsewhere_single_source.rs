//! ROADMAP 0.6 — decoded_elsewhere/previously_decoded só em tests/support/mod.rs.

use std::path::{Path, PathBuf};

fn tests_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests")
}

fn rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("tests/ existe e é legível") {
        let path = entry.expect("entrada de diretório legível").path();
        if path.is_dir() {
            rs_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

fn defines_the_control(text: &str) -> bool {
    text.contains("fn decoded_elsewhere(")
        || text.contains("fn previously_decoded(")
        || text.contains("let previously_decoded")
        || text.contains("let decoded_elsewhere")
}

#[test]
fn the_negative_decode_control_is_defined_in_exactly_one_place() {
    let mut files = Vec::new();
    rs_files(&tests_dir(), &mut files);
    assert!(!files.is_empty(), "a varredura achou os testes de gb-core");

    // Esta busca procura o texto "fn decoded_elsewhere(" nos arquivos, e este
    // arquivo contém esse texto dentro de uma string literal — exclui a si
    // mesmo, ou nunca chegaria a 1.
    let this_file = Path::new(file!())
        .file_name()
        .expect("file!() tem nome de arquivo");

    let defining_files: Vec<PathBuf> = files
        .into_iter()
        .filter(|path| path.file_name() != Some(this_file))
        .filter(|path| {
            let text = std::fs::read_to_string(path).expect("arquivo .rs é UTF-8");
            defines_the_control(&text)
        })
        .collect();

    assert_eq!(
        defining_files,
        vec![tests_dir().join("support").join("mod.rs")],
        "ROADMAP 0.6: só tests/support/mod.rs pode definir o controle negativo"
    );
}
