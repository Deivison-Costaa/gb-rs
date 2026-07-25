//! Guarda dos jobs do workflow de CI — ROADMAP 0.2a e 0.2b.
//!
//! A proteção de `main` exige que o job `check` fique verde. Isso garante que
//! ele **rodou**, não que ele **verificou alguma coisa**: um passo com um `if:`
//! falso é pulado e o job termina verde do mesmo jeito. Foi exatamente essa a
//! situação criada pelo bootstrap — os três passos de qualidade ficaram atrás
//! de uma guarda `[ -f Cargo.toml ]` que, se um dia voltasse a dar falso,
//! desligaria fmt, clippy e testes **sem pintar nada de vermelho**.
//!
//! Este teste mora aqui, e não no próprio workflow, de propósito: guarda que
//! vive dentro da coisa guardada some junto com ela. Rodando dentro de
//! `cargo test --all`, ele reprova o PR que mexer no `ci.yml` para pior.
//!
//! Mora no `gb-cli` porque é o crate de ferramental do projeto — o que já
//! carrega o contrato com `scripts/scoreboard.sh`. O `gb-core` é a máquina de
//! estados pura e não deve saber que existe CI.
//!
//! `unwrap`/`expect` são permitidos aqui: R6 proíbe fora de teste.

use std::path::{Path, PathBuf};

/// O que o job `check` tem de rodar, como (rótulo, fragmentos obrigatórios).
///
/// Fragmentos, e não a linha de comando inteira, para o teste reprovar o que
/// importa — clippy sem `-D warnings` é clippy decorativo — sem quebrar por
/// uma flag nova qualquer. Mudou o comando de verdade? Mude aqui também, e
/// que seja uma decisão, não um acidente.
const REQUIRED_STEPS: &[(&str, &[&str])] = &[
    ("formatação", &["cargo fmt", "--check"]),
    ("clippy", &["cargo clippy", "-D warnings"]),
    ("testes", &["cargo test"]),
];

/// Os passos do job `scoreboard` cujo veredito **é** o veredito do job.
///
/// Se qualquer um destes falhar sem derrubar o job, o placar da apresentação
/// para de crescer em silêncio — o CSV do artefato passa a ser o de ontem e
/// nada fica vermelho.
///
/// Os fragmentos incluem o `run: ` e o `./` de propósito. `scoreboard.sh` sozinho
/// é substring de `publish-scoreboard.sh` (ROADMAP 0.2c), e `find` devolve o
/// **primeiro** passo que casa: bastaria alguém reordenar os passos para a
/// guarda passar a examinar o passo errado, sem nada ficar vermelho. Guarda que
/// depende da ordem do arquivo que ela guarda não é guarda.
const SCOREBOARD_STEPS: &[(&str, &[&str])] = &[
    ("download das ROMs", &["run: ./scripts/fetch-test-roms.sh"]),
    ("execução do placar", &["run: ./scripts/scoreboard.sh"]),
];

/// ROADMAP 0.2c — o passo que publica a série na branch de dados.
const PUBLISH_STEP: &[&str] = &["run: ./scripts/publish-scoreboard.sh"];

/// `crates/gb-cli` → `crates` → raiz do workspace.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("gb-cli deve morar em <raiz>/crates/gb-cli")
        .to_path_buf()
}

fn read_workflow() -> String {
    let path = workspace_root().join(".github/workflows/ci.yml");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("não consegui ler {}: {e}", path.display()))
}

/// Um passo de um job, já sem a indentação do bloco: as chaves do passo
/// (`run:`, `if:`, `uses:`) ficam na coluna zero.
#[derive(Debug, PartialEq, Eq)]
struct Step {
    body: String,
}

impl Step {
    /// O passo cita todos estes fragmentos?
    fn mentions_all(&self, fragments: &[&str]) -> bool {
        fragments.iter().all(|f| self.body.contains(f))
    }

    /// O passo tem a chave `key` no seu próprio nível?
    ///
    /// Coluna zero é o que distingue o `if:` do passo de um `if:` aninhado
    /// dentro de um `with:` ou de um bloco `run: |`.
    fn has_key(&self, key: &str) -> bool {
        self.value_of(key).is_some()
    }

    /// O valor da chave `key` do passo, já sem espaços em volta.
    fn value_of(&self, key: &str) -> Option<&str> {
        let prefix = format!("{key}:");
        self.body
            .lines()
            .find(|l| l.starts_with(&prefix))
            .map(|l| l[prefix.len()..].trim())
    }
}

fn indent_of(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

fn is_meaningful(line: &str) -> bool {
    let trimmed = line.trim();
    !trimmed.is_empty() && !trimmed.starts_with('#')
}

/// O corpo do job `job`, já sem a indentação do bloco: as chaves do job
/// (`runs-on:`, `permissions:`, `steps:`) ficam na coluna zero, e o que estiver
/// aninhado nelas continua aninhado.
///
/// Varredura por indentação, deliberadamente burra — mesma escolha (e mesmo
/// motivo) do parser de manifesto em `gb-core/tests/purity.rs`: puxar um crate
/// de YAML para o workspace só para ler o próprio arquivo de CI custa mais do
/// que vale. Cobre a forma que este repositório usa, e os testes de parser
/// abaixo dizem exatamente qual é essa forma.
///
/// Devolve vazio se o job não existir.
fn job_body(workflow: &str, job: &str) -> Vec<String> {
    let lines: Vec<&str> = workflow.lines().collect();

    let jobs_at = match lines
        .iter()
        .position(|l| l.trim() == "jobs:" && indent_of(l) == 0)
    {
        Some(i) => i,
        None => return Vec::new(),
    };

    let header = format!("{job}:");
    let job_at = match lines[jobs_at + 1..]
        .iter()
        .position(|l| l.trim() == header && indent_of(l) > 0)
    {
        Some(i) => jobs_at + 1 + i,
        None => return Vec::new(),
    };
    let job_indent = indent_of(lines[job_at]);

    // O bloco do job vai até a próxima linha significativa que volte ao nível
    // dele ou acima — isto é, até o próximo job.
    let job_end = lines[job_at + 1..]
        .iter()
        .position(|l| is_meaningful(l) && indent_of(l) <= job_indent)
        .map(|i| job_at + 1 + i)
        .unwrap_or(lines.len());

    let body = &lines[job_at + 1..job_end];

    // Dedentar pelo cabeçalho do job (`  scoreboard:`) **não** põe as chaves na
    // coluna zero: elas ficam um nível mais fundo, porque são o mapeamento
    // aninhado sob o nome do job. Quem manda é a indentação da primeira chave.
    let key_indent = match body.iter().find(|l| is_meaningful(l)) {
        Some(l) => indent_of(l),
        None => return Vec::new(),
    };

    body.iter()
        .map(|l| l.get(key_indent..).unwrap_or("").to_string())
        .collect()
}

/// O valor de `<scope>:` dentro do bloco `permissions:` do job, se houver.
///
/// O `GITHUB_TOKEN` deste repositório é **read** por padrão
/// (`actions/permissions/workflow` → `default_workflow_permissions: "read"`).
/// Um job que precise escrever tem de pedir, e pedir no job é o certo: dar
/// escrita no topo do workflow daria escrita também ao `check`, que não precisa.
fn job_permission(workflow: &str, job: &str, scope: &str) -> Option<String> {
    let body = job_body(workflow, job);
    let at = body
        .iter()
        .position(|l| l.trim() == "permissions:" && indent_of(l) == 0)?;

    let prefix = format!("{scope}:");
    body[at + 1..]
        .iter()
        .take_while(|l| !is_meaningful(l) || indent_of(l) > 0)
        .find(|l| l.trim().starts_with(&prefix))
        .map(|l| l.trim()[prefix.len()..].trim().to_string())
}

/// Extrai os passos do job `job` de um workflow do GitHub Actions.
///
/// Devolve vazio se o job não existir, ou não tiver `steps:`.
fn steps_of_job(workflow: &str, job: &str) -> Vec<Step> {
    let body = job_body(workflow, job);

    let steps_at = match body.iter().position(|l| l.trim() == "steps:") {
        Some(i) => i,
        None => return Vec::new(),
    };
    let items = &body[steps_at + 1..];

    let item_indent = match items.iter().find(|l| is_meaningful(l)) {
        Some(l) => indent_of(l),
        None => return Vec::new(),
    };

    let mut steps: Vec<Vec<String>> = Vec::new();
    for line in items {
        if !is_meaningful(line) {
            continue;
        }
        let starts_item = indent_of(line) == item_indent && line.trim_start().starts_with("- ");

        // `- ` ocupa duas colunas; tirá-las alinha a primeira chave do passo
        // com as demais.
        let dedented = line.get(item_indent + 2..).unwrap_or("").to_string();

        if starts_item {
            steps.push(vec![dedented]);
        } else if let Some(current) = steps.last_mut() {
            current.push(dedented);
        }
    }

    steps
        .into_iter()
        .map(|body| Step {
            body: body.join("\n"),
        })
        .collect()
}

// --- testes do parser -----------------------------------------------------
//
// O parser é o guarda do guarda. Sem estes, um `steps_of_job` que devolvesse
// sempre `vec![]` deixaria os dois testes de verdade passarem por vacuidade —
// que é precisamente o modo de falha registrado na nota 8 do `STATUS.md`.

const SAMPLE: &str = "\
name: CI

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - if: steps.x.outputs.ok == 'true'
        run: cargo fmt --all -- --check
      # um comentário solto
      - run: cargo test --all

  outro:
    runs-on: ubuntu-latest
    permissions:
      contents: write
      packages: none
    steps:
      - run: echo nada a ver
";

#[test]
fn step_scanner_splits_the_steps_of_the_named_job() {
    let steps = steps_of_job(SAMPLE, "check");
    assert_eq!(steps.len(), 3, "esperava 3 passos, veio {steps:#?}");
    assert!(steps[0].has_key("uses"));
    assert!(steps[2].mentions_all(&["cargo test"]));
}

/// `uses:` e `with:` são irmãs no YAML — a primeira só parece diferente por
/// dividir a linha com o `- `. Tirar `item_indent + 2` colunas alinha as duas
/// na coluna zero, e é isso que faz `has_key` significar "chave **do passo**".
/// O que estiver mais fundo (`components:`, dentro do `with:`) continua fundo.
#[test]
fn step_scanner_puts_step_keys_at_column_zero_and_keeps_nesting() {
    let steps = steps_of_job(SAMPLE, "check");
    assert_eq!(
        steps[0].body,
        "uses: dtolnay/rust-toolchain@stable\nwith:\n  components: rustfmt, clippy"
    );
    assert!(steps[0].has_key("uses"));
    assert!(steps[0].has_key("with"));
    assert!(
        !steps[0].has_key("components"),
        "`components` está dentro do `with:`, não é chave do passo"
    );
}

#[test]
fn step_scanner_distinguishes_conditional_steps() {
    let steps = steps_of_job(SAMPLE, "check");
    assert!(steps[1].has_key("if"), "o passo do fmt é condicional aqui");
    assert!(!steps[2].has_key("if"), "o passo do test não é");
}

#[test]
fn step_scanner_does_not_leak_steps_from_other_jobs() {
    let steps = steps_of_job(SAMPLE, "check");
    assert!(
        !steps.iter().any(|s| s.body.contains("nada a ver")),
        "passo do job `outro` vazou para o job `check`"
    );
    assert_eq!(steps_of_job(SAMPLE, "outro").len(), 1);
}

#[test]
fn step_scanner_returns_empty_for_an_absent_job() {
    assert!(steps_of_job(SAMPLE, "inexistente").is_empty());
}

/// O bloco `permissions:` do job não é um passo e não pode virar um.
#[test]
fn step_scanner_ignores_job_level_blocks_before_the_steps() {
    let steps = steps_of_job(SAMPLE, "outro");
    assert_eq!(steps.len(), 1, "esperava 1 passo, veio {steps:#?}");
    assert!(steps[0].mentions_all(&["nada a ver"]));
}

#[test]
fn permission_scanner_reads_the_job_level_permissions_block() {
    assert_eq!(
        job_permission(SAMPLE, "outro", "contents"),
        Some("write".to_string())
    );
    assert_eq!(
        job_permission(SAMPLE, "outro", "packages"),
        Some("none".to_string())
    );
    assert_eq!(
        job_permission(SAMPLE, "outro", "id-token"),
        None,
        "escopo não declarado não pode ser lido como concedido"
    );
    assert_eq!(
        job_permission(SAMPLE, "check", "contents"),
        None,
        "o job `check` não declara `permissions:` — não herda a do vizinho"
    );
    assert_eq!(job_permission(SAMPLE, "inexistente", "contents"), None);
}

// --- a guarda de verdade --------------------------------------------------

/// ROADMAP 0.2 — o job `check` roda fmt, clippy `-D warnings` e test.
#[test]
fn ci_check_job_runs_fmt_clippy_and_tests() {
    let steps = steps_of_job(&read_workflow(), "check");
    assert!(
        !steps.is_empty(),
        "o job `check` sumiu do .github/workflows/ci.yml, ou perdeu os `steps:`"
    );

    for (label, fragments) in REQUIRED_STEPS {
        assert!(
            steps.iter().any(|s| s.mentions_all(fragments)),
            "ROADMAP 0.2: nenhum passo do job `check` roda {label} ({fragments:?})"
        );
    }
}

/// ROADMAP 0.2a — e roda **sempre**. Um passo pulado é um passo que não mede
/// nada, e o job termina verde mentindo.
#[test]
fn ci_quality_steps_are_unconditional() {
    let steps = steps_of_job(&read_workflow(), "check");

    for (label, fragments) in REQUIRED_STEPS {
        let step = steps
            .iter()
            .find(|s| s.mentions_all(fragments))
            .unwrap_or_else(|| panic!("nenhum passo do job `check` roda {label}"));

        assert!(
            !step.has_key("if"),
            "ROADMAP 0.2a: o passo de {label} está atrás de um `if:` e pode ser \
             pulado sem que a CI fique vermelha. Passo:\n{}",
            step.body
        );
    }
}

// --- a guarda do job `scoreboard` (ROADMAP 0.2b) --------------------------
//
// O job `scoreboard` é o que alimenta a série temporal do ROADMAP 8.2. A
// proteção de `main` exige que ele fique verde, o que — de novo — garante que
// ele rodou, não que mediu. Aqui a preocupação é o inverso da 0.2a: não é o
// passo ser pulado, é o passo **falhar sem derrubar o job**.

/// ROADMAP 0.2b — o job existe e roda as duas coisas de que depende.
#[test]
fn ci_scoreboard_job_fetches_roms_and_runs_the_scoreboard() {
    let steps = steps_of_job(&read_workflow(), "scoreboard");
    assert!(
        !steps.is_empty(),
        "o job `scoreboard` sumiu do .github/workflows/ci.yml, ou perdeu os `steps:`"
    );

    for (label, fragments) in SCOREBOARD_STEPS {
        assert!(
            steps.iter().any(|s| s.mentions_all(fragments)),
            "ROADMAP 0.2b: nenhum passo do job `scoreboard` faz {label} ({fragments:?})"
        );
    }
}

/// ROADMAP 0.2b — e o fracasso deles é o fracasso do job.
///
/// `continue-on-error: true` é o modo de falha específico desta iteração: o
/// passo fica vermelho, o job fica verde, a proteção de `main` libera o merge e
/// o `scoreboard.csv` congela sem que ninguém veja. `if:` reprova pelo motivo
/// da 0.2a — passo pulado não mede nada.
#[test]
fn ci_scoreboard_steps_cannot_fail_silently() {
    let steps = steps_of_job(&read_workflow(), "scoreboard");

    for (label, fragments) in SCOREBOARD_STEPS {
        let step = steps
            .iter()
            .find(|s| s.mentions_all(fragments))
            .unwrap_or_else(|| panic!("nenhum passo do job `scoreboard` faz {label}"));

        assert!(
            !step.has_key("if"),
            "ROADMAP 0.2b: o passo de {label} está atrás de um `if:` e pode ser \
             pulado sem que a CI fique vermelha. Passo:\n{}",
            step.body
        );
        assert!(
            matches!(step.value_of("continue-on-error"), None | Some("false")),
            "ROADMAP 0.2b: o passo de {label} tem `continue-on-error` — ele pode \
             morrer com o job terminando verde. Passo:\n{}",
            step.body
        );
    }
}

/// ROADMAP 0.2b — quando o placar falha, é o artefato que se olha.
///
/// O `upload-artifact` só roda em passo de sucesso por padrão. Justamente na
/// execução que interessa investigar — a que morreu no meio — o CSV parcial
/// ficaria dentro do runner descartado. Este é o único `if:` desejado no job.
#[test]
fn ci_uploads_the_scoreboard_csv_even_on_failure() {
    let steps = steps_of_job(&read_workflow(), "scoreboard");
    let upload = steps
        .iter()
        .find(|s| s.mentions_all(&["upload-artifact"]))
        .expect("ROADMAP 0.2: o job `scoreboard` não sobe mais o artefato do CSV");

    assert_eq!(
        upload.value_of("if"),
        Some("always()"),
        "ROADMAP 0.2b: sem `if: always()` o CSV da execução que falhou se perde \
         com o runner — e é essa a execução que se quer ler. Passo:\n{}",
        upload.body
    );
}

// --- a publicação da série (ROADMAP 0.2c) ---------------------------------
//
// O artefato guarda o CSV de **uma** execução, por 90 dias. Ele não monta a
// série: quando o runner morre, as linhas que a CI produziu somem, e o que
// sobra no git é só o que uma iteração commitou à mão. O passo de publicação é
// o que fecha esse buraco.

/// ROADMAP 0.2c — o job publica a série depois de medi-la.
#[test]
fn ci_scoreboard_job_publishes_the_series() {
    let steps = steps_of_job(&read_workflow(), "scoreboard");
    assert!(
        steps.iter().any(|s| s.mentions_all(PUBLISH_STEP)),
        "ROADMAP 0.2c: nenhum passo do job `scoreboard` publica a série \
         ({PUBLISH_STEP:?}) — as linhas geradas pela CI morrem com o runner"
    );
}

/// ROADMAP 0.2c — e publica **depois** de medir.
///
/// Publicar antes do `scoreboard.sh` mandaria para a branch de dados o CSV do
/// checkout, sem as linhas desta execução: um commit por push em `main` que não
/// acrescenta nada. Continuaria verde, e a série continuaria congelada.
#[test]
fn ci_publishes_after_measuring() {
    let steps = steps_of_job(&read_workflow(), "scoreboard");
    let measure = steps
        .iter()
        .position(|s| s.mentions_all(&["run: ./scripts/scoreboard.sh"]))
        .expect("o job `scoreboard` não roda mais o placar");
    let publish = steps
        .iter()
        .position(|s| s.mentions_all(PUBLISH_STEP))
        .expect("o job `scoreboard` não publica mais a série");

    assert!(
        publish > measure,
        "ROADMAP 0.2c: a publicação (passo {publish}) vem antes da medição \
         (passo {measure}) — publicaria o CSV do checkout, sem esta execução"
    );
}

/// ROADMAP 0.2c — o `GITHUB_TOKEN` deste repositório é read por padrão.
///
/// Sem `contents: write` **no job**, o push é rejeitado por falta de permissão
/// e o passo morre. Verificado na API, não suposto:
/// `actions/permissions/workflow` → `default_workflow_permissions: "read"`.
#[test]
fn ci_scoreboard_job_asks_for_write_access() {
    assert_eq!(
        job_permission(&read_workflow(), "scoreboard", "contents").as_deref(),
        Some("write"),
        "ROADMAP 0.2c: o job `scoreboard` não pede `permissions: contents: write` \
         e o token padrão deste repositório é read — o push da série seria \
         rejeitado por permissão"
    );
}

/// ROADMAP 0.2c — publicar só faz sentido no push para `main`.
///
/// Numa execução de PR o commit medido não está em `main`, e a série ganharia
/// pontos de código que talvez nunca entre. Pior: PR vindo de fork recebe token
/// somente-leitura, e o passo falharia sempre. Este é o segundo `if:` desejado
/// do job — e, como o do artefato, é afirmado aqui para ser uma decisão.
#[test]
fn ci_publishes_only_on_push_to_main() {
    let steps = steps_of_job(&read_workflow(), "scoreboard");
    let publish = steps
        .iter()
        .find(|s| s.mentions_all(PUBLISH_STEP))
        .expect("o job `scoreboard` não publica mais a série");

    let condition = publish
        .value_of("if")
        .expect("ROADMAP 0.2c: o passo de publicação roda em qualquer evento");

    for fragment in ["github.event_name", "'push'", "refs/heads/main"] {
        assert!(
            condition.contains(fragment),
            "ROADMAP 0.2c: a condição do passo de publicação não menciona \
             {fragment} — `{condition}`"
        );
    }
}

/// ROADMAP 0.2c — e o fracasso dele é o fracasso do job, pelo mesmo motivo da
/// 0.2b: série que para de crescer em silêncio é pior do que série nenhuma.
#[test]
fn ci_publish_step_cannot_fail_silently() {
    let steps = steps_of_job(&read_workflow(), "scoreboard");
    let publish = steps
        .iter()
        .find(|s| s.mentions_all(PUBLISH_STEP))
        .expect("o job `scoreboard` não publica mais a série");

    assert!(
        matches!(publish.value_of("continue-on-error"), None | Some("false")),
        "ROADMAP 0.2c: o passo de publicação tem `continue-on-error` — ele pode \
         morrer com o job terminando verde. Passo:\n{}",
        publish.body
    );
}
