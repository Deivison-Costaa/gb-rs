# Iteração 0077d — DMA de OAM ($FF46) e prioridade BG-over-OBJ por índice de cor

- **Data:** 2026-07-30
- **Item do roadmap:** nenhum — conserto fora da escada (fecha o stub do 3.1b e corrige o 3.5)

## Objetivo

Fazer sprite aparecer em jogo real: implementar a transferência da DMA de OAM
(até aqui `$FF46` era só um registrador que guardava o valor) e corrigir a
prioridade BG-over-OBJ, que comparava shade em vez de índice de cor.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | OAM DMA Transfer | `docs/reference/06-ppu.md` |
| Pan Docs | BG-to-OBJ Priority | `docs/reference/06-ppu.md` |

## O que estava quebrado

O 3.1b entregou `$FF46` como **stub declarado** ("DMA stub") e nenhuma iteração
posterior fechou a lacuna. O 3.5 renderizou sprites e passou nos testes
unitários porque eles escrevem OAM direto pelo barramento — caminho que jogo
nenhum usa. Super Mario Land escreve `$C0` em `$FF46` e espera 160 M-cycles: a
OAM ficava zerada, `Y=0` põe todo objeto fora da tela, e o jogo rodava sem um
único sprite. O placar não pegou: dmg-acid2 monta a OAM sem DMA, e nenhuma ROM
da bateria exercita `$FF46`.

O segundo bug é do próprio 3.5: a prioridade lia `framebuffer[x]`, que já é o
shade depois da BGP. A spec diz **índice de cor** 1–3. Com a BGP mapeando cor 0
em shade 3 (comum em tela de fundo escura) o sprite sumia atrás do BG; com cor
3 mapeada em shade 0, sprite com bit 7 vazava por cima do BG.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | Fonte da DMA passa pelo `Bus::read` normal | A DMA tem barramento próprio: não a bloqueia o modo da PPU, e fonte acima de `$DF` cai no echo da WRAM | espec lida antes de implementar |
| 2 | timing | Copiar os 160 bytes de uma vez na escrita de `$FF46` | 160 M-cycles, um byte por M-cycle; a OAM fica inacessível à CPU nesse intervalo | espec lida antes de implementar |
| 3 | nenhum | — | — | — |

> O erro que custou tempo não é de spec: é de **método**. Testes de PPU que
> escrevem OAM pelo barramento validam o renderizador e mentem sobre o console.
> A lacuna só apareceu quando uma ROM comercial rodou de fato.

## Implementação

- `Bus` ganhou `OamDma { active, source_high, index }` e `tick_dma()`, chamado
  por `Cpu::step` junto de `tick_timer`/`tick_ppu`/`tick_apu` (R2).
- Escrita em `$FF46` continua indo para a PPU (o registrador segue legível) e
  arma a transferência do zero — inclusive por cima de uma DMA em curso.
- `dma_read` desvia do bloqueio de VRAM/OAM da PPU.
- Leitura e escrita da CPU na OAM passam por `cpu_can_access_oam()`.
- `Ppu` ganhou `bg_color: [u8; 160]`, gravado nos três caminhos do background
  (LCDC.0 = 0, window, BG) e lido pela prioridade em `render_sprites`.

**Limitação assumida:** o conflito de barramento do DMG (durante a DMA a CPU só
enxerga HRAM) **não** foi implementado. Jogos esperam a transferência com o
laço em HRAM, então a diferença é inobservável neles; ROM de teste que meça o
conflito vai reprovar. Registrado aqui para não parecer descuido depois.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| dmg-acid2 | 1/1 (hash `f844ea76…`) | 1/1 (mesmo hash) |
| Testes | — | **+8** (6 em `oam_dma.rs`, 2 em `ppu_sprites.rs`) |

O hash do dmg-acid2 não mudou: a ROM monta a OAM sem DMA e usa BGP identidade,
então nenhuma das duas correções a toca. O total do workspace não foi anotado
aqui porque a árvore tinha trabalho não commitado do 0077c junto.

Verificação que importa e que o placar não mede: Super Mario Land, 20 M ciclos,
framebuffer despejado em ASCII. Antes, OAM inteira zerada e nenhum objeto.
Depois, objetos reais na OAM (`Y=128, X=43, tile 0`…) e o Mario desenhado na
tela de título.

## Notas

Vale um slide: **o placar de ROMs de teste ficou verde por 17 iterações sobre um
emulador que não desenhava um sprite sequer em jogo comercial.** As duas coisas
não se contradizem — a bateria mede o que ela cobre, e `$FF46` não estava
coberto por nenhuma ROM baixada. O custo de "stub declarado" é esse: fica
honesto no commit e invisível no placar.
