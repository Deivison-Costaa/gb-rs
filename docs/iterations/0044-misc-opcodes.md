# Iteração 0044 — misc: CPL, SCF, CCF, DAA, DI, EI, STOP

- **Data:** 2026-07-26
- **Item do roadmap:** 1.11

## Objetivo

Implementar os 7 opcodes miscelâneos restantes do M1 — `CPL`/`SCF`/`CCF`/`DAA`/`DI`/`EI`/`STOP`. Todos têm 1 M-cycle e 4 T-cycles. `NOP` já existia desde o 1.3.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| `02-cpu.md` | The BCD Flags (DAA) | `docs/reference/02-cpu.md` |
| `03-opcodes.md` | Tabela de opcodes (colunas Z/N/H/C, M-cycles) | `docs/reference/03-opcodes.md` |
| `05-interrupts.md` | DI/EI, IME, EI delay | `docs/reference/05-interrupts.md` |
| `06-ppu.md` | STOP, VRAM access, DIV reset | `docs/reference/06-ppu.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | flags | O `CPL` do Z80 e do SM83 divergem nas flags — achei que seria como as rotações (Z/N/H zerados). A spec diz N=1, H=1, Z/C preservados, igual ao Z80. | N, H, Z e C comportam-se exatamente como no Z80. | Nenhum — a intuição veio da spec, não da memória. |
| 2 | algoritmo | DAA só se aplica após adição (N=0). O caminho de subtração (N=1) é um segundo algoritmo inteiro. | DAA ajusta tanto após ADD quanto após SUB, usando os mesmos H/C como empréstimo em vez de carry. | Leitura da spec antes da implementação (R1). |
| 3 | flags | DAA preservaria N (já é o que a coluna diz) mas eu achei que zerava H também como efeito colateral do ajuste — o que é verdade e a spec confirma. | H=0, N preservado, Z recalculado. Idêntico ao que implementei. | Nenhum. |
| 4 | timing | O STOP é instruction-stepped — a CPU para completamente e não há mais M-cycles. O teste `an_opcode_this_emulator_has_not_reached_is_not_an_illegal_one` em `cpu_mcycle_loop.rs` usava `$10` como exemplo de opcode não implementado e precisou ser trocado para `$76` (HALT). | R1 e R2: a coluna tem `fetch` como M-cycle, e o STOP entra em low-power state. O teste que apontava para o opcode antigo foi pego pela suíte completa (`cargo test --all`). | `cargo test --all` — o teste falhou porque STOP deixou de ser `UndecodedOpcode`. Trocado para `$76` (HALT). |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 0/12 | 0/12 |
| blargg/dmg_sound | 0/13 | 0/13 |

Nenhuma ROM passou a rodar — os opcodes não desbloqueiam sozinhos nenhuma suíte (faltam HALT, interrupções, timer e PPU). O scoreboard continua em 0/121.

## Bateria de mutação

6 mutações (uma por categoria de opcode) + 2 controles:

| Mutação | Resultado | Teste que falhou |
|---|---|---|
| M1: CPL remove `N=true` | PEGO | `cpl_complements_a_and_sets_n_and_h` |
| M2: SCF remove `C=true` | PEGO | `scf_sets_c_and_clears_n_and_h` |
| M3: CCF `C=true` (em vez de `!c`) | PEGO | `ccf_flips_c_and_clears_n_and_h` |
| M4: DAA remove `H=false` (alu.rs) | PEGO | `daa_after_addition_adjusts_bcd_correctly` |
| M5: DI skip `ime=false` | PEGO | `di_clears_ime` |
| M6: STOP retorna `State::Fetch` | PEGO | `stop_advances_pc_and_stops_the_cpu` |
| C1: SCF reordena flags (equivalente) | VERDE | — |
| C2: CCF N/H antes do C (equivalente) | VERDE | — |

**Placar: 6/6 pegos, 2/2 controles verdes.**

## Revisão cruzada (segundo modelo)

Não realizada — iteração de um só desenvolvedor (agente).

## Decisões de arquitetura

1. **`State::Stopped`** é um estado novo, não um `Lockup`. A CPU parada não está _travada_ (não executou lixo nem opcode inexistente) — está suspensa. `lockup()` retorna `None` para `Stopped`, distinguindo-o de `IllegalOpcode` e `UndecodedOpcode`. `is_between_instructions()` retorna `false`. O stub não implementa o despertar por botão (4.1).

2. **`DAA` em `alu.rs`** — como as demais operações da ALU, `daa()` é uma função livre sobre `&mut Registers`. Não há estado de M-cycle: 1 M-cycle resolve tudo no `fetch`.

3. **EI delay** não foi implementado — `self.ime = true` é imediato. O handoff da 0043 registrou que "esse delay só vira relevante com interrupções (2.2)". A decisão está correta e será revisitada no 2.2.

## Notas

- Os 25 testes de `cpu_misc.rs` cobrem cada opcode individualmente (comportamento funcional, flags, 1 M-cycle) mais STOP (PC avança, CPU para, não destrava) e DI/EI (IME).
- O teste `an_opcode_this_emulator_has_not_reached_is_not_an_illegal_one` foi atualizado de `$10` (STOP, agora implementado) para `$76` (HALT, 2.3).
- `NOP` (`$00`) já estava implementado; sua presença no item 1.11 é só de registro — ele marcava o primeiro opcode decodificado do projeto (1.3).
- A suíte `cpu_misc` fecha a cobertura de todos os opcodes single-M-cycle do M1 que não são do prefixo CB nem de desvios — sobram `HALT` (2.3), timer (2.1), interrupções (2.2) e a porta serial (1.12) para fechar o marco M1.
