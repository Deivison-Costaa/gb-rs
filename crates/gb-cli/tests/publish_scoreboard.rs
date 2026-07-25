//! Guarda comportamental do `scripts/publish-scoreboard.sh` — ROADMAP 0.2c.
//!
//! O `scoreboard.sh` faz a série crescer **dentro do runner**. Quando o job
//! acaba, o runner é descartado e as linhas que a CI produziu somem: o que
//! sobrevive no git é só o que uma iteração commitou à mão. O artefato não
//! resolve isso — ele guarda o CSV por 90 dias em um zip por execução, não
//! monta a série.
//!
//! Este script fecha o ciclo publicando o CSV acumulado numa branch de dados.
//! **Não em `main`:** a proteção de `main` exige PR, e o `GITHUB_TOKEN` não tem
//! bypass (ver `docs/iterations/0004-ci-serie-persistida.md`).
//!
//! Os testes montam um repositório git de mentira — um bare fazendo de
//! `origin`, um clone de trabalho — dentro de `target/tests-tmp/`. **Nunca**
//! tocam o repositório real nem o `scoreboard.csv` versionado: publicar linhas
//! de teste na branch de dados corromperia o dado da apresentação.
//!
//! `unwrap`/`expect` são permitidos aqui: R6 proíbe fora de teste.

#![cfg(unix)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const CSV_HEADER: &str = "timestamp,commit,suite,rom,status,ciclos";
const DATA_BRANCH: &str = "scoreboard-data";

/// `crates/gb-cli` → `crates` → raiz do workspace.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("gb-cli deve morar em <raiz>/crates/gb-cli")
        .to_path_buf()
}

/// Diretório exclusivo do caso de teste, sempre zerado antes de usar. Mora sob
/// `target/` para sair no `cargo clean` e já estar no `.gitignore`.
fn sandbox(name: &str) -> PathBuf {
    let dir = workspace_root().join("target/tests-tmp").join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("limpar sandbox");
    }
    std::fs::create_dir_all(&dir).expect("criar sandbox");
    dir
}

fn describe(out: &Output) -> String {
    format!(
        "saída {:?}\n--- stdout ---\n{}\n--- stderr ---\n{}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    )
}

/// Roda `git` no diretório dado e exige sucesso.
fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("executar git {args:?}: {e}"));
    assert!(
        out.status.success(),
        "git {args:?} em {} falhou\n{}",
        dir.display(),
        describe(&out)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// Uma linha de dado plausível do CSV. O conteúdo não importa para o script —
/// ele trata a linha como opaca —, mas linhas distintas é o que deixa ver quem
/// sobreviveu à publicação.
fn row(n: u32) -> String {
    format!("2026-07-25T00:00:{n:02}Z,abc123,\"blargg/cpu_instrs\",\"{n:02}.gb\",crash,0")
}

fn csv_with(rows: &[String]) -> String {
    let mut text = String::from(CSV_HEADER);
    for r in rows {
        text.push('\n');
        text.push_str(r);
    }
    text.push('\n');
    text
}

/// Repositório de mentira: um bare (`origin`) e um clone de trabalho com um
/// commit inicial em `main` e um `scoreboard.csv` com as linhas dadas.
struct Repo {
    remote: PathBuf,
    work: PathBuf,
}

impl Repo {
    fn new(dir: &Path, rows: &[String]) -> Self {
        let remote = dir.join("remote.git");
        let work = dir.join("work");
        std::fs::create_dir_all(&remote).expect("criar remote");
        std::fs::create_dir_all(&work).expect("criar work");

        git(&remote, &["init", "--bare", "--initial-branch=main", "-q"]);

        git(&work, &["init", "--initial-branch=main", "-q"]);
        git(&work, &["config", "user.name", "Teste"]);
        git(&work, &["config", "user.email", "teste@exemplo.invalido"]);
        git(
            &work,
            &["remote", "add", "origin", remote.to_str().expect("utf-8")],
        );
        std::fs::write(work.join("README.md"), b"repo de teste\n").expect("escrever README");
        git(&work, &["add", "README.md"]);
        git(&work, &["commit", "-qm", "inicial"]);
        git(&work, &["push", "-q", "origin", "main"]);

        let repo = Self { remote, work };
        repo.write_csv(rows);
        repo
    }

    fn write_csv(&self, rows: &[String]) {
        std::fs::write(self.work.join("scoreboard.csv"), csv_with(rows)).expect("escrever CSV");
    }

    fn publish(&self) -> Output {
        Command::new(workspace_root().join("scripts/publish-scoreboard.sh"))
            .current_dir(&self.work)
            .env("DATA_BRANCH", DATA_BRANCH)
            .env("DATA_REMOTE", "origin")
            // Identidade explícita: o script cria o commit com `git commit-tree`,
            // que exige autor. No runner quem fornece é o ambiente do Actions.
            .env("GIT_AUTHOR_NAME", "Teste")
            .env("GIT_AUTHOR_EMAIL", "teste@exemplo.invalido")
            .output()
            .expect("executar scripts/publish-scoreboard.sh")
    }

    /// O conteúdo do `scoreboard.csv` **no remoto**, que é o que interessa:
    /// o que ficou no clone de trabalho não sobrevive ao fim do job.
    fn published_csv(&self) -> String {
        git(
            &self.remote,
            &["show", &format!("{DATA_BRANCH}:scoreboard.csv")],
        )
    }

    fn published_rows(&self) -> Vec<String> {
        self.published_csv()
            .lines()
            .skip(1)
            .filter(|l| !l.trim().is_empty())
            .map(str::to_string)
            .collect()
    }

    fn published_commits(&self) -> usize {
        git(&self.remote, &["rev-list", "--count", DATA_BRANCH])
            .parse()
            .expect("contagem de commits")
    }

    fn branch_exists(&self) -> bool {
        Command::new("git")
            .current_dir(&self.remote)
            .args(["rev-parse", "--verify", "--quiet", DATA_BRANCH])
            .output()
            .expect("git rev-parse")
            .status
            .success()
    }

    /// Semeia a branch de dados **por fora do script**, com porcelana comum.
    ///
    /// De propósito não usa o mesmo `hash-object`/`commit-tree` do script: se o
    /// mecanismo do script estiver quebrado, semear com ele quebraria junto e o
    /// teste passaria por vacuidade.
    fn seed_data_branch(&self, dir: &Path, rows: &[String]) {
        let seed = dir.join("seed");
        git(
            dir,
            &[
                "clone",
                "-q",
                self.remote.to_str().expect("utf-8"),
                seed.to_str().expect("utf-8"),
            ],
        );
        git(&seed, &["config", "user.name", "Outro Runner"]);
        git(&seed, &["config", "user.email", "outro@exemplo.invalido"]);
        git(&seed, &["switch", "-q", "--orphan", DATA_BRANCH]);
        std::fs::write(seed.join("scoreboard.csv"), csv_with(rows)).expect("escrever CSV semente");
        git(&seed, &["add", "scoreboard.csv"]);
        git(&seed, &["commit", "-qm", "semente de outra execução"]);
        git(&seed, &["push", "-q", "origin", DATA_BRANCH]);
    }

    /// Instala um `pre-receive` que reprova o primeiro push e aceita os
    /// seguintes — o jeito determinístico de encenar a corrida entre duas
    /// execuções da CI.
    fn reject_first_push(&self) {
        use std::os::unix::fs::PermissionsExt;
        let hook = self.remote.join("hooks/pre-receive");
        std::fs::create_dir_all(self.remote.join("hooks")).expect("criar hooks/");
        std::fs::write(
            &hook,
            "#!/bin/sh\n\
             cat >/dev/null\n\
             if [ ! -e ./rejeitei-uma-vez ]; then\n\
             \t: > ./rejeitei-uma-vez\n\
             \techo 'rejeitando de proposito na primeira tentativa (teste)' >&2\n\
             \texit 1\n\
             fi\n\
             exit 0\n",
        )
        .expect("escrever hook");
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755))
            .expect("tornar o hook executável");
    }
}

// --- os casos -------------------------------------------------------------

/// O caso base: a branch não existe, e a primeira execução a cria com a série.
#[test]
fn publish_creates_the_data_branch_on_the_first_run() {
    let dir = sandbox("publish-cria-branch");
    let repo = Repo::new(&dir, &[row(1), row(2)]);
    assert!(!repo.branch_exists(), "a branch não deveria existir ainda");

    let out = repo.publish();
    assert!(out.status.success(), "{}", describe(&out));

    assert!(repo.branch_exists(), "a branch de dados não foi criada");
    assert_eq!(
        repo.published_rows(),
        vec![row(1), row(2)],
        "as linhas publicadas não são as do CSV local\n{}",
        describe(&out)
    );
    assert!(
        repo.published_csv().starts_with(CSV_HEADER),
        "o CSV publicado perdeu o cabeçalho"
    );
}

/// O caso que dá sentido ao resto: **nada do que já está publicado pode sumir**.
///
/// Encena outra execução da CI que publicou uma linha que este runner não tem
/// (o clone dele é de um commit anterior). Sobrescrever a branch com o CSV
/// local seria trocar a série por um pedaço dela — perda silenciosa, e é a
/// série inteira que vira gráfico no ROADMAP 8.2.
#[test]
fn publish_keeps_rows_that_another_run_had_already_published() {
    let dir = sandbox("publish-preserva-alheio");
    let repo = Repo::new(&dir, &[row(1), row(2)]);
    repo.seed_data_branch(&dir, &[row(1), row(9)]);

    let out = repo.publish();
    assert!(out.status.success(), "{}", describe(&out));

    let published = repo.published_rows();
    assert!(
        published.contains(&row(9)),
        "a linha publicada por outra execução sumiu: {published:?}\n{}",
        describe(&out)
    );
    assert_eq!(
        published,
        vec![row(1), row(9), row(2)],
        "esperava o que já estava publicado, na ordem, mais o que é novo\n{}",
        describe(&out)
    );
}

/// Publicar duas vezes a mesma coisa não gera commit novo.
///
/// Sem isso, todo push em `main` que não mudasse o placar ainda assim empilharia
/// um commit vazio de conteúdo na branch de dados — e um `rev-list` deixaria de
/// dizer quantas execuções mediram alguma coisa.
#[test]
fn publish_is_a_no_op_when_there_is_nothing_new() {
    let dir = sandbox("publish-idempotente");
    let repo = Repo::new(&dir, &[row(1), row(2)]);

    let first = repo.publish();
    assert!(first.status.success(), "{}", describe(&first));
    assert_eq!(repo.published_commits(), 1);

    let second = repo.publish();
    assert!(second.status.success(), "{}", describe(&second));
    assert_eq!(
        repo.published_commits(),
        1,
        "a segunda publicação criou commit sem ter linha nova\n{}",
        describe(&second)
    );
    assert_eq!(repo.published_rows(), vec![row(1), row(2)]);
}

/// Duas execuções da CI podem terminar juntas; a segunda a empurrar leva um
/// push rejeitado. Desistir aí perderia a medição — o script tem de refazer
/// sobre o topo novo e tentar de novo.
#[test]
fn publish_retries_when_the_push_is_rejected() {
    let dir = sandbox("publish-retenta");
    let repo = Repo::new(&dir, &[row(1)]);
    repo.reject_first_push();

    let out = repo.publish();
    assert!(
        out.status.success(),
        "o script desistiu no primeiro push rejeitado\n{}",
        describe(&out)
    );
    assert_eq!(repo.published_rows(), vec![row(1)]);
}

/// CSV só com cabeçalho é o mesmo modo de falha da 0.2b visto daqui: publicar
/// "zero linhas" com sucesso é afirmar que mediu e estava tudo bem.
#[test]
fn publish_fails_when_the_csv_has_no_data_rows() {
    let dir = sandbox("publish-csv-vazio");
    let repo = Repo::new(&dir, &[]);

    let out = repo.publish();
    assert!(
        !out.status.success(),
        "publicou um CSV sem nenhuma linha de dado e saiu com sucesso\n{}",
        describe(&out)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("nenhuma linha de dado"),
        "falhou, mas não por ter detectado o CSV vazio\n{}",
        describe(&out)
    );
    assert!(
        !repo.branch_exists(),
        "criou a branch de dados mesmo sem ter o que publicar"
    );
}
