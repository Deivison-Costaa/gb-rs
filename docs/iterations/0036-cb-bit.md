# Iteração 0036 — BIT (CB 40–CB 7F)

- **Data:** 2026-07-26
- **Item do roadmap:** 1.9d

> PR, custo, turnos e duração: `docs/metricas.csv`.

## Objetivo

Adicionar 64 opcodes do prefixo CB: BIT (bits 7-6 = 0b01). É o primeiro mecanismo novo dentro de `cb_fetch` — o dispatch deixa de ser plano (`(opcode >> 3) & 0b11111`) e passa a ser hierárquico (bits 7-6 primeiro). BIT não modifica registrador nem memória: só ajusta flags (`Z` = bit testado, `N=0`, `H=1`, `C` intocado). Para `(HL)` são 12 T-cycles (3 M-cycles) com leitura sem escrita.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| tabela gbops | § Opcodes com prefixo CB | `docs/reference/03-opcodes.md` (linhas 377–440) |
| Pan Docs | § CB prefix instructions (layout de bits) | `docs/reference/02-cpu.md` (linhas 791–806) |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | nenhum | — | — | — |

A trap anunciada no handoff — o BIT usa bits 7-6 para operação, e não cabe no dispatch plano por `(opcode >> 3) & 0b11111` — foi antecipadamente neutralizada pelo `STATUS.md`. A reestruturação hierárquica (`match opcode >> 6 { 0b00 => ..., 0b01 => self.cb_bit(opcode), _ => ... }`) saiu correta de primeira.

O comportamento de hardware do BIT do SM83 é idêntico ao do Z80 (mesmas flags, mesma codificação), então não havia divergência de especificação a tropeçar. O `H=1` é a primeira flag H não-zero numa operação CB, e a primeira desde o `AND` do 1.6c — confirmado na spec e implementado corretamente.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 0/12 | 0/12 |

Testes do workspace: 426 (eram 414 — +12 do novo arquivo `cpu_cb_bit`).

## Revisão cruzada (segundo modelo)

- **Modelo:** —
- **Achados:** —
- **Procedentes:** —
- **Falso positivo mais interessante:** —

## Decisões de arquitetura

A reestruturação de `cb_fetch` para dispatch hierárquico (bits 7-6 como primeiro nível) é o layout definitivo para o restante do prefixo CB. Os buckets 0b10 (RES) e 0b11 (SET) terão sua própria chamada de dispatch, com um braço `_` que cai em `UndecodedOpcode` até que 1.9e e 1.9f os implementem.

A escolha de `State::CbBit(u8)` — um estado simples com o índice do bit como payload, em vez de uma máquina de fases como `CbRotHl` — reflete que BIT (HL) tem um único M-cycle extra (leitura sem escrita). RES e SET devolverão read-modify-write, então herdarão o padrão `CbRotHl` (ou uma variante nova) no 1.9e/1.9f.

`cb_bit_hl` recebe `bus: &Bus` (não `&mut Bus`), o que é correto para uma operação read-only. A mutação 6 da bateria confirmou que um hipotético write-back seria pego pelo teste `bit_hl_does_not_modify_memory`.

## Notas

A bateria de mutação foi mais reveladora do que o esperado. O mutante 5 (bit index hardcoded para bit 0: `1 << bit_index` → `1`) só foi pego por 5 dos 12 testes — os 7 que passaram eram ou controles de não-escrita (registrador intocado, memória preservada) ou testes de registrador que testavam justamente o bit 0. O achado é consistente com a nota 46 (o operando de teste tem de distinguir os casos) e com a nota 22 (a previsão de qual armadilha dói erra — e o `STATUS.md` previu o dispatch, não o bit index). Para 1.9e/1.9f, o mesmo padrão se repete e a suíte deve cobrir todos os 8 índices de bit com valores que os distingam.

O segundo byte `0x76` (BIT 6,(HL)) é o único da faixa 0x40–0x7F que não está em `decoded_elsewhere` (porque `$76` é HALT, ainda como `UndecodedOpcode`). O controle negativo o excluiu explicitamente com um comentário no array `known`, seguindo o mesmo padrão da 0035 (que excluiu JR condicional, DAA, CPL, SCF, CCF).
