# Iteração 0054 — LY e STAT bits de modo (início da PPU)

- **Data:** 2026-07-26
- **Item do roadmap:** 3.1a

## Objetivo

Módulo PPU inicial: LY ($FF44) incrementa a cada scanline (456 T-cycles), STAT ($FF41) reporta os bits de modo (Mode 2→3→0 por scanline, Mode 1 no VBlank), LCDC ($FF40) com PPU enable, LYC ($FF45) para o bit LYC=LY do STAT.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | FF44 — LY, FF41 — STAT, PPU modes, Rendering overview | `docs/reference/06-ppu.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | O range `0xFF40..=0xFF45` delegaria SCY/SCX também ao PPU | SCY ($FF42) e SCX ($FF43) são da iteração 3.1b — mapear o range inteiro capturou endereços que ainda não têm owner | `bus_boot_state.rs` quebrou: `ppu só atende $FF40-$FF45, recebeu $FF42` |
| 2 | flags | STAT bit 7 não é listado na spec mas faz parte do valor de boot | O boot state de STAT é 0x85 (bit 7=1); a máscara writable precisa ser 0xF8 (bits 7-3), não 0x78 (bits 6-3) | Conferi o valor de boot antes de implementar — erro evitado, não cometido |
| 3 | timing | O timing base de Mode 3 seria 160 dots (um scanline de pixels) | A spec define mínimo 172 dots (160 + 12 de penalty dos dois tile fetches iniciais) | Lido da spec antes de codificar — erro evitado |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 11/12 | 11/12 |
| instr_timing | 1/1 | 1/1 |
| mem_timing | 4/4 | 4/4 |
| mem_timing-2 | 0/4 | 0/4 |
| halt_bug | 0/1 | 0/1 |
| oam_bug | 0/9 | 0/9 |
| interrupt_time | 0/1 | 0/1 |
| dmg_sound | 0/13 | 0/13 |
| dmg-acid2 | 0/1 | 0/1 |
| mooneye | 0/75 | 0/75 |

Testes do workspace: **670** (eram **653** na 0053 — 17 novos em `ppu_ly_stat_mode.rs`).

halt_bug e mem_timing-2 continuam crashando: LY agora incrementa, mas as ROMs precisam de VBlank/STAT interrupts para produzir saída serial — o STAT interrupt dispatch (3.2) ainda não existe.

## Revisão cruzada (segundo modelo)

- **Modelo:** —
- **Achados:** —
- **Procedentes:** —
- **Falso positivo mais interessante:** —

## Decisões de arquitetura

- A PPU mora num módulo `ppu.rs` separado, como `serial.rs`. O `Bus` tem um campo `ppu: Ppu` e chama `bus.tick_ppu()` em `Cpu::step()` logo após `bus.tick_timer()`. R2 preservada: a PPU avança 4 dots por M-cycle.
- O PPU armazena sua própria cópia de LCDC, LYC e os bits writable de STAT — não lê do array `io[]` do Bus. LY é calculado; STAT bits 2-0 são computados a cada leitura.
- Mode 3 base = 172 dots (mínimo sem scrolling, window ou sprites). A expandibilidade (penalties de OBJ, window, scrolling) está prevista mas o código atual não alonga Mode 3.
- O roteamento no Bus usa `match` por endereço exato (`0xFF40 | 0xFF41 | 0xFF44 | 0xFF45`), não range, para deixar SCY/SCX com o array `io[]` até a iteração 3.1b.

## Notas

- O scoreboard não mudou — halt_bug e mem_timing-2 ainda não produzem saída serial, embora LY já esteja vivo. O problema é a ausência de STAT interrupts (3.2) e VBlank interrupt (já implementado no 2.2, mas as ROMs podem depender de STAT).
- O teste `ly_and_tac_have_storage_and_no_read_semantics_yet` de `bus_boot_state.rs` foi renomeado para `ly_is_read_only_and_tac_has_storage` — o guarda cumpriu seu papel e a mensagem original dizia "se o componente dono chegou, este teste é que está velho".
- O teste `every_named_register_holds_the_dmg_column_value_at_hand_off` agora pula STAT ($FF41) porque seus bits de modo são computados, não armazenados estaticamente.
