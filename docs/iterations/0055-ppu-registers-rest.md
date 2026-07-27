# Iteração 0055 — registradores restantes da PPU (ROADMAP 3.1b)

- **Data:** 2026-07-26
- **Item do roadmap:** 3.1b

## Objetivo

Roteamento de SCY ($FF42), SCX ($FF43), DMA ($FF46), BGP ($FF47), OBP0 ($FF48),
OBP1 ($FF49), WY ($FF4A) e WX ($FF4B) pelo `ppu.read`/`ppu.write`.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | Scrolling, Palettes, Window, OAM DMA Transfer, LCDC, STAT | `docs/reference/06-ppu.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | Usei `0xFF40 \| 0xFF41 \| ... \| 0xFF4B` em dois match arms separados (read e write) | Clippy sugeriu `0xFF40..=0xFF4B` porque todo o intervalo contíguo é PPU | clippy `-D warnings` |

> Nota: os testes de leitura/escrita passavam antes da implementação porque a rota
> genérica de I/O (`io` array) já atendia $FF42–$FF4B com os mesmos valores do
> hand-off da boot ROM. A implementação transferiu a propriedade para o PPU sem
> alterar comportamento visível. A bateria de mutação (campos do PPU, não do io
> array) confirmou que a rota está correta.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 11/12 | 11/12 (sem regressão) |

Rom de spot check (`01-special.gb`) passando com `--max-cycles 5000000`.
Scoreboard inalterado em relação à 0054 (16/121).

## Bateria de mutação

**Placar: 3/3 pegos, 2/2 controles verdes.**

| # | Tipo | Mutação | Resultado |
|---|---|---|---|
| M1 | catch | `dma: 0xFF` → `0x00` em `Ppu::new()` | `dma_starts_at_0xff_and_is_writable` falhou (`0` != `255`) |
| M2 | catch | `SCY_ADDR => self.scy = value` removido (write no-op) | `scy_starts_at_zero_and_is_writable` falhou (`0` != `127`) |
| M3 | catch | `bgp: 0xFC` → `0x00` em `Ppu::new()` | `bgp_starts_at_0xfc_and_is_writable` falhou (`0` != `252`) |
| C1 | control | `dots: 0` → `dots: 1` em `Ppu::new()` | 29/29 PPU tests continuaram verdes |
| C2 | control | `scy` e `scx` trocados de ordem na struct | 29/29 PPU tests continuaram verdes (nomes de campo, não posição) |

## Decisões de arquitetura

O range `0xFF40..=0xFF4B` agora é todo atendido pelo PPU no `match` do `Bus`.
Isso cobre 12 endereços contíguos — antes eram só 4 (`$FF40`, `$FF41`, `$FF44`,
`$FF45`).

## Notas

- DMA é stub: `self.dma = value` na escrita, `self.dma` na leitura. O trigger de
  transferência (160 M-cycles) fica para M3 (ROADMAP 3.2).
- OBP0/OBP1 seguem a mesma inicialização do `io` array: $00 por escolha (a spec
  marca `??`), replicada no `Ppu::new()`.
- O teste de `obp0_and_obp1_are_uninitialized_in_the_spec_and_zero_by_choice_here`
  em `bus_boot_state.rs` continua passando porque os valores são idênticos em ambas
  as rotas.
