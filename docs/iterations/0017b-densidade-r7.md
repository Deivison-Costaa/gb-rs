# Iteração 0017b — densidade de comentário e a R7

- **Data:** 2026-07-26
- **Item do roadmap:** nenhum — trabalho fora da escada, pedido pelo usuário
  para o relatório final (8.1). Sufixo `b` pela convenção da 0004b.
- **PR:** #20
- **Duração:** não medida
- **Custo reportado:** não medido — ver `Notas`
- **Turnos:** não medidos

> **Este documento é reconstrução, não relato.** Quem executou o PR #20 foi uma
> sessão de Kimi K3 no OpenCode, que não escreveu doc de iteração. O que segue
> foi remontado do git, do CSV de métricas e do reflog em 26/07, no passo de
> auditoria da 0018. Os campos que dependem de ter estado lá — o que o agente
> pensou antes de agir — estão marcados como não observados, e não preenchidos
> por inferência.

## Objetivo

Derrubar a densidade de comentário do código e fixar um teto executável, movendo
justificativa de decisão para os docs de iteração em vez de apagá-la.

## Spec consultada

Nenhuma de hardware. A regra nova é do projeto, não do SM83.

| Fonte | Seção | Arquivo local |
|---|---|---|
| `CLAUDE.md` | R7, escrita nesta iteração | `CLAUDE.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que era de fato | Como foi pego |
|---|---|---|---|---|
| 1 | processo | a branch podia se chamar `iter/0018-*` | `0018` é número de iteração e o PR não gerou doc nem commit `iter 0018:`; a convenção pra trabalho fora da escada já existia desde o PR #6 (`iter/0004b-*`) | auditoria da 0018 — e antes disso, na prática: a sessão seguinte criou `iter/0019-cpu-ld16-stack`, voltou, criou `iter/0018-cpu-ld-16bit-stack`, e abandonou as duas sem um commit |
| 2 | processo | o teto pedido era o teto entregue | pedido: 10% por arquivo. Entregue: 5% como alvo no `CLAUDE.md`, 12% como número que reprova no `comment_density.rs`, e "teto de 5%" no cabeçalho do `comment-density.sh` — três números em três lugares | leitura dos três arquivos na auditoria |

Erros de conteúdo — o que o agente teria escrito de memória e a spec contradisse
— **não observados**. Não houve spec envolvida e ninguém registrou o processo.

## Placar

Medido em 26/07 com a fórmula do `comment-density.sh`, rodada contra as duas
árvores (`1eb5a34` antes, `37c5e75` depois):

| Métrica | Antes | Depois |
|---|---|---|
| Densidade de comentário (total) | 36% | 3% |
| Linhas de comentário | 3.091 (2.498 doc + 593 internas) | 179 (34 doc + 145 internas) |
| Linhas de código | 5.295 | 5.355 |
| Pior arquivo | `gb-cli/src/exit.rs` 90% | `gb-core/src/cart/mod.rs` 9% |
| Arquivos acima de 12% | 30 | 0 |
| Testes | 196 | 197 (+1: `comment_density`) |
| ROMs de teste | 0/121 | 0/121 |

Diff: 33 arquivos, +202 −3.046. Saíram 2.912 linhas de comentário, e **81%
delas eram doc-comment** (`///`/`//!`), não ruído inline — o volume estava na
documentação de API, que é onde a citação de spec e a justificativa de decisão
tinham ido parar.

Um detalhe que o teto de 12% não decidiu: dos 30 arquivos reprovados antes,
**os mesmos 30** reprovariam a 10%. Nenhum arquivo do projeto vivia na faixa
entre os dois números, então a escolha do teto não mudou uma linha do trabalho.

## Revisão cruzada (segundo modelo)

Não houve — este PR **é** o segundo modelo. Inverteu-se: o Claude fez o
diagnóstico e o plano (medição, verificação de segurança, texto da R7), o Kimi
executou a limpeza.

## Decisões de arquitetura

**O teto é executável, não conselho.** `crates/gb-core/tests/comment_density.rs`
reprova qualquer `.rs` acima de `MAX_PERCENT`, e roda no mesmo `cargo test --all`
da CI. `scripts/comment-density.sh` usa a mesma fórmula para diagnóstico — as
duas classificam linha por prefixo (`///`/`//!` = doc, `//`/`/*`/`*` = interna)
e dividem comentário por (código + comentário), ignorando linha em branco.
Mexeu numa, mexa na outra.

**O que sobrevive no código são duas coisas**, e a segunda é o achado: ponteiro
de uma linha para o erro registrado (`ver docs/iterations/NNNN`). Dez deles
passaram pela limpeza. Os dois da 0017 (`mcycle.rs:111` e `:133`) são o motivo
de a regra existir nessa forma: quando a sessão daquela iteração morreu no meio
do RED→GREEN, foi por eles que a sessão seguinte se reorientou.

## Notas

1. **O custo desta iteração não existe.** O `loop.sh` mede custo/turnos/duração
   lendo o JSON do `claude -p --output-format json`; o que roda por fora não
   deixa rastro. Três PRs do projeto estão nessa situação (#1, #6, #20) e o
   #20 é de longe o maior deles. Para o relatório: a série de custo cobre as
   iterações do loop, não o projeto inteiro.

2. **A guarda de árvore limpa e a limpeza não podiam coexistir.** `loop.sh`
   aborta a iteração se `git status --porcelain` não for vazio, então qualquer
   edição no repo durante uma cadeia mata as iterações seguintes. Foi por isso
   que esta limpeza esperou a cadeia de 10 fechar, e por isso o material de
   preparação viveu em `logs/comment-cleanup/` (gitignorado) até virar PR.

3. **Custo do erro de numeração, medido:** duas branches criadas e abandonadas
   (~5 min e ~2 min), zero commits, antes de a tarefa 1.5 finalmente sair na
   0018. O nome de uma branch consumiu um número que o registro não tinha, e a
   sessão seguinte gastou o tempo tentando adivinhar qual era o seu.

4. **Pendência aberta:** o número que reprova é 12, o pedido foi 10, e o alvo
   escrito é 5. Reduzir para 10 passaria hoje sem tocar em nenhum arquivo (o
   pior está em 9%), mas aperta arquivos curtos — `gb-desktop/src/main.rs` tem
   15 linhas, e uma linha de comentário a mais lá já são 13%. Decisão do
   usuário, não do agente.
