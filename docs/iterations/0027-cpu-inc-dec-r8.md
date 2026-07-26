# Iteração 0027 — `INC`/`DEC r8`: a primeira flag que fica intocada

- **Data:** 2026-07-26
- **Item do roadmap:** 1.6e
- **PR:** #33
- **Duração:** ~1 sessão
- **Custo reportado:** n/d — sessão interativa de Claude Code
- **Turnos:** 1

## Objetivo

Os 16 opcodes `INC r8`/`DEC r8` (`00 ddd 100`/`00 ddd 101`), fechando o 1.6.
Primeira operação da ALU que deixa uma coluna — `C` — **intocada** em vez de
calculada (1.6a/1.6b) ou literal (1.6c).

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| gbops | linhas `04`/`05`/.../`3C`/`3D`/`34`/`35` (flags, T-cycles, coluna de M-cycles) | `docs/reference/03-opcodes.md` |
| Pan Docs | layout de bits `inc r8` (`00 ddd 100`/`00 ddd 101`) | `docs/reference/02-cpu.md` |

## Erros de primeira tentativa

`nenhum`. A coluna de flags (`Z` calculado, `N` literal `0`/`1`, `H` carry/
empréstimo do nibble baixo, `C` com `-`) veio pronta da tabela do
`03-opcodes.md` e do handoff da 0026, e as 88 linhas do grupo `x8/alu` já
tinham ensinado o padrão de `H` como carry/empréstimo (1.6a/1.6b) — a única
novidade era `C` ficando de fora, o que é ausência de código (não
`set_flag(Flag::C, ...)` nenhum), não conta nova. A forma de M-cycle do
`$34`/`$35` (`fetch → read((HL)) → write((HL))`) já tinha precedente exato no
`$36` do 1.4b (`StoreImmediateToHl`) — copiei a mesma estrutura de dois
estados com latch.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| todas as 11 | 0/121 | 0/121 |
| testes do workspace | 289 | 304 |

Bateria de mutação: **12/12 pegos, 2/2 controles verdes**.

| # | Mutação | Pego por |
|---|---|---|
| M1 | `increment`: `Z` usa `value == 0` em vez de `result == 0` | `inc_and_dec_set_zero_only_when_the_result_is_zero` |
| M2 | `increment`: `N` forçado a `true` | `inc_sets_n_to_zero_and_dec_sets_n_to_one_no_matter_the_result` |
| M3 | `increment`: fronteira de `H` invertida (`== 0x00` em vez de `== 0x0F`) | `inc_sets_half_carry_exactly_at_the_low_nibble_boundary` |
| M4 | `decrement`: `Z` usa `value == 0` em vez de `result == 0` | `inc_and_dec_set_zero_only_when_the_result_is_zero` |
| M5 | `decrement`: `N` forçado a `false` | `inc_sets_n_to_zero_and_dec_sets_n_to_one_no_matter_the_result` |
| M6 | `decrement`: fronteira de `H` invertida | `dec_sets_half_borrow_exactly_at_the_low_nibble_boundary` |
| M7 | `increment` ganha `set_flag(Flag::C, result < value)` (carry calculado) | `neither_inc_nor_dec_touches_the_carry_flag` |
| M8 | `decrement` ganha `set_flag(Flag::C, result > value)` (empréstimo calculado) | `neither_inc_nor_dec_touches_the_carry_flag` |
| M9 | `inc_dec_r8` troca `increment`↔`decrement` por operação | 8 testes (o próprio valor e as flags divergem) |
| M10 | `inc_dec_hl` junta leitura e escrita num M-cycle só (2 em vez de 3) | `inc_and_dec_hl_are_three_m_cycles_and_the_write_is_the_third`, `inc_and_dec_touch_only_...` |
| M11 | `inc_dec_hl` escreve o valor lido sem aplicar `increment`/`decrement` | 5 testes (valor final errado) |
| M12 | Troca `INC_R8_PATTERN`↔`DEC_R8_PATTERN` | 9 testes |

Controles (não devem quebrar nada — e não quebraram):

| # | Mutação | Resultado |
|---|---|---|
| C1 | Troca a ordem dos dois `match` guards de `INC`/`DEC` no `fetch` (padrões disjuntos) | 15/15 verdes |
| C2 | Reordena as três chamadas `set_flag` independentes dentro de `increment` | 15/15 verdes |

## Revisão cruzada (segundo modelo)

Não houve. `scripts/review.sh` continua sem `REVIEWER_CMD` configurado
(`STATUS.md`, nota 5).

## Decisões de arquitetura

**`increment`/`decrement` devolvem o resultado em vez de escrever em
`registers.a`.** Ao contrário de `add`/`subtract`/`logic` (que sempre operam
sobre o acumulador), o operando de `INC`/`DEC` é qualquer `r8` ou `(HL)` — a
função da ALU não sabe o destino, só recebe o valor e devolve o novo valor
mais as três flags que calcula. Quem escreve o resultado no lugar certo é
`inc_dec_r8`/`inc_dec_hl`, em `mcycle.rs`.

**`IncDecHl` espelha `StoreImmediateToHl` (1.4b) e não `AluFromHl` (1.6a).** O
`(HL)` do 1.6a lê e aplica no mesmo M2 porque a ALU ali só lê — nunca escreve
de volta em memória. Aqui a leitura e a escrita são dois acessos ao barramento
no mesmo endereço, e por isso precisam de dois M-cycles distintos com um latch
entre eles, como o `$36`. Confundir os dois padrões era exatamente o mutante
M10, e ele deu 12 T-cycles corretos com a estrutura errada por baixo — só a
bateria de mutação, não a leitura do código, prova a diferença.

## Notas

### O ponto do item é a ausência, e ausência não aparece lendo o código

`C` fica intocado porque nenhuma linha em `increment`/`decrement` o menciona.
Isso não se vê olhando o `diff` — se vê testando que o flag **sobrevive** dos
dois lados (limpo antes de um `INC`/`DEC` que estouraria o byte inteiro, ligado
antes de um que não estoura nada). Os mutantes M7/M8 confirmam que um `C`
"calculado por engano" (carry/empréstimo aritmético de verdade) passaria
despercebido em qualquer teste que só olhasse `Z`/`N`/`H` — é a mesma classe de
"controle negativo que sobrevive por redundância" da nota 29, aplicada a uma
flag em vez de a um opcode.

### `decoded_elsewhere` e o exemplo que expirou

`cpu_mcycle_loop.rs` usava `$04` (`INC B`) como exemplo de "opcode que este
emulador ainda não decodifica" desde a 0013 — a 0022 já havia registrado que
esse exemplo sobreviveria "mais quatro sub-itens" (nota no doc da 0022). Este
é o sub-item que o consome; o teste foi trocado para `$07` (`RLCA`, 1.8, ainda
não implementado). Os doze arquivos que consultam `decoded_elsewhere` ganharam
`opcode & 0b1100_0111 == 0b0000_0100` e `== 0b0000_0101` em
`tests/support/mod.rs`, o único lugar onde a função existe desde a 0026 — nenhum
arquivo precisou de edição além desse.
