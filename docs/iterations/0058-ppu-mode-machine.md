# Iteração 0058 — Máquina de modos + interrupções STAT e VBlank

- **Data:** 2026-07-26
- **Item do roadmap:** 3.2

## Objetivo

A PPU sinaliza interrupção VBlank (IF bit 0) ao entrar no VBlank e interrupção STAT (IF bit 1) nas bordas de modo (Mode 2, Mode 0, Mode 1) e na coincidência LYC=LY, conforme os bits de seleção do STAT.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | PPU modes | `docs/reference/06-ppu.md` |
| Pan Docs | STAT ($FF41) | `docs/reference/06-ppu.md` |
| Pan Docs | INT $40 — VBlank | `docs/reference/05-interrupts.md` |
| Pan Docs | INT $48 — STAT | `docs/reference/05-interrupts.md` |
| Pan Docs | Interrupt Sources | `docs/reference/05-interrupts.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | O VBlank pode ser detectado comparando `self.ly == 144` ao final de `tick()`, depois do incremento | LY é incrementado dentro do mesmo tick — é preciso guardar `old_ly` antes do incremento e comparar com `new_ly` para detectar a transição 143→144 | leitura da spec (secaram-se as ordens internas do `tick()`) |
| 2 | timing | Cada fonte de STAT (modo 2, 0, 1, LYC=LY) dispara independentemente | As fontes são ORed em uma única "STAT interrupt line", e a interrupção é disparada na borda de subida (0→1) dessa linha combinada. Se duas fontes consecutivas mantêm a linha alta, não há borda — fenômeno de "STAT blocking" | leitura da spec (STAT blocking não existiria no modelo ingênuo) |
| 3 | flags | LYC=LY é simples comparação `ly == lyc` | A flag LYC=LY só é setada quando o PPU está habilitado (`LCDC.7 == 1`). Com PPU desligado, a flag é sempre 0 | leitura da spec (o método `lyc_eq_ly` incluí a condição extra) |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 11/12 | 11/12 |

Scoreboard inalterado (16/121 — o emulador ainda não renderiza). Testes do workspace: **700** (eram 692 na 0057 — 8 novos em `ppu_ly_stat_mode.rs`).

## Bateria de mutação

**Placar: 5/5 pegos, 1/1 controle verde.**

| # | Mutação | Testes que pegaram |
|---|---|---|
| M1 | `vblank_fired = true` vira `vblank_fired = false` | `vblank_interrupt_fires_when_entering_vblank` |
| M2 | STAT vira level-triggered (`new_stat_line && !old_stat_line` → `new_stat_line`) | `stat_blocking_mode_0_to_mode_1_prevents_vblank_stat_interrupt` |
| M3 | Remove `(lyc_match && sel_lyc)` de `compute_stat_line` | `stat_interrupt_fires_on_lyc_equality_when_enabled` |
| M4 | Remove `(mode == MODE_OAM_SCAN && sel_mode2)` de `compute_stat_line` | `stat_interrupt_fires_on_mode_2_transition_when_enabled` |
| M5 | Remove `(mode == MODE_HBLANK && sel_mode0)` de `compute_stat_line` | `stat_interrupt_fires_on_mode_0_transition_when_enabled` |

Controle:
- C1: implementação correta — 61 suítes de teste verdes

## Decisões de arquitetura

**`PpuSignals` via retorno de `tick()`.** A PPU não tem acesso direto ao array `io` do `Bus` — os sinais de interrupção são retornados como `PpuSignals` e o `Bus::tick_ppu()` os consome, setando diretamente `self.io[IF_IDX]`. Isso evita dois problemas: (1) a PPU não depende do layout do `Bus` e (2) as interrupções disparadas dentro do `tick()` são processadas antes do `check_interrupt()` da CPU, que roda no mesmo `step()` — cumprindo a nota 53 do `STATUS.md`.

**`compute_stat_line` como método privado.** A linha combinada de STAT é o OR lógico das quatro fontes habilitadas. O método é usado duas vezes em `tick()` (antes e depois do avanço de estado) para detecção de borda. Nenhum estado novo (`prev_stat_line`) foi adicionado ao struct — os dois valores são locais ao `tick()`.

**Limitação conhecida:** Escritas em LYC e STAT durante um M-cycle podem causar transições na `stat_line` antes do próximo `tick()`, e essas transições não são detectadas por enquanto (a borda só é vista no `tick()` seguinte). O spurious STAT interrupt por escrita em STAT também não está implementado. Ambos ficam para iterações futuras de refinamento de timing.

## Notas

O STAT blocking entre Mode 0 e Mode 1 é o primeiro caso do projeto em que a especificação local descreve um fenômeno que depende de dois modos consecutivos — e o desenho com `compute_stat_line` ORed + detecção de borda o captura naturalmente, sem caso especial. A transição Mode 0→Mode 1 ocorre entre o último M-cycle de LY=143 (dots ≥ 252, mode = 0) e o primeiro de LY=144 (dots = 0, LY=144, mode = 1). Como ambos acontecem no mesmo `tick()`, a `old_stat_line` inclui Mode 0 e a `new_stat_line` inclui Mode 1 — se ambos estiverem habilitados, a linha fica alta nos dois lados e não há borda.
