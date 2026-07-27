# Iteração 0059 — Background por scanline

- **Data:** 2026-07-27
- **Item do roadmap:** 3.3

## Objetivo

Renderizar o background por scanline durante o Mode 3 — tilemap, tiledata, endereçamento signed/unsigned, SCX/SCY e paleta BGP. Primeira iteração que produz pixels de fato no framebuffer.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | VRAM Tile Data | `docs/reference/06-ppu.md` |
| Pan Docs | VRAM Tile Maps | `docs/reference/06-ppu.md` |
| Pan Docs | Scrolling | `docs/reference/06-ppu.md` |
| Pan Docs | Palettes (BGP) | `docs/reference/06-ppu.md` |
| Pan Docs | LCDC | `docs/reference/06-ppu.md` |
| Pan Docs | Rendering overview | `docs/reference/06-ppu.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | SCY=8 leria a linha 1 do mesmo tile (deslocamento vertical de 1 linha dentro do tile), porque bg_y = 8 parecia indicar o 8º pixel vertical do tile. | `bg_y = (ly + SCY) % 256` = 8 → `tile_row = 8/8 = 1`, `line_in_tile = 8%8 = 0`. SCY desloca a viewport no tilemap inteiro: os 8 pixels de deslocamento mudam a linha do tilemap, não a linha dentro do tile. Para deslocar 1 linha dentro do mesmo tile, SCY=1. | teste unitário `scy_shifts_pixels_vertically` — a primeira versão do teste esperava shade 3 com SCY=8 e recebeu 0; corrigido para testar SCY=0 vs SCY=1. |

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
| mooneye acceptance | 0/66 | 0/66 |
| mooneye acceptance (outros) | 0/9 | 0/9 |

Placar de ROMs inalterado (121 ROMs, 0 passando — framebuffer não é verificado por ROMs de teste ainda). Testes unitários: 698 (eram 692 na 0058 — 6 novos em `ppu_background_scanline.rs`).

## Revisão cruzada (segundo modelo)

- **Modelo:** não houve — iteração sem revisor humano.
- **Achados:** —
- **Procedentes:** —
- **Falso positivo mais interessante:** —

## Decisões de arquitetura

1. **Framebuffer no `Ppu`, exposto via `Bus`.** O `Ppu` é dono do `framebuffer: [u8; 160 * 144]` e o `Bus` expõe `pub fn framebuffer(&self) -> &[u8; 160*144]`. Segue o padrão do projeto: `Ppu` é interno (pub(crate)), `Bus` é a fachada pública. Cada pixel armazena o shade final (0–3) após aplicação da paleta BGP.

2. **Renderização acionada por sinal em `PpuSignals`.** `tick()` detecta a entrada no Mode 3 (`old_dots < 80 && self.dots >= 80 && ly < 144`) e levanta `begin_mode3`. O `Bus::tick_ppu()` reage chamando `ppu.render_background_scanline(&self.vram)`. Isso mantém o VRAM no `Bus` e o `Ppu` sem dependência de leitura de memória própria — o `Bus` é o dono do estado compartilhado.

3. **Renderizador por scanline, não FIFO.** Todos os 160 pixels da scanline são calculados de uma vez na entrada do Mode 3, usando a fórmula direta: `bg_x = (pixel_x + SCX) % 256`, `bg_y = (LY + SCY) % 256`, tilemap → tile index → tile data → extração de cor → BGP. A invariante do projeto é renderizador por scanline; o pixel FIFO do Pan Docs foi lido mas não implementado. Os penalties de Mode 3 (SCX % 8, window, OBJ) alongam o Mode 3 na especificação mas não afetam a saída de pixels no modelo por scanline — só o modo permanece em Mode 3 pelos dots corretos.

4. **Shade final armazenado no framebuffer, não o color index.** O BGP é aplicado durante `render_background_scanline` e o framebuffer contém o valor de shade (0–3) — é o que o `gb-desktop` vai consumir. Se a paleta mudar mid-frame, scanlines já renderizadas não são afetadas (comportamento correto para DMG).

## Notas

- O `lcdc0_disabled_fills_screen_with_bgp_color_0` testa que LCDC.0=0 preenche com a cor 0 do BGP (não branco fixo). O Pan Docs diz "blank (white)" para DMG, mas como o BGP mapeia cor 0 para um shade configurável e o framebuffer armazena o shade pós-paleta, o teste usa BGP=0x1B (cor 0 mapeia para shade 3/preto) e verifica que todos os pixels são shade 3. O "white" do Pan Docs refere-se à cor na tela física do Game Boy com a paleta padrão; com BGP reprogramado, "blank" significa "usa a entrada 0 da paleta".

- A bateria de mutação (5/5 pegos) cobriu: tilemap base, signed/unsigned, ordem MSB/LSB, SCX e LCDC.0. A mutação de `tilemap_base` também quebrou `signed_addressing_maps_tile_index_128_to_block_1`, que não pretendia testar tilemap mas usa LCDC.3=0 — o tilemap errado ($9C00 vazio em vez de $9800 preenchido com tile 128) fez todos os pixels serem shade 0, e o assert esperava shade 3 do bloco 1.

- O `lcdc3_selects_alternate_tilemap` verifica que LCDC.3=1 seleciona $9C00. O primeiro tile do $9C00 é tile 1 (cor 3 em todos os pixels), e o segundo é tile 0 (padrão checker, pixel 0 = shade 0). O teste confirma ambos — isso cobre a indexação do tilemap (tile_row * 32 + tile_col) indiretamente, já que o segundo tile está no offset 1 do tilemap.
