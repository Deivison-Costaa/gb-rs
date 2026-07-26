# Iteração 0037 — RES (CB 80–CB BF)

- **Data:** 2026-07-26
- **Item do roadmap:** 1.9e
- **PR:** #46
- **Duração:** — min
- **Custo reportado:** —
- **Turnos:** 1

## Objetivo

Adicionar 64 opcodes do prefixo CB: RES (bits 7-6 = 0b10). Mesmo layout que BIT (bits 5-3 = índice do bit, bits 2-0 = operando), mas sem flags (`Z N H C` intocados) e com write-back no registrador ou em `(HL)`. Para registrador são 2 M-cycles (8 T-cycles); para `(HL)` são 4 M-cycles (16 T-cycles) com read-modify-write em fases separadas.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| tabela gbops | § Opcodes com prefixo CB (linhas 441–504) | `docs/reference/03-opcodes.md` |
| Pan Docs | § CB prefix instructions (layout de bits) | `docs/reference/02-cpu.md` (linhas 791–806) |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | falso-positivo | O teste `res_0_of_b_does_not_change_already_zero_bit` passou verde com RES não implementado porque o CPU lock mantinha o valor inalterado. O mesmo com `res_preserves_all_flags_unchanged` — as flags não mudam durante lockup, o teste passava e não media nada. | Teste que não confirma `is_between_instructions()` tolera o lockup silenciosamente. | Três testes passaram na primeira execução pós-escrita (antes da implementação), revelando que o lockup é um falso-positivo silencioso quando o teste só verifica "não mudou". |
| 2 | API-Rust | Usei closures com tipos diferentes em tuplas `[ (opcode, bit, |cpu| cpu.registers.x), ... ]` — Rust infere cada closure como um tipo distinto, e a compilação quebrou. | — | Erro de compilação; refatorei para funções auxiliares `set_reg`/`read_reg`. |

O comportamento de hardware do RES era direto e não houve divergência entre intuição e spec: quatro flags intocadas, máscara `value & !(1 << bit_index)`, (HL) em 4 M-cycles com read-modify-write. A adaptação do padrão `CbRotHl` (Read/Write em duas fases) para `CbResHl` foi mecânica e saiu correta de primeira, incluindo o uso de `latch` entre as fases e o `#[expect(clippy::cast_possible_truncation)]` no write.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 0/12 | 0/12 |

Testes do workspace: **439** (eram **426** — +13 do novo arquivo `cpu_cb_res`).

## Revisão cruzada (segundo modelo)

- **Modelo:** —
- **Achados:** —
- **Procedentes:** —
- **Falso positivo mais interessante:** —

## Decisões de arquitetura

`CbResHlPhase` (Read/Write) foi criado como enum próprio em vez de reusar `CbRotHlPhase`. Os dois têm os mesmos dois estados, mas o compartilhamento criaria dependência conceitual entre operações que a spec trata como independentes — e se uma futura iteração alterar `CbRotHlPhase` (por exemplo, adicionando uma fase de `Internal`), o RES seria arrastado sem aviso.

O dispatch continua usando o `match opcode >> 6` hierárquico introduzido na 0036. O bucket `0b10` (RES) agora chama `cb_res(opcode)`. O bucket `0b11` (SET) continua como `UndecodedOpcode` até o 1.9f.

A mesma nota da 0036 sobre o `decoded_elsewhere` se aplica: toda a faixa `0x80`–`0xBF` já estava em `decoded_elsewhere` como opcodes ALU não-CB (`ADD`/`SUB`/`AND`/`XOR`/`OR`/`CP A,r8`), sem nenhuma exceção — ao contrário do `0x76` que o BIT precisou excluir.

## Notas

O erro 1 (falso-positivo por lockup) revelou um padrão que afeta todos os testes futuros de opcodes novos: testes que só verificam "não mudou" passam contra `UndecodedOpcode` porque o lockup também não muda nada. A correção foi adicionar `assert!(cpu.is_between_instructions(), ...)` a todo teste que exercita a CPU — padrão que já estava presente nos testes de (HL) mas ausente em metade dos de registrador.

A bateria de mutação também perdeu a implementação no meio: o `git checkout` usado para reverter cada mutante também revertia a implementação porque ela nunca havia sido commitada. A correção foi usar `cp` de backup em vez de `git checkout`. Esse é um acidente de workflow, não de spec, mas afeta todas as iterações futuras — o backup via `cp` antes da bateria deve ser feito ANTES de qualquer `git checkout`.

**Bateria de mutação: 7/7 pegos, 2/2 controles verdes.**

| # | Mutação | Sobreviveu? | Quem pegou |
|---|---|---|---|
| 1 | `1 << bit_index` → `1u8` (hardcoded bit 0) | Não (7/13 falham) | Testes com bit_index > 0 |
| 2 | `& !(1 << b)` → `\| (1 << b)` (SET ao invés de RES) | Não (12/13) | Todos os testes de valor |
| 3 | `(opcode >> 3) & 0b111` → `(opcode >> 4) & 0b111` (índice errado) | Não (7/13) | Testes com índice ≠ falso |
| 4 | Remove `bus.write` no Write do (HL) | Não (3/13) | Testes de memória do (HL) |
| 5 | Remove `write_r8` em `cb_res` | Não (8/13) | Testes de registrador |
| 6 | Seta flags Z/N em `cb_res` | Não (2/13) | Testes de flag |
| 7 | Write do (HL) escreve `0x00` | Não (2/13) | Testes de memória do (HL) |
| C1 | Arquivo original restaurado | Controle verde | 13/13 passam |
| C2 | Arquivo original restaurado (pós-bateria) | Controle verde | 13/13 passam |
