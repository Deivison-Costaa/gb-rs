# Iteração 0062 — Bloqueio de acesso a VRAM/OAM por modo da PPU

- **Data:** 2026-07-27
- **Item do roadmap:** 3.6

## Objetivo

Bloquear leitura (devolve $FF) e escrita (ignorada) em VRAM durante Mode 3 e em OAM durante Mode 2/3, conforme § Accessing VRAM and OAM.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | Accessing VRAM and OAM | `docs/reference/06-ppu.md` |
| Pan Docs | PPU modes | `docs/reference/06-ppu.md` |
| Pan Docs | LCDC.7 (PPU enable) | `docs/reference/06-ppu.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | OAM é acessível em Mode 2 (OAM scan) — eu assumi que só Mode 3 bloqueia OAM, porque o CPU e a PPU leriam faixas diferentes da memória | OAM é bloqueado em Mode 2 **e** Mode 3; só Mode 0/1 permitem acesso. O PPU lê OAM inteira durante o scan, não dá para o CPU acessar concorrentemente | Testes `oam_read_returns_ff_during_mode_2` e `oam_write_is_ignored_during_mode_2` pegaram; também quebrou `bus_oam.rs` e `ppu_sprites.rs` que escreviam OAM em Mode 2 |
| 2 | read-value | Leitura bloqueada devolveria $00, por analogia com a região NotUsable ($FEA0-$FEFF) | A spec diz "typically $FF"; o valor correto é OPEN_BUS ($FF), e a decisão de usar $00 para NotUsable é independente | Revisão da spec antes da implementação — não chegou a virar código errado |
| 3 | coverage | A guarda `lcdc & 0x80 == 0` em `is_vram_accessible`/`is_oam_accessible` parecia cobrir um caso distinto do `current_mode()` | `current_mode()` já retorna `MODE_HBLANK` quando o PPU está desligado — a guarda é redundante. Mutante M7 sobreviveu sem consequência funcional | Bateria de mutação (M7 e C1 sobreviveram porque o comportamento é idêntico) |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 11/12 | 11/12 |
| blargg total | 17/121 | 17/121 |

## Bateria de mutação

| # | Mutação | Pego? | Testes que falharam |
|---|---|---|---|
| M1 | `is_vram_accessible` sempre `false` | Sim | `vram_is_accessible_during_mode_{0,2}`, `vram_is_accessible_when_ppu_disabled`, `vram_write_is_ignored_during_mode_3` |
| M2 | `is_oam_accessible` sempre `false` | Sim | `oam_is_accessible_during_mode_0`, `oam_is_accessible_when_ppu_disabled` + 4 `bus_oam.rs` |
| M3 | `is_vram_accessible` sempre `true` | Sim | `vram_read_returns_ff_during_mode_3`, `vram_write_is_ignored_during_mode_3` |
| M4 | `is_oam_accessible` sempre `true` | Sim | `oam_read_returns_ff_during_mode_{2,3}`, `oam_write_is_ignored_during_mode_{2,3}` |
| M5 | VRAM bloqueada em Mode 2 em vez de Mode 3 | Sim | `vram_is_accessible_during_mode_{0,2}`, `vram_is_accessible_when_ppu_disabled`, `vram_read_returns_ff_during_mode_3`, `vram_write_is_ignored_during_mode_3` |
| M6 | OAM bloqueada só em Mode 3 (não em Mode 2) | Sim | `oam_read_returns_ff_during_mode_2`, `oam_write_is_ignored_during_mode_2` |
| M7 | `is_vram_accessible` sem guarda `lcdc & 0x80` | Não | `current_mode()` já devolve `MODE_HBLANK` com PPU desligado |
| C1 | `is_oam_accessible` sem guarda `lcdc & 0x80` (controle) | Não (verde) | Idem — `current_mode()` cobre o caso |

**Placar: 6/7 pegos, 1/1 controle verde.** O mutante sobrevivente (M7) é inócuo: `current_mode()` é a fonte autoritativa do modo e já retorna HBLANK com PPU desligado.

## Decisões de arquitetura

- Os métodos `is_vram_accessible` e `is_oam_accessible` ficaram em `Ppu` (não em `Bus`) porque a decisão depende de estado interno da PPU (`dots`, `ly`, `lcdc`). O `Bus` só consulta e age.
- A guarda `lcdc & 0x80 == 0` foi mantida por clareza semântica, mesmo sendo redundante com `current_mode()`.
- O valor de retorno para leitura bloqueada é `OPEN_BUS` ($FF), consistente com o resto do barramento.
- Os testes em `bus_oam.rs` foram adaptados para usar PPU desligado (`bus_oam_accessible`), e `ppu_sprites.rs` passou a iniciar com PPU desligado para setup dos dados de OAM.

## Notas

Nenhuma nota nova. O impacto nos testes existentes (`bus_oam.rs`, `ppu_sprites.rs`) era esperado porque o comportamento de OAM em Mode 2 nunca foi o correto — os PRs anteriores da PPU nunca exercitaram OAM via barramento com a PPU ligada.
