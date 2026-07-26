//! Guardas das invariantes estruturais do `gb-core` — R3 e R6 do `CLAUDE.md`.

use std::path::{Path, PathBuf};

const ALLOWED_DEPENDENCIES: &[&str] = &[];

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn workspace_root() -> PathBuf {
    crate_root()
        .parent()
        .and_then(Path::parent)
        .expect("gb-core deve morar em <raiz>/crates/gb-core")
        .to_path_buf()
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("não consegui ler {}: {e}", path.display()))
}

fn names_in_section(manifest: &str, section: &str) -> Vec<String> {
    let header = format!("[{section}]");
    let table_prefix = format!("[{section}.");
    let mut names = Vec::new();
    let mut inside = false;

    for line in manifest.lines() {
        let line = line.trim();

        if line.starts_with('[') {
            if let Some(rest) = line.strip_prefix(&table_prefix) {
                // `[dependencies.foo]` → `foo`
                names.push(rest.trim_end_matches(']').trim().to_string());
            }
            inside = line == header;
            continue;
        }

        if !inside || line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((name, _)) = line.split_once('=') {
            names.push(name.trim().to_string());
        }
    }

    names
}

#[test]
fn section_parser_reads_inline_form() {
    let manifest = "\
[package]
name = \"gb-core\"

[dependencies]
# um comentário
alfa = \"1\"
beta = { version = \"2\" }

[dev-dependencies]
gama = \"3\"
";
    assert_eq!(names_in_section(manifest, "dependencies"), ["alfa", "beta"]);
    assert_eq!(names_in_section(manifest, "dev-dependencies"), ["gama"]);
}

#[test]
fn section_parser_reads_table_form() {
    let manifest = "\
[dependencies.alfa]
version = \"1\"

[dependencies]
beta = \"2\"
";
    assert_eq!(names_in_section(manifest, "dependencies"), ["alfa", "beta"]);
}

#[test]
fn section_parser_returns_empty_when_section_is_absent() {
    assert!(names_in_section("[package]\nname = \"x\"\n", "dependencies").is_empty());
}

#[test]
fn gb_core_has_no_unapproved_dependencies() {
    let manifest = read(&crate_root().join("Cargo.toml"));
    let intruders: Vec<_> = names_in_section(&manifest, "dependencies")
        .into_iter()
        .filter(|d| !ALLOWED_DEPENDENCIES.contains(&d.as_str()))
        .collect();

    assert!(
        intruders.is_empty(),
        "R3: gb-core é uma máquina de estados pura, sem I/O. \
         Dependências não autorizadas: {intruders:?}. \
         Se alguma delas é comprovadamente livre de I/O, adicione-a a \
         ALLOWED_DEPENDENCIES e justifique no doc da iteração."
    );
}

#[test]
fn gb_core_forbids_unsafe() {
    let lib = read(&crate_root().join("src/lib.rs"));
    assert!(
        lib.contains("#![forbid(unsafe_code)]"),
        "R6: crates/gb-core/src/lib.rs perdeu `#![forbid(unsafe_code)]`"
    );
}

#[test]
fn workspace_declares_the_three_crates() {
    let manifest = read(&workspace_root().join("Cargo.toml"));
    for name in ["gb-core", "gb-cli", "gb-desktop"] {
        let path = format!("crates/{name}");
        assert!(
            manifest.contains(&path),
            "workspace não declara o membro {path}"
        );
        assert!(
            workspace_root().join(&path).join("Cargo.toml").exists(),
            "{path}/Cargo.toml não existe"
        );
    }
}
