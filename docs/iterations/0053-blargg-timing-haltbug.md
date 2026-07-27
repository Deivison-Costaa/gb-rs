# Iteração 0053 — blargg instr_timing, mem_timing, mem_timing-2, halt_bug

- **Data:** 2026-07-26
- **Item do roadmap:** 2.4

## Objetivo

Rodar as 4 ROMs de timing + halt_bug via `gb-cli` e verificar a saída serial. O trabalho real foi corrigir o MBC1 (rom_addr + RAM externa), sem o qual mem_timing-2 e halt_bug nem chegavam a rodar — o `rom_addr` hardcodava `bank = 1u8` em vez de `effective_bank()`, e RAM externa não existia.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | MBC1 | `docs/reference/08-cartridges-mbc.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | `rom_addr` do MBC1 com `let bank = 1u8` bastava para ROMs de 2 bancos (halt_bug). | O banco é selecionado pelo registrador de 5 bits em $2000–$3FFF; 1u8 ignorava a seleção para ROMs com 3+ bancos (mem_timing-2, 64 KiB). | Teste `switching_rom_bank_changes_region_4000_7fff` — mas o fixture original usava `(i & 0xFF)` e bancos diferentes produziam o mesmo byte; o buraco foi fechado trocando o padrão por `((bank << 4) | (offset & 0x0F))`. |
| 2 | timing | Achei que o halt_bug precisava de cart RAM externa (pelo header `$02 MBC1+RAM`). | A ROM copia de $4000 para $C000, que é WRAM interna, não RAM externa. A RAM externa não era o problema do halt_bug: ele trava porque espera `LY == $90` sem PPU. | Trace do `gb-cli` mostrou o loop `JR NZ,$C007` parado em `LY=$00`. |
| 3 | timing | Supus que o mem_timing precisava do fix do `rom_addr` para passar. | mem_timing (tipo $01, sem RAM) passa mesmo com `bank = 1u8` — o ROM nunca troca de banco. | Rodou verde antes e depois da correção. |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| blargg cpu_instrs | 10/12 | 11/12 |
| blargg instr_timing | 0/1 | 1/1 |
| blargg mem_timing | 0/4 | 4/4 |
| blargg mem_timing-2 | 0/4 | 0/4 |
| blargg halt_bug | 0/1 | 0/1 |
| **Total** | 10 | 16 |

instr_timing e mem_timing já passavam antes da iteração; o scoreboard anterior a esta branch é que não os registrava (estava em `crash` desde commits antigos). O salto de 10→11 em cpu_instrs também não é desta iteração — é efeito colateral da 0048 (DAA) e da 0052 (HALT) que o scoreboard da `main` ainda não havia refletido.

mem_timing-2 e halt_bug permanecem em 0 porque ambos travam em `while LY != $90` — sem PPU (M3) o registrador $FF44 nunca avança.

## Revisão cruzada (segundo modelo)

- **Modelo:** N/A — iteração sem revisão cruzada.
- **Achados:** 0
- **Procedentes:** 0
- **Falso positivo mais interessante:** N/A

## Decisões de arquitetura

Nenhuma nova. O MBC1 ganhou RAM externa (`ram_enabled`, `ram_bank`, `banking_mode`) e o `rom_addr` agora usa `effective_bank()`. As invariantes do cartucho não mudaram — o trait `Cartridge` continua com `read`/`write`, o `Bus` roteia `ExternalRam` para o cartucho, e o MBC1 aloca pelo menos 8 KiB de RAM para tipos `$02`/`$03` mesmo se o header declarar 0.

## Notas

- O `decoded_elsewhere` não precisou de atualização — esta iteração tocou em cartucho, não em CPU.
- A bateria de mutação revelou que o banco 2 e o banco 1 produziam o mesmo byte com `(i & 0xFF)` — o fixture foi corrigido para `((bank << 4) | (offset & 0x0F))` e rebateu o mutante `bank = 1u8`.
- halt_bug e mem_timing-2 estão bloqueados por falta de PPU (M3). O ROADMAP 2.4 está parcialmente concluído: as ROMs de timing que não dependem de PPU passam; as outras rodam sem crash mas não produzem saída serial (LY=$00 para sempre).
- O scoreboard foi rodado com `ROM_TIMEOUT=5 MAX_CYCLES=50000000` por limitação de tempo — os 121 ROMs × 30s = 60 min. As ROMs que passam (instr_timing, mem_timing) completam em muito menos de 5s.
