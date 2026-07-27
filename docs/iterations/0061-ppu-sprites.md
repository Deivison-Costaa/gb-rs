# Iteração 0061 — Sprites (OAM scan, limite 10/linha, prioridade, flip X/Y, modo 8×16)

- **Data:** 2026-07-27
- **Item do roadmap:** 3.5

## Objetivo

Renderizar sprites (OBJ) da OAM sobre o framebuffer: OAM scan com limite de
10 por scanline, prioridade por X no DMG, flip X/Y, modo 8×16 e paletas OBP0/OBP1.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | Object Attribute Memory (OAM) | `docs/reference/06-ppu.md` |
| Pan Docs | Rendering overview | `docs/reference/06-ppu.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | Dois tiles consecutivos (NN, NN+1) em modo 8×16 | LSB é ignorado: topo = NN & $FE, base = NN \| $01 | espec escrito durante implementação — nunca virou código errado |
| 2 | prioridade | Prioridade de desenho por índice OAM no DMG | DMG é por X (menor X = maior prioridade), desempate por índice OAM; CGB que é só por índice | espec lida antes de implementar — nunca virou código |
| 3 | nenhum | — | — | — |

> Os três erros que apareceram foram de teste (valores de paleta, posição Y,
> endereçamento de tile), não de spec. Corrigidos no próprio passo 4.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 11/11 | 11/11 |
| blargg (todas) | 17/121 | 17/121 |
| Testes do workspace | 707 | 717 |

## Bateria de mutação

| # | Mutação | Resultado |
|---|---|---|
| M1 | `lcdc & 0x02 != 0` → `false` (sprites nunca renderizam) | 6/10 pegos |
| M2 | `sprite_y <= ly` → `sprite_y < ly` (perde primeira linha do sprite) | 6/10 pegos |
| M3 | `attr & 0x20 != 0` → `false` (X flip ignorado) | 1 pego (`sprite_com_flip_x`) |
| M4 | `attr & 0x40 != 0` → `false` (Y flip ignorado) | 1 pego (`sprite_com_flip_y`) |
| M5 | `a.0.cmp(&b.0)` → `b.0.cmp(&a.0)` (prioridade invertida) | **sobreviveu na 1ª tentativa** — `sprite_menor_x_tem_prioridade` testava pixel 2 (sem sobreposição); corrigido para pixel 6, repetido: 1 pego |
| M6 | `sprite_wins` → `true` (BG-over-OBJ sempre deixa sprite ganhar) | 1 pego (`bg_over_obj`) |
| M7 | `palette_bit != 0` → `false` (OBP1 ignorado, sempre OBP0) | 1 pego (`atributo_palette`) |
| C1 | `MAX_SPRITES: 10` → `11` (controle: limite não testado a <11) | 10/10 verde |

**Placar: 7/7 pegos, 1/1 controle verde** (M5 sobreviveu na 1ª tentativa — buraco de cobertura no próprio teste, não na spec).

## Decisões de arquitetura

- `render_scanline(vram, oam)` substitui `render_background_scanline(vram)`
  como ponto de entrada chamado por `Bus::tick_ppu()`. Renderiza BG/Win
  primeiro, depois sobrepõe sprites — a ordem natural das camadas do GB.
- OAM scan ocorre dentro de `render_sprites`, não em `tick()`: como o
  renderizador é por scanline (não por dot), o scan do Mode 2 pode ser
  feito junto da renderização do Mode 3 sem perda de correção funcional.
- Sprites selecionados são armazenados em array local `[(u8;5); 10]`
  (x, y, tile, attr, oam_index), ordenados por X (crescente) + desempate
  por índice OAM.
- O flip Y em modo 8×16 espelha o OBJ inteiro (16 pixels), não cada
  metade de 8 pixels — confirmado contra a spec ("Entire OBJ is
  vertically mirrored").
- `render_background_scanline` mantém o `return` quando LCDC.0=0 (BG
  desligado preenche com BGP cor 0 e sai); sprites são renderizados
  depois, pelo chamador `render_scanline`, sobre o fill.

## Notas

- O teste `sprite_menor_x_tem_prioridade` testava pixel 2, que não é ponto
  de sobreposição com sprite 0 em X=12 (cobre pixels 4–11). Corrigido para
  pixel 6 — o mutante M5 mostrou o buraco (nota 8: "escrever teste antes
  não torna o teste testado").
- O `bg_over_obj_esconde_sprite_quando_bg_cores_1_3` passa com sprites
  desabilitados (BG mostra cor 3 sem sprite sobrepor). O teste é correto
  na direção que importa (pega "sprite sempre ganha"), mas a direção
  inversa ("sprites desligados acidentalmente passam") é uma limitação
  — o controle negativo de BG-over-OBJ não tem guarda de ausência.
- 10 testes novos em `ppu_sprites.rs`. 717 testes totais no workspace
  (eram 707).
- O scoreboard permanece em 17/121 — as ROMs de sprite ainda não passam
  porque faltam bloqueio de VRAM/OAM (3.6) e o OAM bug não está implementado.
