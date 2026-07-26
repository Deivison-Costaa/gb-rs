# Iteração 0028 — `INC`/`DEC r16`: a primeira ALU que não toca flag nenhuma

- **Data:** 2026-07-26
- **Item do roadmap:** 1.7a
- **PR:** #34
- **Duração:** n/d — sessão interrompida duas vezes (sem relato) e retomada
  por uma terceira sessão
- **Custo reportado:** n/d — sessão interativa de Claude Code
- **Turnos:** n/d (duas primeiras tentativas) + 1 (retomada)

## Objetivo

Os 8 opcodes `INC r16`/`DEC r16` (`00 rr 0011`/`00 rr 1011`, `$03 $13 $23 $33
$0B $1B $2B $3B`), primeiro sub-item do 1.7 (quebrado em quatro na própria
0028, commit anterior a este). Primeira ALU do projeto que deixa **as quatro**
colunas de flag intocadas — o 1.6e (`INC`/`DEC r8`) só tinha feito isso para
`C`.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| gbops | linhas `03`/`0B`/`13`/`1B`/`23`/`2B`/`33`/`3B` (flags `-`/`-`/`-`/`-`, T-cycles, coluna de M-cycles) | `docs/reference/03-opcodes.md` |
| Pan Docs | layout de bits `inc r16` (`00 rr 0011`) | `docs/reference/02-cpu.md` |

## Retomada — o que foi encontrado

Duas sessões anteriores morreram implementando isto sem deixar relato; o
código chegou pronto na branch (`mcycle.rs` +51 linhas, `tests/support/mod.rs`
+2, `tests/cpu_inc_dec_r16.rs` novo) sem ter passado por nenhum gate do passo
6. Por instrução explícita da retomada, o código não foi presumido correto —
foi lido inteiro contra `docs/reference/03-opcodes.md`/`02-cpu.md` e depois
submetido à bateria de mutação obrigatória antes de qualquer commit novo.

**A implementação em si (`mcycle.rs`) bateu com a spec em tudo**: os oito
opcodes reduzem à mesma máscara `0b1100_1111` de `LD r16,u16`/`PUSH`/`POP`
(o campo `rr` nos bits 5-4), `inc_dec_r16` não chama nenhuma função de flag —
só `wrapping_add(1)`/`wrapping_sub(1)` sobre o par de 16 bits — e o M-cycle
está partido do jeito que a coluna anota (`fetch` escreve a metade baixa via
`write_r16_low` e devolve `State::IncDecR16`; o `internal` seguinte lê o
`latch` e escreve a metade alta via `write_r16_high`, State::Fetch). Nenhum
erro de hardware sobreviveu à leitura.

## Erros de primeira tentativa

| # | Categoria | O que estava no código órfão | O que a bateria de mutação achou | Como foi pego |
|---|---|---|---|---|
| 1 | cobertura | `tests/cpu_inc_dec_r16.rs` sujava `F` com um único valor, `DIRTY_F = 0b1010_0101`, antes de cada `INC`/`DEC` e conferia que `F` não mudava | O bit 7 (`Z`) de `0b1010_0101` já é `1`. Uma mutação que força `set_flag(Z, true)` quando o resultado dá `0x0000` (engano plausível — é exatamente a regra de `Z` que `INC r8` usa) **não muda o byte observado** e passa despercebida em `inc_and_dec_of_a_pair_leave_every_flag_untouched` | Bateria de mutação obrigatória, mutação 1 (ver Placar). Corrigido trocando o valor único por dois: `ALL_FLAGS_SET = 0b1111_0000` e `ALL_FLAGS_CLEAR = 0b0000_1111`, cobrindo as duas polaridades de cada um dos quatro bits de flag |

Nenhum erro de hardware: o achado é de cobertura de teste no código órfão, não
de spec mal lida — `inc_dec_r16`/`finish_inc_dec_r16` já vieram certos das
duas tentativas anteriores. A medição "nenhum erro de hardware" está
enfraquecida pela mesma razão da nota 42 do `STATUS.md`: "nenhum observável por
quem chegou depois" não é o mesmo que "nenhum aconteceu" — as duas sessões
mortas não deixaram rastro de quais becos elas andaram antes de travar.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| todas as 11 | 0/121 | 0/121 |
| testes do workspace | 304 | 314 |

Bateria de mutação: **5/5 pegos, 2/2 controles verdes** (depois da correção do
achado #1 — a mutação 1 não era pega antes dela).

| # | Mutação | Pego por |
|---|---|---|
| M1 | `inc_dec_r16` ganha `set_flag(Flag::Z, true)` quando `result == 0` | `inc_and_dec_of_a_pair_leave_every_flag_untouched` (só depois da correção do achado #1; sobrevivia com o `DIRTY_F` único) |
| M2 | `inc_dec_r16` escreve a metade alta também no `fetch` (as duas metades no M1, em vez de uma por M-cycle) | `each_half_of_the_pair_lands_on_its_own_m_cycle` |
| M3 | `inc_dec_r16` escreve a metade baixa sempre em `R16::Bc`, ignorando o `target` decodificado | 7 dos 10 testes (`DE`/`HL`/`SP` divergem do valor esperado) |
| M4 | `inc_dec_r16` faz tudo num M-cycle só (escreve as duas metades e devolve `State::Fetch`) | `each_half_of_the_pair_lands_on_its_own_m_cycle`, `inc_dec_changes_only_its_own_pair_and_the_program_counter` (a segunda `step` executa o `$00` seguinte por engano, e o `PC` sobra em 1) |
| M5 | `IncDecR16` ganha um M-cycle extra (`bool` de padding) antes de escrever a metade alta — 3 M-cycles em vez de 2 | `each_half_of_the_pair_lands_on_its_own_m_cycle`, `inc_wraps_from_ffff_to_0000`, `dec_wraps_from_0000_to_ffff` |

Controles (não devem quebrar nada — e não quebraram):

| # | Mutação | Resultado |
|---|---|---|
| C1 | Troca a ordem dos dois `match` guards de `INC_R16_PATTERN`/`DEC_R16_PATTERN` no `fetch` (padrões disjuntos) | 10/10 verdes |
| C2 | Reescreve o `match op { Increment => ..., Decrement => ... }` como `if op == IncDecOp::Increment { .. } else { .. }` | 10/10 verdes |

## Revisão cruzada (segundo modelo)

Não houve. `scripts/review.sh` continua sem `REVIEWER_CMD` configurado
(`STATUS.md`, nota 5).

## Decisões de arquitetura

Nenhuma nova. `read_r16` (leitura do par pelo `R16`) completa a trinca que já
existia só para escrita (`write_r16_low`/`write_r16_high`) — simetria, não
decisão.

## Notas

### Terceira sessão numa fileira de retomadas silenciosas

Depois da 0026 (uma retomada) e agora da 0028 (duas), o padrão está claro: uma
sessão que trava no meio do RED→GREEN deixa código no disco sem deixar
julgamento sobre ele, e o código *parece* pronto — compila, os testes que
existem passam — sem ter passado pela bateria de mutação que prova se os
testes efetivamente olham para o que afirmam. As notas 41/42 do `STATUS.md` já
previam a forma do buraco; esta iteração não achou um erro de hardware, mas
achou exatamente o tipo de buraco de cobertura que a nota 47/48 descreve —
desta vez num valor de teste (`DIRTY_F`) escolhido sem pensar nas duas
polaridades de cada bit, não numa regra de decodificação.

### `DIRTY_F` como controle negativo que não controlava nada

O mesmo mecanismo da nota 29 (`STATUS.md`) — "controle negativo que sobrevive
por redundância/coincidência não é controle" — apareceu aqui numa forma nova:
não é redundância de cópias, é um valor de teste que coincidia, por acidente,
com o resultado que uma mutação plausível produziria. Testar as duas
polaridades (`ALL_FLAGS_SET`/`ALL_FLAGS_CLEAR`) em vez de um valor "sujo"
único é o que fecha essa classe de buraco para qualquer flag futura testada
da mesma forma.
