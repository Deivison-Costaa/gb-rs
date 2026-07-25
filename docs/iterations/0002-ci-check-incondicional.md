# Iteração 0002 — Job `check` incondicional

- **Data:** 2026-07-25
- **Item do roadmap:** 0.2a (sub-item criado nesta iteração; ver § Notas)
- **PR:** #3
- **Duração:** ~20min
- **Custo reportado:** não medido — sessão interativa, sem `--output-format json`.
  Mesma dívida da 0001.
- **Turnos:** 1

## Objetivo

Fazer os três passos de qualidade do job `check` (fmt, clippy `-D warnings`,
test) rodarem **incondicionalmente**, e deixar um teste que reprove o PR que
tentar condicioná-los de novo.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| — | — | — |

Sem spec, e pelo mesmo motivo da 0001: a R1 vincula "opcode, registrador ou
comportamento de periférico", e a tabela *Qual arquivo ler para cada item do
ROADMAP* em `docs/reference/README.md` começa no 0.3. Nada de hardware foi
tocado aqui — o diff é YAML de CI e um teste que lê YAML.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | `ferramental` | Que num passo do Actions o `with:` é **filho** do `uses:`, e que depois de tirar `item_indent + 2` colunas ele continuaria indentado. Escrevi a asserção esperando `"uses: …\n  with: …"`. | `uses:` e `with:` são **irmãs**: chaves do mesmo mapa. A primeira só parece diferente por dividir a linha com o `- `, que ocupa duas colunas. Tirar `item_indent + 2` alinha as duas na coluna zero — que é exatamente o que faz `has_key` significar "chave *do passo*". | O teste do próprio parser. |
| 2 | `ferramental` | Que a guarda `[ -f Cargo.toml ]` tinha virado **código morto** com o 0.1 — foi assim que a nota 1 do `STATUS.md` a descreveu, e foi assim que eu a li. | Não é código morto: é uma condicional **viva** cujo valor por acaso é `true` hoje. Código morto não roda e não falha; condicional viva desliga três passos no dia em que o valor virar. É essa diferença que torna a remoção uma tarefa de verdade e não uma faxina. | Releitura do `ci.yml` antes de editar, procurando o que exatamente havia de errado com "código morto". |

**Sobre o #1 — quem estava errado era o teste, não o código.** Vale registrar
porque inverte a expectativa do ciclo RED→GREEN: o RED apareceu, mas não no
lugar previsto. O parser já dedentava certo; a asserção é que descrevia um YAML
que não existe. Corrigir foi trocar a asserção *e* aproveitar para cobrir a
distinção que o `has_key` de fato precisa — `components:`, esse sim dentro do
`with:`, tem de continuar invisível como chave de passo. O teste ficou mais
forte do que o que eu tinha escrito de primeira.

**Sobre a vacuidade, de novo (nota 8 do `STATUS.md`).** Dos dois testes de
verdade, `ci_quality_steps_are_unconditional` nasceu vermelho — RED legítimo,
os três passos estavam mesmo atrás do `if:`. Mas
`ci_check_job_runs_fmt_clippy_and_tests` **passou de primeira**, que é o sintoma
descrito na nota 8. Forcei duas mordidas antes de confiar nele:

| Mutação no `ci.yml` | Resultado |
|---|---|
| passo do clippy removido | `nenhum passo do job check roda clippy` |
| `-D warnings` retirado do clippy | `nenhum passo do job check roda clippy` |

Ambas revertidas. A segunda é a que importa: clippy sem `-D warnings` é clippy
decorativo, e o teste distingue os dois casos.

**E o guarda do guarda.** `steps_of_job` devolvendo `vec![]` sempre deixaria os
dois testes de verdade passarem por vacuidade — a mesma armadilha, um nível
abaixo. Daí os cinco testes de parser sobre um YAML sintético, incluindo o de
não vazar passos de um job para outro.

## Placar

Sem mudança e sem regressão — nenhuma linha de emulação foi escrita.

| Suíte | Antes | Depois |
|---|---|---|
| todas as 11 | 0/121 | 0/121 |

`scoreboard.csv`: 242 → 363 linhas (121 novas, todas `crash`, como na 0001).

## Revisão cruzada (segundo modelo)

- **Modelo:** nenhum.
- **Achados:** —
- **Procedentes:** —
- **Falso positivo mais interessante:** —

Segue sem `REVIEWER_CMD` configurado (nota 5 do `STATUS.md`). Campo vazio por
ausência de ferramenta, não por esquecimento.

## Decisões de arquitetura

1. **O teste mora em `cargo test --all`, não no workflow.** Guarda que vive
   dentro da coisa guardada some junto com ela: um passo de lint de YAML dentro
   do `ci.yml` seria removido pelo mesmo PR que remove o que ele protege. Dentro
   de `cargo test --all` ele é exigido pela proteção de `main`, que requer o job
   `check` verde — e `check` só fica verde rodando os testes.
2. **Mora no `gb-cli`.** É o crate de ferramental (já carrega o contrato com
   `scripts/scoreboard.sh` no doc de `main.rs`). O `gb-core` é máquina de estados
   pura e não deve saber que existe CI; pôr o teste lá também tensionaria a
   `purity.rs` sem necessidade.
3. **Parser de YAML escrito à mão, sem dependência nova.** Mesma decisão e mesmo
   motivo do parser de manifesto em `gb-core/tests/purity.rs`: puxar um crate de
   YAML para o workspace só para ler o próprio arquivo de CI custa mais do que
   vale. Cobre a forma que este repositório usa, e os testes de parser dizem
   qual é essa forma.
4. **Fragmentos, não linha de comando inteira.** `REQUIRED_STEPS` casa
   `["cargo clippy", "-D warnings"]` em vez da linha exata. Reprova o que
   importa sem quebrar por uma flag nova.

## Notas

**O 0.2 foi quebrado em três.** Ele acumulava conceitos independentes, e o
protocolo manda quebrar antes de fazer:

- **0.2a** (esta) — passos de qualidade incondicionais + a guarda.
- **0.2b** — job `scoreboard` falhar quando o script morre ou o CSV não cresce.
  Vem da nota 7 do `STATUS.md`: o bug do `scoreboard.sh` corrigido na 0001 teria
  passado despercebido indefinidamente, porque a CI não olha para isso.
- **0.2c** — commit-back do `scoreboard.csv` gerado pela CI. Vem da nota 2: hoje
  a série temporal que vira gráfico no 8.2 só cresce quando uma iteração commita
  na mão; o que a CI produz morre com o job.

**O que este PR *não* conserta.** O job `scoreboard` continua podendo terminar
verde com o script morto — 0.2b. E a proteção de `main` continua exigindo
`check` e `scoreboard`, o que é o mecanismo certo: é ele que dá dentes ao teste
desta iteração.

**Um slide, talvez.** A nota 1 da 0001 previu que a guarda "some sozinha" e
comemorou quando os passos voltaram a rodar. Estava certa sobre o efeito e
errada sobre a natureza: a guarda não sumiu, ela passou a dar `true`. A CI ficou
verde do jeito certo por um motivo frágil, e ninguém teria notado enquanto
continuasse verde. É o padrão que a iteração inteira persegue — verde que não
prova nada.
