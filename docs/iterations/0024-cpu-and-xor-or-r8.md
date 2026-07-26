# Iteração 0024 — `AND a,r8`, `XOR a,r8` e `OR a,r8`: flags que não contam nada

- **Data:** 2026-07-26
- **Item do roadmap:** 1.6c
- **PR:** #28
- **Duração:** ~1 sessão
- **Custo reportado:** n/d — sessão interativa de Claude Code, fora do `loop.sh`
- **Turnos:** 1

## Objetivo

Os 24 opcodes `$A0`–`$B7` (`10 100 rrr`, `10 101 rrr`, `10 110 rrr`): `AND`,
`XOR` e `OR` sobre os oito `r8`. Muda a natureza da armadilha em relação ao
1.6a/1.6b — aqui `H`/`C` são constantes na coluna, não resultado de carry ou de
empréstimo.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| gbops | linhas `A0`–`B7` (flags, T-cycles, coluna de M-cycles) | `docs/reference/03-opcodes.md` |
| `STATUS.md` | Handoff da 0023 (`Próxima tarefa`): a armadilha muda de natureza | — |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | flags | *(nenhum de conta)* — a tabela do 1.6c em `STATUS.md`/`ROADMAP.md` já tinha os valores de `H`/`C` por operação (`AND`: `H=1`,`C=0`; `XOR`/`OR`: `H=0`,`C=0`) escritos letra por letra desde o handoff da 0023, e a linha de gbops confirmou sem ajuste. `alu::logic` recebe `half` como parâmetro literal por chamada em vez de calcular — a forma que a armadilha pedia para evitar. | idem | conferido contra `docs/reference/03-opcodes.md`, linhas `A0`–`B7`, antes de escrever `apply` |
| 2 | clippy | Escrevi `every_bitwise_op_matches_...` com `CASES: [(u8, fn(u8, u8) -> u8, bool); 3]` e o teste do M2 com `0xFF & AFTER`/`0xFF ^ AFTER`. | `clippy::type_complexity` (tipo de função inline sem `type` nomeado) e `clippy::identity_op` (`0xFF` é o elemento neutro do `&`, e o lint pega literal-contra-literal mesmo em teste). | `cargo clippy --all-targets -- -D warnings` reprovou nos dois antes do `cargo test --all`. |

Nenhum erro de hardware: a tabela de flags do 1.6c veio pronta do handoff da
0023, e a única superfície nova (`logic()` recebendo `H` como parâmetro em vez
de computar) era exatamente o que a nota "half-carry calculado genericamente
erra as três" pedia para evitar — não houve versão errada a descartar antes
dela. O que sobreviveu foi atrito de ferramenta (clippy), não de spec.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| todas as 11 | 0/121 | 0/121 |
| testes do workspace | 266 | 277 |

## Revisão cruzada (segundo modelo)

Não houve. `scripts/review.sh` continua sem `REVIEWER_CMD` configurado
(`STATUS.md`, nota 5).

## Decisões de arquitetura

**Nenhuma nova.** `AluOp` ganhou três variantes (`And`, `Xor`, `Or`) e
`alu::apply` despacha as três para uma quarta função interna, `logic`, que
recebe o resultado já combinado (`a & operand`, `a ^ operand`, `a | operand`)
e o valor literal de `H` — sem calcular carry nenhum, ao contrário de
`add`/`subtract`. `mcycle.rs` não ganhou forma de M-cycle nova: os seis
opcodes de `(HL)` (`$A6 $AE $B6`) reusam `State::AluFromHl(AluOp)`, que a 0022
generalizou prevendo exatamente esta terceira chegada.

## Notas

### `H`/`C` como parâmetro, não como conta

`add`/`subtract` calculam `H`/`C` a partir dos operandos porque são carry e
empréstimo de verdade. `AND`/`XOR`/`OR` não têm carry nenhum — a coluna do
gbops é literal, igual ao `N=1` do 1.6b. Copiar a forma de `add`/`subtract`
(calcular um "half-carry" a partir de `a`/`operand`) teria acertado o `AND`
por coincidência em alguns casos e errado sistematicamente o `XOR`/`OR`, que
são `H=0` sempre. Os testes `and_sets_h_and_clears_c_no_matter_the_operands` e
`xor_and_or_clear_h_and_c_no_matter_the_operands` escolhem pares de operando
que dariam o resultado errado se `H` fosse computado pela fórmula de soma ou
de subtração — não pares aleatórios.

### O controle negativo, pela quarta vez

`decoded_elsewhere`/`previously_decoded` ganhou `(0xA0..=0xB7)` nos dez
arquivos que já o tinham (os nove da 0023 mais o próprio
`cpu_sub_sbc_cp_r8.rs`, que passou a precisar dele agora que não é mais o
sub-item mais recente). O procedimento que a 0023 registrou — procurar por
`0x00..=0xFF`/`for opcode in`, não só por `fn decoded_elsewhere` — continua
sendo o que acha os dez de uma vez; `cpu_ld_r8_u8.rs` é o único com a variável
inline.

### Atrito

- **`alu.rs` passou no R7 de primeira desta vez** (52 → 78 linhas, 5% de
  comentário): os dois comentários novos (um por função) seguem o padrão
  reduzido que a 0023 já havia adotado depois de reprovar — uma linha,
  apontando para o `STATUS.md`/ROADMAP em vez de reexplicar a regra.
