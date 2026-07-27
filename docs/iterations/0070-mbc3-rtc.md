# Iteração 0070 — MBC3: RTC (registradores de clock, latch, halt flag)

- **Data:** 2026-07-27
- **Item do roadmap:** 5.2b

## Objetivo

Implementar os 5 registradores do RTC (`$08`–`$0C`: segundos, minutos, horas, day low, day high com halt/carry) acessíveis via janela RAM quando `ram_rtc_select` está na faixa RTC, mecanismo de latch (`$00` → `$01` via `$6000`–`$7FFF`) e halt flag (bit 6 de `$0C`). O timer não avança sozinho (nota 54).

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | MBC3 | `docs/reference/08-cartridges-mbc.md` § MBC3 |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | cobertura | Achei que o teste `ram_bank_selection_masks_to_three_bits` quebraria com a introdução do RTC porque `select 0x08` agora aponta para RTC e não RAM. Mas o teste usa fixture sem RTC (`has_rtc=false`), então `rtc_select_active()` retorna `false` e o comportamento de RAM persiste. | A função `rtc_select_active()` só retorna `true` quando `has_rtc` está setado. Fixtures sem RTC mantêm o comportamento anterior de RAM banking. | Nenhum teste quebrou na migração; a bateria de mutação confirmou que a proteção de `has_rtc` funciona. |
| 2 | cobertura | O latch `$00` → `$01` teria efeito observável nos registradores RTC. Como o timer não avança (nota 54), a cópia `live→latched` é identidade e o latch não tem efeito mensurável. A mutação que removeu o tracking de `latch_previous` sobreviveu à suíte inteira (42/42 passando). | A spec descreve o latch como cópia dos contadores vivos para registradores visíveis. Sem ticking, não há divergência entre os dois conjuntos. | Mutante C2 (latch no-op) sobreviveu. Não há como testar latch sem ticking de timer. |
| 3 | API-Rust | Pensei que o RTC exigiria um `enum` ou `struct` separado para os registradores. | Um array `[u8; 5]` indexado por `(select - 8)` é suficiente para os 5 registradores contíguos, dispensando match por nome. | A implementação com array indexado passou na primeira tentativa; o erro #1 da 0069 (nibble-A exato vs. máscara) não se repetiu porque o enable de RAM já usa `& 0x0F == 0x0A`. |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 12/12 | 12/12 |
| instr_timing | 1/1 | 1/1 |
| mem_timing | 4/4 | 4/4 |
| dmg-acid2 | 1/1 | 1/1 |
| mooneye acceptance | 0/66 | 0/66 |

Testes do workspace: **855 → 867** (12 novos em `cart_mbc3.rs`).

## Revisão cruzada (segundo modelo)

- **Modelo:** —
- **Achados:** —
- **Procedentes:** —
- **Falso positivo mais interessante:** —

`scripts/review.sh` ainda não configurado (nota 5).

## Decisões de arquitetura

- **RTC como array `[u8; 5]`**: os 5 registradores (`$08`–`$0C`) são contíguos e a indexação por `(select - 8)` é direta. Um `enum` ou struct nomeado acrescentaria boilerplate de match sem ganho de legibilidade.

- **RTC acessível só com `has_rtc`**: o flag `has_rtc` (setado por `with_rtc()` nos tipos `$0F` e `$10`) controla se `rtc_select_active()` retorna `true` para a faixa `$08`–`$0C`. Tipos `$11`–`$13` mantêm o comportamento anterior (RAM banking para todos os selects).

- **RTC independente de RAM física**: o tipo `$0F` (TIMER+BATTERY) não tem RAM externa (`ram.is_empty()`), mas os registradores RTC são acessíveis via `rtc_registers` mesmo assim. A condição `ram_enabled` controla ambos (RAM e RTC), como a spec prescreve.

- **Latch tracking sem efeito**: `latch_previous` armazena o último valor escrito em `$6000`–`$7FFF`. O mecanismo `$00` → `$01` está implementado mas não tem efeito observável porque o timer não avança (nota 54). Quando o ticking for implementado (via relógio do host ou interface manual), bastará copiar `rtc_registers` para um conjunto `latched` dentro do branch de latch.

- **Writes sob latch**: seguindo a nota 55, escritas nos registradores RTC sempre passam (o latch só afetaria leitura). Como não há separação live/latched ainda, a escolha é transparente.

## Notas

- O comentário do arquivo foi atualizado: `ROADMAP 5.2a` → `ROADMAP 5.2a–5.2b` e a linha sobre RTC pendente foi removida.

- A mutação C2 (latch no-op) revela um buraco de cobertura estrutural: enquanto o timer não avança, o latch não tem como ser testado. Isso não é corrigível sem ticking — e ticking não é parte deste item. O handoff para o 5.3 (MBC5) ou para uma futura iteração de ticking deve registrar que o latch precisa de teste quando houver avanço do RTC.

- Placar da bateria: **6/6 pegos, 2/2 controles verdes**.
  - M1: RTC read sem `ram_enabled` → pego por `rtc_registers_require_ram_enabled` + 2 testes de RAM
  - M2: `has_rtc` removido de `rtc_select_active` → pego por `rtc_registers_not_mapped_for_non_rtc_types`
  - M3: offset de índice errado (`-7` em vez de `-8`) → pânico em 4 testes que acessam `$0C`
  - M4: RTC write sem `ram_enabled` → pego por `rtc_writes_are_ignored_when_ram_disabled`
  - M5: faixa RTC exclui `$0C` → pego por 4 testes de day_high
  - M6: faixa RTC deslocada (`$07`–`$0B`) → pego por 4 testes de day_high
  - C1: `latch_previous` inicial alterado → verde (42/42)
  - C2: latch como no-op → verde (42/42)
