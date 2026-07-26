use std::path::{Path, PathBuf};

const MAX_BYTES: usize = 24_000;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("gb-core mora dois níveis abaixo da raiz do workspace")
        .to_path_buf()
}

#[test]
fn status_stays_small_enough_to_load_every_turn() {
    let path = workspace_root().join("STATUS.md");
    let size = std::fs::read_to_string(&path)
        .expect("STATUS.md é UTF-8 e legível")
        .len();

    assert!(
        size <= MAX_BYTES,
        "STATUS.md está com {size} bytes, acima do teto de {MAX_BYTES}.\n\
         Ele entra em contexto em toda iteração, então cada byte é pago vezes o \
         número de turnos: a 107 KB custava ~US$ 0,65 por iteração só de leitura \
         de cache.\n\
         Corpo de nota vai para docs/notas.md, corpo de invariante para \
         docs/invariantes.md, e no STATUS.md fica uma linha de índice.",
    );
}

#[test]
fn the_bodies_live_outside_status() {
    for nome in ["docs/notas.md", "docs/invariantes.md"] {
        assert!(
            workspace_root().join(nome).is_file(),
            "{nome} sumiu — se o corpo voltou para o STATUS.md, o teto acima só \
             adia o problema. Ver docs/orquestracao.md.",
        );
    }
}
