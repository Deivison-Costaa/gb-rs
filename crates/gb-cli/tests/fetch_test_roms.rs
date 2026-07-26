//! Guarda comportamental do `scripts/fetch-test-roms.sh` — ROADMAP 0.5.
//!
//! O script já existia desde o scaffold e a CI o roda a cada push desde a 0002.
//! O que faltava era guarda: dos quatro scripts do projeto, era o único sem
//! teste, e ele é o que decide **o que existe para medir**. Um `fetch` que
//! entrega menos ROMs do que prometeu não pinta nada de vermelho — o
//! `scoreboard.sh` mede o que achou, sai `0`, e a série encolhe sem sinal
//! nenhum. É o mesmo modo de falha da 0.2b um degrau acima: lá o placar parava
//! de crescer, aqui ele passa a medir um universo menor com a mesma cara.
//!
//! Os testes rodam o script de verdade contra um **bundle falso local**,
//! servido por `file://`. Sem rede: teste que depende de release do GitHub
//! falha por motivo alheio ao que mede, e falha justamente quando a CI está com
//! pressa. O par `TEST_ROMS_BUNDLE_URL`/`TEST_ROMS_BUNDLE_SHA256` existe no
//! script para isso — e `ci_does_not_override_the_pinned_bundle` guarda que ele
//! continue sendo costura de teste, não porta dos fundos da CI.
//!
//! O bundle falso é montado com `zip`. Se ele faltar, o teste **falha** em vez
//! de pular: suíte que se desliga sozinha quando o ambiente encolhe é a
//! vacuidade da nota 8 do `STATUS.md` com outro nome. O script já exige
//! `curl`/`unzip`/`tar`/`sha256sum`; exigir `zip` para testá-lo é o mesmo
//! contrato.
//!
//! `unwrap`/`expect` são permitidos aqui: R6 proíbe fora de teste.

#![cfg(unix)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// O conteúdo do bundle falso, no layout que o `unzip` do script espera:
/// `blargg/*`, `mooneye-test-suite/acceptance/*` e `dmg-acid2/*`.
///
/// As oito suítes blargg exigidas aparecem para o `verify()` do script não ter
/// do que reclamar — o que se mede aqui é o transporte, não o aviso. Duas
/// entradas não são enfeite:
///
/// - `blargg/cgb_sound/` é o intruso. Vem no bundle real, é suíte de Game Boy
///   Color, e tem de ser podada (nota 6 do `STATUS.md`).
/// - o `-marker` no nome é o que distingue "o script instalou o bundle que eu
///   mandei" de "o script baixou o bundle real da internet". Sem ele, o teste
///   passaria contra a release verdadeira e não mediria a costura nenhuma.
const FAKE_BUNDLE_FILES: &[&str] = &[
    "blargg/cpu_instrs/cpu_instrs.gb",
    "blargg/cpu_instrs/individual/01-marker.gb",
    "blargg/instr_timing/instr_timing.gb",
    "blargg/mem_timing/mem_timing.gb",
    "blargg/mem_timing-2/mem_timing.gb",
    "blargg/oam_bug/oam_bug.gb",
    "blargg/interrupt_time/interrupt_time.gb",
    "blargg/dmg_sound/dmg_sound.gb",
    "blargg/halt_bug.gb",
    "blargg/cgb_sound/cgb_sound-marker.gb",
    "mooneye-test-suite/acceptance/boot_regs-marker.gb",
    "dmg-acid2/dmg-acid2.gb",
];

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
/// `.gitignore` — e para **nunca** encostar em `tests/roms/`, que é o que a
/// máquina de quem roda o teste tem de verdade.
fn sandbox(name: &str) -> PathBuf {
    let dir = workspace_root().join("target/tests-tmp").join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("limpar sandbox");
    }
    std::fs::create_dir_all(&dir).expect("criar sandbox");
    dir
}

/// Caminho de arquivo → URL `file://` que o curl aceita.
///
/// Não é preciosismo: o repositório pode estar em qualquer lugar, e este aqui
/// mora sob `Área de trabalho/Programação com Agentes/`. Espaço cru em URL faz
/// o curl 8 recusar com `URL rejected: Malformed input to a URL function`
/// antes de tocar no disco — e o teste morreria por causa do caminho do
/// checkout, não do script.
fn file_url(path: &Path) -> String {
    const UNRESERVED: &[u8] = b"-._~/";
    let mut url = String::from("file://");
    for byte in path.as_os_str().as_encoded_bytes() {
        if byte.is_ascii_alphanumeric() || UNRESERVED.contains(byte) {
            url.push(*byte as char);
        } else {
            url.push_str(&format!("%{byte:02X}"));
        }
    }
    url
}

/// Monta o bundle falso e devolve `(url, sha256)` prontos para o script.
fn fake_bundle(dir: &Path) -> (String, String) {
    let src = dir.join("bundle-src");
    for entry in FAKE_BUNDLE_FILES {
        let path = src.join(entry);
        std::fs::create_dir_all(path.parent().expect("ROM falsa tem diretório pai"))
            .expect("criar diretório do bundle falso");
        // O conteúdo é o próprio caminho: se alguma ROM chegar trocada de
        // lugar, a mensagem de erro diz de onde ela veio.
        std::fs::write(&path, entry.as_bytes()).expect("criar ROM falsa");
    }

    let zip = dir.join("game-boy-test-roms-falso.zip");
    let status = Command::new("zip")
        .arg("-q")
        .arg("-r")
        .arg(&zip)
        .args(["blargg", "mooneye-test-suite", "dmg-acid2"])
        .current_dir(&src)
        .status()
        .expect("executar `zip` — este teste exige o utilitário `zip` no PATH");
    assert!(status.success(), "`zip` falhou ao montar o bundle falso");

    (file_url(&zip), sha256_of(&zip))
}

fn sha256_of(path: &Path) -> String {
    let out = Command::new("sha256sum")
        .arg(path)
        .output()
        .expect("executar `sha256sum`");
    assert!(out.status.success(), "`sha256sum` falhou em {path:?}");
    String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .next()
        .expect("sha256sum imprime <hash> <espaço> <caminho>")
        .to_string()
}

/// Roda o `fetch-test-roms.sh` com tudo apontado para dentro do sandbox.
fn run_fetch(dir: &Path, url: &str, sha256: &str) -> Output {
    Command::new(workspace_root().join("scripts/fetch-test-roms.sh"))
        .env("ROMS_DIR", dir.join("roms"))
        .env("ROM_CACHE_DIR", dir.join("cache"))
        .env("TEST_ROMS_BUNDLE_URL", url)
        .env("TEST_ROMS_BUNDLE_SHA256", sha256)
        .output()
        .expect("executar scripts/fetch-test-roms.sh")
}

fn describe(out: &Output) -> String {
    format!(
        "saída {:?}\n--- stdout ---\n{}\n--- stderr ---\n{}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    )
}

/// ROADMAP 0.5, ao pé da letra: as três suítes chegam em `tests/roms/`.
///
/// O item cita blargg, mooneye e dmg-acid2 nominalmente. Cada uma é conferida
/// por uma ROM que só existe no bundle falso: assim o teste não pode passar
/// porque o script baixou a release verdadeira da internet.
#[test]
fn fetch_lands_the_three_suites_under_the_roms_dir() {
    let dir = sandbox("fetch-tres-suites");
    let (url, sha256) = fake_bundle(&dir);
    let out = run_fetch(&dir, &url, &sha256);

    assert!(
        out.status.success(),
        "o download falhou\n{}",
        describe(&out)
    );

    let roms = dir.join("roms");
    for (suite, rom) in [
        ("blargg", "blargg/cpu_instrs/individual/01-marker.gb"),
        ("mooneye", "mooneye/acceptance/boot_regs-marker.gb"),
        ("dmg-acid2", "dmg-acid2/dmg-acid2.gb"),
    ] {
        assert!(
            roms.join(rom).is_file(),
            "ROADMAP 0.5: a suíte {suite} não chegou em tests/roms/ — faltou {rom}\n{}",
            describe(&out)
        );
    }

    assert_eq!(
        std::fs::read_to_string(roms.join(".bundle-version"))
            .expect("o carimbo de versão tem de existir")
            .trim(),
        pinned_bundle_version(),
        "o carimbo de versão não é o do bundle instalado — a próxima execução \
         rebaixaria tudo, ou pior, daria por bom um bundle que não é este"
    );
}

/// O carimbo grava a **versão** fixada no script, não o sha do bundle: é ela
/// que `already_current` compara. Ler do próprio script evita que este teste
/// precise ser editado a cada bump de release.
fn pinned_bundle_version() -> String {
    let script = std::fs::read_to_string(workspace_root().join("scripts/fetch-test-roms.sh"))
        .expect("ler scripts/fetch-test-roms.sh");
    script
        .lines()
        .find_map(|l| l.strip_prefix("readonly BUNDLE_VERSION="))
        .map(|v| v.trim().trim_matches('"').to_string())
        .expect("scripts/fetch-test-roms.sh declara BUNDLE_VERSION")
}

/// Nota 6 do `STATUS.md`: `blargg/cgb_sound` é suíte de Game Boy Color e este
/// emulador é DMG. Ela vem no bundle real e tem de ser podada.
///
/// Sem esta guarda, o modo de falha é discreto: 13 ROMs a mais no
/// `scoreboard.csv`, falhando para sempre, que ninguém consegue distinguir de
/// regressão do emulador — e a série da apresentação passa a ter um degrau que
/// não corresponde a evento nenhum.
#[test]
fn cgb_sound_is_pruned_because_this_emulator_is_dmg() {
    let dir = sandbox("fetch-poda-cgb");
    let (url, sha256) = fake_bundle(&dir);
    let out = run_fetch(&dir, &url, &sha256);

    assert!(
        out.status.success(),
        "o download falhou\n{}",
        describe(&out)
    );

    let intruder = dir.join("roms/blargg/cgb_sound");
    assert!(
        !intruder.exists(),
        "cgb_sound sobreviveu à poda: são 13 falhas permanentes de CGB \
         entrando no placar de um emulador DMG\n{}",
        describe(&out)
    );
    // Controle: a poda tem de ser cirúrgica. Um `rm -rf` largo demais também
    // faria a asserção acima passar — e levaria as suítes de verdade junto.
    assert!(
        dir.join("roms/blargg/dmg_sound/dmg_sound.gb").is_file(),
        "a poda levou dmg_sound junto\n{}",
        describe(&out)
    );
    assert!(
        dir.join("roms/blargg/halt_bug.gb").is_file(),
        "a poda levou halt_bug.gb junto — é ROM solta na raiz de blargg/, \
         não diretório, e é fácil de perder num filtro por nome de suíte\n{}",
        describe(&out)
    );
}

/// O sha256 fixado é promessa de que o chão não muda por baixo do placar. Se
/// ele não for conferido de fato, a release pode ser alterada upstream e o
/// scoreboard passa a medir outro conjunto de ROMs com o mesmo nome.
///
/// Exige a mensagem, e não só o código de saída: o script tem um caminho de
/// fallback logo ali, e "falhou" e "caiu no plano B" são coisas diferentes que
/// um teste só de código de saída confundiria.
#[test]
fn a_bundle_whose_sha256_does_not_match_is_refused() {
    let dir = sandbox("fetch-sha-errado");
    let (url, _real) = fake_bundle(&dir);
    let wrong = "0".repeat(64);
    let out = run_fetch(&dir, &url, &wrong);

    assert!(
        !out.status.success(),
        "o script aceitou um bundle cujo sha256 não confere — a fixação é \
         decorativa\n{}",
        describe(&out)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("sha256"),
        "falhou, mas não por ter detectado o sha256 divergente\n{}",
        describe(&out)
    );
    assert!(
        !dir.join("roms/dmg-acid2/dmg-acid2.gb").exists(),
        "instalou o bundle recusado assim mesmo\n{}",
        describe(&out)
    );
}

/// O cabeçalho do script promete "no-op se já estiver lá" — e a CI depende
/// disso a cada push.
///
/// A prova é remover o bundle antes da segunda execução: se ela ainda assim
/// terminar bem, é porque não tentou baixar nada.
#[test]
fn a_second_run_is_a_noop() {
    let dir = sandbox("fetch-idempotente");
    let (url, sha256) = fake_bundle(&dir);

    let first = run_fetch(&dir, &url, &sha256);
    assert!(
        first.status.success(),
        "a primeira execução falhou\n{}",
        describe(&first)
    );

    std::fs::remove_dir_all(dir.join("bundle-src")).expect("remover a origem do bundle");
    std::fs::remove_file(dir.join("game-boy-test-roms-falso.zip")).expect("remover o bundle");
    std::fs::remove_dir_all(dir.join("cache")).expect("remover o cache");

    let second = run_fetch(&dir, &url, &sha256);
    assert!(
        second.status.success(),
        "a segunda execução tentou baixar de novo, com o bundle já fora do ar\n{}",
        describe(&second)
    );
    assert!(
        String::from_utf8_lossy(&second.stdout).contains("Nada a fazer"),
        "a segunda execução não reconheceu que as ROMs já estavam lá\n{}",
        describe(&second)
    );
}

/// A costura de teste não pode virar porta dos fundos.
///
/// `TEST_ROMS_BUNDLE_URL`/`_SHA256` existem para este arquivo apontar o
/// download para um zip local. Se a CI passasse a defini-los, a fixação por
/// tag e sha256 deixaria de valer para a execução que produz o placar de
/// verdade — e ninguém notaria, porque tudo continuaria verde.
#[test]
fn ci_does_not_override_the_pinned_bundle() {
    let workflow = std::fs::read_to_string(workspace_root().join(".github/workflows/ci.yml"))
        .expect("ler .github/workflows/ci.yml");
    assert!(
        !workflow.contains("TEST_ROMS_BUNDLE_"),
        "o workflow de CI está sobrescrevendo o bundle fixado — a costura de \
         teste virou configuração de produção"
    );
}
