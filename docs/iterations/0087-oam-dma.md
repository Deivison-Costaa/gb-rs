# Iteração 0087 — DMA de OAM ($FF46) de verdade

- **Data:** 2026-07-30
- **Item do roadmap:** 3.8

## Objetivo

Fazer sprite aparecer em jogo real: implementar a transferência da DMA de OAM,
que até aqui nunca aconteceu, com o bloqueio de barramento que ela impõe à CPU.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | OAM DMA Transfer | `docs/reference/06-ppu.md` |
| Pan Docs | OAM DMA bus conflicts | `docs/reference/06-ppu.md` |
| Pan Docs | BG-to-OBJ Priority | `docs/reference/06-ppu.md` |

## O que estava quebrado

O 3.1b entregou `$FF46` como *stub* deliberado e o 3.8 registrou a consequência
em 27/07: **nenhum jogo mostra sprite algum**. Confirmado aqui rodando o Super
Mario Land headless — a OAM ficava inteira zerada, e `Y = 0` põe todo objeto
fora da tela.

Junto veio um segundo bug, este do 3.5: a prioridade BG-over-OBJ lia
`framebuffer[x]`, que já passou pela BGP. A spec diz **índice de cor** 1–3. Com
a BGP mapeando cor 0 em shade 3 o sprite sumia atrás do BG; com cor 3 mapeada em
shade 0, sprite com bit 7 vazava por cima. Achado ao ler o 3.5 para escrever
este item — não estava na descrição do 3.8.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | A fonte da DMA passa pelo `Bus::read` normal | A DMA tem barramento próprio: não a bloqueia o modo da PPU, e fonte acima de `$DF` cai no echo da WRAM | spec lida antes de implementar |
| 2 | timing | Copiar os 160 bytes de uma vez na escrita de `$FF46` | 160 M-cycles, um byte por M-cycle | spec lida antes de implementar |
| 3 | **escopo** | Dava para entregar o item sem o conflito de barramento, deixando-o registrado como limitação | O 3.8 exige em letra: "a CPU só enxerga HRAM enquanto ela corre — R1 vale aqui" | reler o item do ROADMAP antes de fechar a caixa |

> O erro #3 é o que vale registrar. A implementação estava fechada e a limitação
> já escrita no doc **antes** de o texto do 3.8 ser aberto. O item já antecipava
> exatamente essa tentação — "o 3.1b entregou um *stub* deliberado, fechou a
> caixa e nunca abriu esta". Era o mesmo atalho, uma escada abaixo.

## Implementação

- `Bus` ganhou `OamDma { active, source_high, index }` e `tick_dma()`, chamado
  por `Cpu::step` junto de `tick_timer`/`tick_ppu`/`tick_apu` (R2). Escrita em
  `$FF46` segue indo à PPU (o registrador continua legível) e arma a
  transferência do zero, inclusive por cima de uma DMA em curso.
- `dma_read` desvia do bloqueio de VRAM/OAM da PPU.
- `dma_blocks` fecha ROM, VRAM, SRAM, WRAM, echo, OAM e a região proibida
  enquanto a transferência corre: leitura devolve `OPEN_BUS`, escrita é
  engolida. Sobra a HRAM — que é por onde a rotina de DMA dos jogos roda.
- `Ppu` ganhou `bg_color: [u8; 160]`, gravado nos três caminhos do background
  (LCDC.0 = 0, window e BG) e lido pela prioridade em `render_sprites`.

## Divergências da spec, e por quê

1. **I/O e IE ficam fora do bloqueio.** A spec diz "the CPU can access only
   HRAM". Aqui a faixa `$FF00–$FF7F` e o `$FFFF` continuam acessíveis: são
   internos à CPU, não passam pelo barramento externo que a DMA toma, e fechá-los
   tornaria `IF`/`IE` ilegíveis no dispatch de interrupção deste emulador. Há
   teste fixando a decisão (`a_dma_nao_bloqueia_a_faixa_de_io`).
2. **O valor lido durante o conflito é `OPEN_BUS`.** A spec não fixa o que a CPU
   lê de uma região tomada — o DMG devolve o byte que a própria DMA está
   buscando. `OPEN_BUS` é a convenção que o resto do barramento já usa, e não
   inventa hardware.
3. **Sem atraso de partida.** O primeiro byte sai no M-cycle seguinte à escrita,
   e a transferência fecha em `escrita + 160`. As ROMs da mooneye que medem o
   atraso exato (`oam_dma_start`, `oam_dma_timing`) não distinguem hoje: a suíte
   inteira está em 0/66 por outros motivos.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| Total | 18/121 | 18/121 |
| dmg-acid2 | 1/1 | 1/1 (hash `f844ea76…` idêntico) |

Placar inalterado é o resultado esperado, e é o dado interessante: **nenhuma ROM
da bateria exercita `$FF46`**. O `dmg-acid2` é a única que escreve na OAM, e
escreve direto.

A verificação que valeu foi outra: Super Mario Land e Tetris rodados headless com
o framebuffer despejado em ASCII. Antes, OAM zerada e nenhum objeto. Depois, os
dois passam da tela de título com sprites desenhados — inclusive com o bloqueio
de barramento ligado, que é o caminho em que um erro de janela trava a CPU em
`$FF` (`RST 38h`) em vez de degradar em silêncio.

## Notas

Vale um slide: **o placar ficou verde por 17 iterações sobre um emulador que não
desenhava um sprite sequer em jogo comercial.** Não é contradição — a bateria
mede o que ela cobre, e `$FF46` não estava coberto por nenhuma das 121 ROMs. O
custo do *stub* declarado é esse: honesto no commit, invisível no placar, e só
aparece quando alguém roda um jogo de verdade.
