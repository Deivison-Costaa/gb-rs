# Iteração 0038 — SET (CB C0–CB FF)

- **Data:** 2026-07-26
- **Item do roadmap:** 1.9f

## Objetivo

Adicionar 64 opcodes do prefixo CB: SET (bits 7-6 = 0b11). Mesmo layout que RES (bits 5-3 = índice do bit, bits 2-0 = operando), sem flags (`Z N H C` intocados) e com `value | (1 << bit_index)`. Para registrador são 2 M-cycles (8 T-cycles); para `(HL)` são 4 M-cycles (16 T-cycles) com read-modify-write em fases separadas. Último bucket do prefixo CB — com este item, todos os 256 opcodes CB estão decodificados e o 0b11 deixa de ser `UndecodedOpcode`.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| tabela gbops | § Opcodes com prefixo CB (linhas 505–568) | `docs/reference/03-opcodes.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | O `match opcode >> 6` com 4 braços explícitos cobriria todos os casos — Rust inferiria que o shift de 6 bits num u8 dá 0..=3. | O compilador trata o resultado de `>> 6` como u8 (0..=255), exigindo um catch-all para os valores 4..=255. | Erro de compilação `E0004: non-exhaustive patterns`. Adicionado `_ => unreachable!("opcode >> 6 é 2 bits (0..=3)")`. |

O comportamento de hardware do SET era direto e não houve divergência entre intuição e spec: a spec confirma o layout de bits idêntico ao RES com a única diferença semântica sendo `| (1 << bit_index)` em vez de `& !(1 << bit_index)`. A implementação foi mecânica a partir do `cb_res`/`cb_res_hl`, com `CbSetHl` e `CbSetHlPhase` espelhando `CbResHl`/`CbResHlPhase`.

O handoff da 0037 dizia "idêntico ao RES" — e estava correto. Nenhuma armadilha de hardware se materializou.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 0/12 | 0/12 |

Testes do workspace: **452** (eram **439** — +13 do novo arquivo `cpu_cb_set`).

## Revisão cruzada (segundo modelo)

- **Modelo:** —
- **Achados:** —
- **Procedentes:** —
- **Falso positivo mais interessante:** —

## Decisões de arquitetura

`CbSetHlPhase` (Read/Write) foi criado como enum próprio, seguindo o mesmo critério do `CbResHlPhase` na 0037: os dois têm os mesmos estados, mas o compartilhamento criaria dependência conceitual entre operações que a spec trata como independentes.

O dispatch usa o `match opcode >> 6` hierárquico. Com o bucket `0b11` preenchido, todos os quatro buckets do prefixo CB estão cobertos (rotações/shifts, BIT, RES, SET). O `_ => unreachable!()` cobre os valores impossíveis do match que o compilador insiste em ver.

O controle negativo desta iteração difere do padrão RES/BIT: a faixa 0xC0–0xFF como opcodes não-CB inclui jumps/calls/ret (ROADMAP 1.10) e instruções misc (1.11) que ainda não foram implementadas. O teste `set_second_bytes_c0_to_ff_known_ones_are_still_in_decoded_elsewhere` verifica apenas o subconjunto de 27 bytes que já têm significado não-CB. Os 37 restantes serão cobertos quando 1.10 e 1.11 forem concluídos.

## Notas

O erro 1 (catch-all do `match opcode >> 6`) é um artefato da mudança de `_ => State::Locked(UndecodedOpcode)` para `0b11 => self.cb_set(opcode)`: com o wildcard removido, o compilador exige exaustividade literal do tipo u8. A substituição por `unreachable!()` preserva a intenção — todos os buckets estão cobertos por braços explícitos — sem introduzir panics em produção (o valor de `>> 6` num u8 é garantidamente 0..=3).

A previsão do handoff da 0037 ("idêntico ao RES, o que pode ser a armadilha" — nota 22) se confirmou: a implementação foi de fato idêntica ao RES, sem surpresas. O único atrito foi de API-Rust (match não exaustivo), não de spec.

**Bateria de mutação: 7/7 pegos, 2/2 controles verdes.**

| # | Mutação | Sobreviveu? | Quem pegou |
|---|---|---|---|
| 1 | `(1 << bit_index)` → `1u8` (hardcoded bit 0) | Não (7/13) | Testes com bit_index > 0 |
| 2 | `value \| (1 << b)` → `value & !(1 << b)` (SET→RES) | Não (12/13) | Todos os testes de valor |
| 3 | `(opcode >> 3) & 0b111` → `(opcode >> 4) & 0b111` | Não (10/13) | Testes com índice ≠ falso |
| 4 | Remove `bus.write` no Write do (HL) | Não (3/13) | Testes de memória do (HL) |
| 5 | Remove `write_r8` em `cb_set` | Não (8/13) | Testes de registrador |
| 6 | Seta `Z=false` em `cb_set` (registrador) | Não (2/13) | Testes de flag |
| 7 | Write do (HL) escreve `0x00` | Não (3/13) | Testes de memória do (HL) |
| C1 | Arquivo original restaurado | Controle verde | 13/13 passam |
| C2 | Arquivo original restaurado (pós-bateria) | Controle verde | 13/13 passam |
