# Iteração 0032 — RLCA / RRCA / RLA / RRA

- **Data:** 2026-07-26
- **Item do roadmap:** 1.8
- **PR:** _(a ser preenchido)_
- **Duração:** ~40min
- **Custo reportado:** _(não medido nesta sessão)_
- **Turnos:** 1

## Objetivo

Decodificar as quatro rotações sobre o acumulador: `RLCA` ($07), `RRCA` ($0F),
`RLA` ($17) e `RRA` ($1F) — 1 M-cycle, 4 T-cycles cada.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| gbops | Tabela completa de opcodes (x8/rsb) | `docs/reference/03-opcodes.md` |
| Pan Docs | CPU Instruction Set (encodings Block 0) | `docs/reference/02-cpu.md` |

## Erros de primeira tentativa

> O handoff da 0031 pré-anunciou a armadilha central com precisão: "essas quatro
> zeram Z incondicionalmente (coluna 0), enquanto os equivalentes prefixados
> por CB calculam Z". A leitura do `03-opcodes.md` confirmou antes do código.
> Sem o handoff, eu teria escrito `Z = result == 0` como em toda ALU já feita.

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | flags | `Z` calculado a partir do resultado de `A` (como `AND`/`XOR`/`OR`/`ADD`) | `Z = 0` **incondicional**: mesmo quando o resultado é zero, `Z` fica falso | handoff da 0031 + `03-opcodes.md` confirmou antes do código vir abaixo |
| 2 | flags | `N`/`H` provavelmente iguais às da ALU lógica (`N=0, H=0`) | `N = 0`, `H = 0` — confirmado | spec |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 0/12 | 0/12 |

Nenhuma ROM de teste passa — ainda não há PPU, timer, interrupções, nem os
desvios condicionais (1.10).

Testes do workspace: **352** (eram **337** antes da 0032 — +15 testes novos).

## Revisão cruzada (segundo modelo)

- **Modelo:** _(não realizada — iteração de turno único)_
- **Achados:** —
- **Procedentes:** —
- **Falso positivo mais interessante:** —

## Decisões de arquitetura

As quatro funções de rotação (`rlca`, `rrca`, `rla`, `rra`) foram adicionadas a
`alu.rs` como funções livres (`pub(super)`), cada uma recebendo `&mut Registers`
e operando exclusivamente sobre `A`. A alternativa seria um módulo `rotate.rs`,
descartado porque as funções são curtas (8 linhas cada), setam flags como as
demais em `alu.rs`, e o módulo extra não compraria separação de conceito que o
1.9 (prefixo CB) ainda não pediu.

Cada opcode é reconhecido por constante exata no `fetch()` (padrão do `NOP` e
do `$F9`), sem máscara — as quatro compartilham bits 3–0 = 0111 ou 1111 nos
grupos de bits 5–4 (00, 01, 10, 11), que não formam um bloco contíguo que uma
máscara capture sem falso positivo.

## Bateria de mutação

| # | Mutação | Pego? |
|---|---|---|
| M1 | Z computado (`result == 0`) em vez de `false` na `rlca` | sim |
| M2 | H = `true` em vez de `false` na `rlca` | sim |
| M3 | N = `true` em vez de `false` na `rlca` | sim |
| M4 | RLCA usa `>>` em vez de `<<` (rotação invertida) | sim |
| M5 | RLA não inclui `carry_in` (só `a << 1`) | sim |
| M6 | RRA não inclui `carry_in` (só `a >> 1`) | sim |
| M7 | RRCA lê carry do bit 7 em vez do bit 0 | sim |
| C1 | Variável não usada (`let _unused = 42;`) na `rlca` | verde |

**Placar: 7/7 pegos, 1/1 controles verdes.**

## Notas

O teste `an_opcode_this_emulator_has_not_reached_is_not_an_illegal_one` em
`cpu_mcycle_loop.rs` usava `$07` (`RLCA`) como exemplo de opcode não decodificado.
Trocado para `$10` (`STOP`, item 1.11), que permanece não implementado — o
padrão de manter um opcode "vivo" como testemunha do catch-all persiste.
