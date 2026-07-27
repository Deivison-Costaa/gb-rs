# Iteração 0067 — gb-desktop: janela winit + framebuffer + teclado

- **Data:** 2026-07-27
- **Item do roadmap:** 4.4

## Objetivo

Transformar o stub `gb-desktop` em um frontend funcional: janela winit com renderização do framebuffer da PPU via `pixels` (60 fps) e mapeamento de teclas para os botões do Game Boy.

## Spec consultada

Nenhuma spec de hardware — é uma iteração de I/O (R3). A referência foi a API do `Bus` (framebuffer, key_down/key_up) e o padrão do `gb-cli` (load de ROM, criação de Bus + Cpu, loop de step).

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | `winit 0.29` + `pixels 0.13` compilariam juntos sem conflito de versão | `winit 0.29` usa `raw-window-handle 0.6`; `pixels 0.13` usa `raw-window-handle 0.5`. Traits incompatíveis — `HasRawWindowHandle`/`HasRawDisplayHandle` não são implementados para a `Window` de winit 0.29 na versão que o pixels espera. | Erro de compilação (`E0277`) ao rodar `cargo check` |
| 2 | API-Rust | A API de eventos do winit é estável entre 0.28 e 0.29 | `winit 0.28` usa `VirtualKeyCode`, `Event::MainEventsCleared`, `Event::RedrawRequested(WindowId)`, callback com 3 argumentos (`event, _, control_flow`), `KeyboardInput` sem campo `event`. `winit 0.29` migrou para `KeyCode`, `PhysicalKey`, `Event::AboutToWait`, `Event::WindowEvent { event: WindowEvent::RedrawRequested }`, callback com 2 argumentos, `KeyboardInput` com campo `event`. Reescrita completa do loop de eventos. | Erro de compilação (7 erros) após downgrade para 0.28 |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 12/12 | 12/12 |
| Todos os demais | inalterados | inalterados |
| Workspace tests | 790 | 803 (13 novos em `gb-desktop`) |

Nenhuma regressão no scoreboard (18/121 → 18/121).

## Bateria de mutação

| Mutante | Descrição | Pego por |
|---|---|---|
| MUT1 | `Z → None` em vez de `Some(Key::A)` | `map_key_z_mapeia_para_key_a` |
| MUT2 | `Right → Some(Key::Left)` em vez de `Some(Key::Right)` | `map_key_seta_direita_mapeia_para_key_right` |
| MUT3 | `FRAMEBUFFER_PALETTE[0]` = `[0xFF,0xFF,0xFF,0xFF]` em vez de verde | `framebuffer_to_rgba_converte_zero_para_branco_esverdeado` |
| MUT4 | `color as usize + 1` off-by-one no índice da paleta | `framebuffer_to_rgba_converte_zero_para_branco_esverdeado` (e `_tres`) |
| MUT5 | `copy_from_slice` escreve 3 bytes em vez de 4 | 3/4 testes de framebuffer falharam |

**Placar:** 5/5 pegos, 2/2 controles verdes.

Controles: renomear parâmetro `fb` → `source` (0 falhas); reordenar braços do `match` (0 falhas).

## Decisões de arquitetura

- **Versões:** `winit 0.28` + `pixels 0.13` — a combinação com `raw-window-handle 0.5` comum é a que resolve sem conflito. `winit 0.29` exigiria `pixels` com `raw-window-handle 0.6`, que não existe na série 0.13.
- **Timing:** 17.556 M-cycles por frame (`154 linhas × 456 dots / 4 dots por M-cycle`), ≈60 fps. O loop renderiza um frame inteiro entre `RedrawRequested`, sem delta timing — a latência é irrelevante para o primeiro marco.
- **Paleta de tela:** verde-clássico DMG (`#E0F8D0` / `#88C070` / `#346856` / `#081820`), fixa. A PPU já aplica BGP/OBP no framebuffer — a paleta aqui é só a conversão de 2-bit para RGB para exibição.
- **Estrutura:** `main.rs` → `run::execute()` (espelhando o `gb-cli`). `map_key` e `framebuffer_to_rgba` são funções livres, testáveis sem janela.

## Notas

- O `gb-desktop` puxa 241 crates transitivos (winit → wgpu → ash, wayland, x11). A CI de `check` passou de ~30s para ~1min30s. Isso é inerente ao crate gráfico e não há o que otimizar sem trocar de backend.
- `save` (.sav) ao fechar a janela não foi implementado — o `EventLoop::run` captura o `Bus` por move e o drop no fim da execução não tem acesso ao path. Fica para iteração seguinte.
