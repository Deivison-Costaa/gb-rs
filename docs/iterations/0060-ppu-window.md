# Iteração 0060 — Window por scanline

- **Data:** 2026-07-27
- **Item do roadmap:** 3.4

## Objetivo

Renderizar a Window por scanline: condição Y, contador interno de linha (`window_line`), tilemap independente (LCDC.6), posicionamento via WX/WY, casos especiais WX=0 (shift SCX%8) e WX=166 (bug DMG).

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | Window | `docs/reference/06-ppu.md` |
| Pan Docs | Window behavior | `docs/reference/06-ppu.md` |
| Pan Docs | LCDC | `docs/reference/06-ppu.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | O contador de linha da window seria `LY - WY` (diferença simples) quando a window estivesse visível. | O contador de linha é independente de LY e só incrementa quando a window é efetivamente renderizada na scanline. Scanlines onde a window não está ativa (WY > LY, LCDC.5=0, WX fora de faixa) não incrementam o contador. | Teste `window_line_counter_increments_within_tile_when_visible`: window_line=0 em LY=0, window_line=1 em LY=1 (verificado via tile com linhas alternadas). |
| 2 | timing | `window_y_condition` inicializado como `false` em `new()` cobriria todos os casos — a verificação dentro do bloco de wrapping de dots daria conta. | A primeira scanline (LY=0, dots=0) nunca passa pelo bloco de wrapping. A condição Y precisa ser avaliada quando `dots == 0` no início de `tick()`, não só no wrap. | Teste `window_covers_entire_screen_when_wx7_wy0_and_lcdc5_set` — com WY=0 na primeira scanline, a window nunca ativava e o teste recebia shade 3 do BG em vez de shade 0 da window. Corrigido movendo a verificação para `dots == 0` no topo de `tick()`. |
| 3 | flags | WX=166 seria um caso normal: WX−7 = 159, mostrando 1 pixel de window na borda direita. | WX=166 no DMG dispara um bug: a window cobre a tela inteira, com offset vertical de 1 scanline (`window_line + 1`). | Teste `wx_166_bug_covers_entire_screen_instead_of_one_pixel` — sem o caso especial, o pixel 0 mostrava BG (shade 3) e não window (shade 0). |
| 4 | endereçamento | A window compartilharia o tilemap base do background (LCDC.3), usando o mesmo `tilemap_base` para ambos. | A window tem seu próprio bit de seleção de tilemap (LCDC.6), independente do BG (LCDC.3). Os dois podem apontar para $9800 ou $9C00 separadamente. | Teste `window_uses_its_own_tilemap_when_lcdc6_set` — com BG=$9800 (tile 0, shade 3) e window=$9C00 (tile 1, shade 0), o framebuffer mostra shade 0 quando LCDC.6=1. |
| 5 | API-Rust | `window_left()` deveria retornar a posição baseada só em WX, e o loop de renderização verificaria `win_enabled` separadamente. | A checagem de enable (LCDC.5 e Y condition) foi integrada em `window_left()` retornando `Option<i16>` — se a window não está habilitada, `window_left()` retorna `None` e o loop pula a window inteira. | Depuração: o primeiro draft não verificava `win_enabled` no loop de pixel, fazendo a window renderizar mesmo com LCDC.5=0. O debug externo mostrou `fb[0]=0` (window) quando deveria mostrar shade 3 (BG) nos testes negativos. |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 12/12 | 12/12 |
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

Placar de ROMs inalterado. Testes unitários: 698 → **707** (9 novos em `ppu_window_scanline.rs`).

## Bateria de mutação

| # | Mutação | Resultado |
|---|---|---|
| M1 | Incremento de `window_line` comentado | `window_line_counter_increments_within_tile_when_visible` **pegou** |
| M2 | Verificação `dots == 0` removida do `tick()` | 5 testes de window **pegaram** |
| M3 | Caso especial WX=166 removido | `wx_166_bug_covers_entire_screen_instead_of_one_pixel` **pegou** |
| C1 | `MODE_3_BASE` de 172 para 173 | **Verde** nos testes de window, **pegou** em `ppu_ly_stat_mode` (3 testes) |

**3/3 pegos, 1/1 controle verde** (a quebra nos de LY prova que o controle não passou despercebido).

## Revisão cruzada (segundo modelo)

- **Modelo:** não houve — iteração sem revisor humano.
- **Achados:** —
- **Procedentes:** —
- **Falso positivo mais interessante:** —

## Decisões de arquitetura

1. **Condição Y gerenciada em `tick()`.** `window_y_condition` é avaliada no início de cada scanline (`dots == 0` no topo de `tick()`) e limpa no início do VBlank. A avaliação no topo (não só no bloco de wrapping) foi necessária para cobrir a primeira scanline (LY=0, dots=0), que nunca passa pelo wrapping.

2. **`window_line` como campo do `Ppu`.** O contador interno de linha da window é incrementado ao final de `render_background_scanline()` se a window foi renderizada na scanline atual. É resetado no início do VBlank junto com a condição Y.

3. **`window_left()` como gate único.** A função retorna `Option<i16>` incorporando todas as verificações de enable (LCDC.5, Y condition, faixa de WX). Se a window não deve renderizar, retorna `None` e o loop de pixel a ignora completamente. Isso evita duplicação de lógica de enable no loop.

4. **Casos especiais como flags no loop.** WX=0 (shift SCX%8) e WX=166 (offset vertical +1) são calculados como booleanos antes do loop e aplicados condicionalmente dentro dele (`win_x` para WX=0, `tile_row_base` para WX=166).

## Notas

- Os testes negativos (`window_disabled_when_lcdc5_clear_shows_background_only`, `window_not_visible_when_wy_gt_ly_shows_background`) não têm cobertura completa de mutação dentro desta suíte porque BG e window compartilham o mesmo tile nesses cenários: ambos renderizam tile 0 do $9800, produzindo o mesmo shade. A M1 (remover Y condition de `window_left`) sobreviveu a essa suíte — a mutação foi detectada via debug externo durante o desenvolvimento. A cobertura cruzada virá dos testes de sprite (3.5) que usam tilemaps separados para BG/window.

- A WX=0 também sobreviveu à M1 (remover caso especial) porque sem o caso especial `wx==0`, `window_left()` retorna `Some(-7)` (WX−7 com signed i16), que ainda cobre a tela inteira. A diferença é o deslocamento das coordenadas X do tile, que os testes atuais não detectam por usarem tile uniforme. Testes com tiles não-uniformes (padrão checker/rainbow na window) capturariam essa mutação.
