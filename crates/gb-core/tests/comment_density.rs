use std::path::{Path, PathBuf};

const MAX_PERCENT: usize = 12;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("gb-core mora dois níveis abaixo da raiz do workspace")
        .to_path_buf()
}

fn rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("crates/ existe e é legível") {
        let path = entry.expect("entrada de diretório legível").path();
        if path.is_dir() {
            rs_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

/// (doc, inner, blank, total) — a classificação da fórmula compartilhada.
fn count(path: &Path) -> (usize, usize, usize, usize) {
    let text = std::fs::read_to_string(path).expect("arquivo .rs é UTF-8");
    let mut counts = (0, 0, 0, 0);
    for line in text.lines() {
        let t = line.trim_start();
        if t.is_empty() {
            counts.2 += 1;
        } else if t.starts_with("///") || t.starts_with("//!") {
            counts.0 += 1;
        } else if t.starts_with("//") || t.starts_with("/*") || t.starts_with('*') {
            counts.1 += 1;
        }
        counts.3 += 1;
    }
    counts
}

#[test]
fn no_rs_file_exceeds_the_r7_comment_ceiling() {
    let mut files = Vec::new();
    rs_files(&workspace_root().join("crates"), &mut files);
    assert!(
        !files.is_empty(),
        "a varredura achou os fontes do workspace"
    );

    let mut offenders = Vec::new();
    for path in &files {
        let (doc, inner, blank, total) = count(path);
        let code = total - blank - doc - inner;
        let cmt = doc + inner;
        if code + cmt > 0 && cmt * 100 > MAX_PERCENT * (code + cmt) {
            offenders.push(format!(
                "{}: {cmt}/{} = {}%",
                path.strip_prefix(workspace_root())
                    .unwrap_or(path)
                    .display(),
                code + cmt,
                cmt * 100 / (code + cmt),
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "R7: {} arquivo(s) acima do teto de {MAX_PERCENT}% de comentário:\n  {}",
        offenders.len(),
        offenders.join("\n  "),
    );
}
