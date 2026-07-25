# Iteração 0001 — Workspace Cargo

- **Data:** 2026-07-25
- **Item do roadmap:** 0.1
- **PR:** #2
- **Duração:** ~25min
- **Custo reportado:** não medido — sessão interativa, sem `--output-format json`.
  A partir da 0002, rodar via `scripts/loop.sh` se o número tiver de entrar no gráfico.
- **Turnos:** 1

## Objetivo

Criar o workspace Cargo com `gb-core`, `gb-cli` e `gb-desktop`, com
`#![forbid(unsafe_code)]` no core e guardas automáticas para R3 e R6.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| — | — | — |

**Não houve spec a consultar, e isso é uma resposta legítima.** A R1 vincula
"opcode, registrador ou comportamento de periférico"; 0.1 não tem nenhum dos
três. Confirmei em `docs/reference/README.md` § *Qual arquivo ler para cada
item do ROADMAP*: a tabela começa em 0.3. Nada de hardware foi implementado
aqui — a única constante do core, `MODEL = "DMG"`, é rótulo, não comportamento.

## Erros de primeira tentativa

> Categorias: `flags`, `timing`, `endereçamento`, `borrow-checker`, `API-Rust`,
> `nenhum`. Esta iteração precisou de uma nova: **`ferramental`** — erro em
> script/CI, não em código de emulação. Sugiro adotá-la no TEMPLATE.

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | `ferramental` | Que `scoreboard.sh` rodaria de ponta a ponta assim que existisse um `gb-cli`. | Ele morre em `exit 1` sem anexar **nenhuma** linha. Sob `set -e` + `pipefail`, o `grep -oE 'cycles=[0-9]+'` que não casa sai 1, derruba a atribuição e mata o script — **antes** do fallback `[[ -n "$cycles" ]] \|\| cycles=0` que o próprio autor escreveu na linha seguinte. | A execução do scoreboard: `wc -l scoreboard.csv` não mudou. |
| 2 | `API-Rust` | Que `env!("CARGO_PKG_VERSION")` dentro do `gb-cli`, ao rotular a linha como "core", daria a versão do `gb-core`. | A macro expande em tempo de compilação para a versão do **crate que a invoca**. Eu tinha escrito a mesma macro duas vezes rotulando uma de "core" — dois valores iguais com nomes diferentes. | Releitura do próprio código antes de rodar o scoreboard. Nenhum teste teria pego: era mentira no rótulo, não erro de tipo. |
| 3 | `ferramental` | Que os três testes de invariante nasceriam vermelhos. | Só **um** nasceu (`gb_core_forbids_unsafe`). `gb_core_has_no_unapproved_dependencies` e `workspace_declares_the_three_crates` passaram na primeira execução por **vacuidade** — `gb-core` não tem dependências por construção, e o workspace acabara de ser escrito com os três membros. | Eu mesmo, lendo a saída: 5 passed, 1 failed, quando esperava 3 failed. |

**Sobre o #3 — como resolvi.** Um guarda que nunca foi visto falhar não é
guarda, é decoração. Apontei `gb_core_has_no_unapproved_dependencies`
temporariamente para `crates/gb-cli/Cargo.toml` (que tem uma dependência real)
e confirmei a falha com a mensagem certa:
`Dependências não autorizadas: ["gb-core"]`. Depois revertei. O guarda morde.
`workspace_declares_the_three_crates` continua sem prova de mordida — é
assertiva sobre texto de manifesto, risco baixo, fica registrado como dívida.

**Ordem RED→GREEN, com ressalva honesta.** O protocolo pede teste falhando
antes da implementação. Para um item de scaffolding isso é parcialmente
circular: o teste não roda sem o workspace que ele testa. O que fiz: (a)
`cargo test --all` sem `Cargo.toml` → *could not find Cargo.toml*; (b) escrevi
`tests/purity.rs`; (c) criei o workspace **deliberadamente sem**
`#![forbid(unsafe_code)]`, obtendo um RED verdadeiro; (d) adicionei o atributo.
Os passos (b) e (c) são simultâneos na prática — não dá para fingir o contrário.

## Placar

Sem regressão. O que mudou foi a **classificação**, não o resultado.

| Suíte | Antes | Depois |
|---|---|---|
| blargg cpu_instrs | 0/12 | 0/12 |
| blargg instr_timing | 0/1 | 0/1 |
| blargg mem_timing | 0/4 | 0/4 |
| blargg mem_timing-2 | 0/4 | 0/4 |
| blargg halt_bug | 0/1 | 0/1 |
| blargg oam_bug | 0/9 | 0/9 |
| blargg interrupt_time | 0/1 | 0/1 |
| blargg dmg_sound | 0/13 | 0/13 |
| dmg-acid2 | 0/1 | 0/1 |
| mooneye acceptance | 0/66 | 0/66 |
| mooneye acceptance-nondmg | 0/9 | 0/9 |
| **TOTAL** | **0/121** | **0/121** |

As 121 linhas novas do `scoreboard.csv` saíram de `skip` para `crash`. Isso é
**correto, não regressão**: `skip` significa "gb-cli não existe" e virou mentira
no instante em que o binário passou a existir. `crash` (saída ≠ 0/1/124)
significa "erro do emulador", que é a verdade — não há emulador. Escolhi
`EXIT_NOT_IMPLEMENTED = 2` justamente para não plantar `pass` ou `fail` falso
na série temporal que vira gráfico no 8.2.

Quem for plotar o 8.2: há uma descontinuidade de rótulo no commit desta
iteração. `skip` e `crash` são ambos zero-passando; agrupe os dois como
"não passa" ou o gráfico vai sugerir um evento que não houve.

## Revisão cruzada (segundo modelo)

- **Modelo:** nenhum.
- **Achados:** —
- **Procedentes:** —
- **Falso positivo mais interessante:** —

`scripts/review.sh` continua sem `REVIEWER_CMD` configurado (padrão
`opencode run`, ausente nesta máquina), como já registrado na nota 5 do
`STATUS.md`. Não rodou. Este campo fica vazio por ausência de ferramenta, não
por esquecimento — e como o script só olha `git diff HEAD~1 HEAD -- '*.rs'`,
ele veria apenas o último commit, não o PR inteiro. Vale corrigir junto com a
configuração do revisor.

## Decisões de arquitetura

1. **Identificadores em inglês, comentários e docs em português.** O primeiro
   rascunho de `purity.rs` tinha nomes de teste em português; reescrevi. Motivo:
   `CLAUDE.md` § Arquitetura já nomeia `bus.rs`, `Cartridge`, `NoMbc` em inglês,
   e a API pública de um emulador conversa com nomes de hardware que são
   ingleses de origem. Prosa em português, que é a língua do trabalho.
   → replicado em `STATUS.md`.
2. **Edition 2024, `resolver = "3"`, `rust-version = "1.85"`.** Metadados
   herdados via `[workspace.package]`.
3. **`gb-desktop` sem `winit`/`pixels`/`cpal` por ora.** Entram no 4.4. Cada uma
   custa minutos de CI em todo PR e não há nada para desenhar até a PPU (M3).
4. **`[profile.release] debug = 1`, sem LTO.** O scoreboard roda em release; um
   perfil sem símbolo nenhum torna insuportável depurar pânico de opcode. LTO e
   `codegen-units = 1` ficam para quando houver tempo de execução real a comparar
   (fim do M1) — hoje o custo de compilação é certo e o ganho é especulativo.
5. **`EXIT_NOT_IMPLEMENTED = 2` no `gb-cli`.** Ver § Placar.

## Notas

**A guarda da CI sumiu como previsto** (nota 1 do `STATUS.md`): com `Cargo.toml`
na raiz, `steps.workspace.outputs.exists == 'true'` e os três passos —
`fmt`, `clippy -D warnings`, `test` — voltam a rodar sem tocar em
`.github/workflows/ci.yml`. Confirmar no check do PR.

**O bug do `scoreboard.sh` (erro #1) é o achado mais interessante da iteração.**
Ele estava lá desde o bootstrap e era invisível: o script foi escrito antes do
`gb-cli` (regra R5, corretamente), mas o caminho `mode=run` nunca tinha sido
exercido por ninguém. Um teste escrito antes da implementação também é código
não testado. O sintoma foi cruel — `exit 1` silencioso, zero linhas anexadas,
nenhuma mensagem de erro — e numa CI cujo job `scoreboard` não falha por isso,
teria virado "o CSV parou de crescer" descoberto semanas depois.

Corrigi com `|| true` na atribuição, que é o mínimo para o fallback já escrito
funcionar. Não fiz o `gb-cli` emitir `cycles=0`: o token pertence ao contrato do
1.12 e forjá-lo agora seria escrever no CSV um número que não mede nada.

**Dívida deixada, deliberada:**
- `workspace_declares_the_three_crates` nunca foi visto falhar.
- Custo e duração não instrumentados.
- `scripts/review.sh` sem revisor configurado.
