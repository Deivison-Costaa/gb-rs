# Iteração 0017 — endereço absoluto e a página `$FF00`

- **Data:** 2026-07-26
- **Item do roadmap:** 1.4d
- **PR:** #19
- **Duração:** desconhecida (sessão 1, interrompida) + ~40min (sessão 2)
- **Custo reportado:** — <!-- duas sessões interativas, sem JSON por iteração; ver nota 10 -->
- **Turnos:** 2 sessões, de **dois agentes diferentes** — ver abaixo.

## A iteração que sobreviveu ao agente

Esta iteração começou numa sessão de Claude Code que **morreu no meio do
RED→GREEN** — a janela de contexto/crédito fechou com a árvore suja na branch
`iter/0017-cpu-ld-absolute-ff00`: `mcycle.rs` com +201 linhas e a suíte nova de
706 linhas, nada commitado. Foi retomada e concluída por uma sessão de
**Kimi K3 (via OpenCode)**, que encontrou o trabalho no estado em que ficou e
terminou o que faltava. O loop autônomo do projeto troca de motorista a partir
daqui — ver `STATUS.md`, nota 33.

O que sobreviveu à morte da sessão 1: o código, os testes, e — decisivo — os
**comentários no código** que documentavam os dois erros de memória já cometidos
e corrigidos naquela sessão. O que morreu com ela: o contexto vivo (quanto
tempo levou, quantas tentativas, o que mais foi tentado e descartado). Os erros
#1 e #2 abaixo foram **reconstruídos dos comentários e dos testes**, não
observados ao vivo. A lição é a do protocolo inteiro deste projeto, vista do
avesso: o que não vai para o artefato, vai para o ralo.

O que faltava quando a sessão 1 morreu, e a sessão 2 fez:

1. **Um teste da suíte nova estava vermelho — e o bug era do arnês, não do
   emulador** (erro #3 abaixo). Conserto: laços passam a avançar exatamente
   `m_cycles_of(opcode)` passos.
2. **Os controles negativos das três suítes anteriores** (`cpu_ld_r8_block`,
   `cpu_ld_r8_u8`, `cpu_ld_r16mem`) ainda não declaravam os seis opcodes novos
   nas suas listas de `decoded_elsewhere`. É o fluxo desenhado de propósito
   (invariante do `STATUS.md`): quem acrescenta opcode é obrigado a vir
   declarar. A sessão 1 simplesmente não chegou lá — o `cargo test` para no
   primeiro binário que falha, e a suíte nova escondeu as três atrás dela.
3. Este documento, o `STATUS.md`, o ROADMAP e o PR.

## Objetivo

Os seis opcodes de endereçamento absoluto e de página alta — `LD (u16),A` /
`LD A,(u16)` (`$EA $FA`), `LD (FF00+u8),A` / `LD A,(FF00+u8)` (`$E0 $F0`) e
`LD (FF00+C),A` / `LD A,(FF00+C)` (`$E2 $F2`) — fechando o grupo `x8/lsm`
(85 opcodes) e o ROADMAP 1.4. É o sub-item em que **a tabela de
micro-operações se decidia** (adiada desde a 0013 — ver Decisões).

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| gbops | linhas `E0 E2 EA F0 F2 FA` (`Bytes`, `T-cycles`, coluna passo a passo) | `docs/reference/03-opcodes.md` |
| Pan Docs | § Block 3 (seis layouts empilhados sob um cabeçalho só — conversão corrompida, ver cabeçalho da suíte) | `docs/reference/02-cpu.md` |
| Pan Docs | § Moved, Removed, and Added Opcodes (os seis na coluna `GB CPU`; o porquê de existirem: *"no dedicated I/O bus […] new LD (FF00+n) opcodes"*) | `docs/reference/02-cpu.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | `$E2`/`$F2` (`LDH (C),A` / `LDH A,(C)`) têm **2 bytes**, com um operando imediato — o que tabelas de opcode que circulam há décadas listam | A coluna `Bytes` de gbops diz **1**: `C` **é** o operando e já está na CPU; não há byte a buscar. O efeito sai certo nos dois mundos; o que denuncia é o `PC` um byte adiante — e a instrução **seguinte** desalinhada | Reconstruído dos comentários da sessão 1; teste `the_c_indexed_pair_is_one_byte_and_the_next_opcode_follows_immediately` (o programa tem uma 2ª instrução e exige que **ela** rode) |
| 2 | timing | `$EA`/`$FA`: o acesso cai no **M3**, junto com o byte alto do endereço, e o M4 é `internal` — como no `JP u16` | A coluna é `fetch → read(u16:lower) → read(u16:upper) → write(A->(u16))`: o M4 **é o acesso**. Quarta iteração seguida errando *em qual* M-cycle o efeito cai (notas 26, 30, 32) | Reconstruído dos comentários da sessão 1 ("foi o segundo erro do esqueleto da 0017, e dois dos dezenove testes o reprovaram"); as asserções do M3 nos dois testes `the_absolute_*_is_four_m_cycles_*` |
| 3 | timing (do arnês) | O laço "genérico" dos testes `none_of_the_six_*` pode avançar **4 passos fixos** para qualquer um dos seis opcodes | 4 passos executam `$EA`/`$FA` inteiros, mas passam **dois** M-cycles do fim de `$E2`/`$F2`: o excesso busca os `$00` seguintes como `NOP` e o `PC` medido deixa de ser o da instrução. `none_of_the_six_touches_a_register_the_column_does_not_name` falhava contra a implementação **correta** — vermelho do arnês, não do emulador | Sessão 2, ao retomar: 18/19 verdes com o único vermelho incompatível com qualquer implementação (PC esperado exigiria que o tempo parasse). Conserto: `m_cycles_of(opcode)`, espelhando `bytes_of` — e o laço de flags alinhado junto, porque `$85` e `$C3` (os bytes que sobram no programa) não são neutros por princípio, só por acaso |

**Quarta categoria nova em quatro semanas de projeto:** depois de "erro de
memória sobre flags", "erro de memória sobre timing" e "erro de memória sobre
endereçamento", a 0017 traz o **erro de medição** — o teste que mede errado e
reprova o código certo. A nota 8 do `STATUS.md` já dizia que vermelho não é
prova; a 0017 acrescenta: vermelho **no lugar errado** também não é.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| Todas (11) | 0/121 | 0/121 — inalterado; o `gb-cli run` ainda sai `2` (não há máquina completa) |

Testes do workspace: **177 → 196** (+19 da suíte nova; as três suítes
anteriores só ganharam declarações no controle negativo, não testes novos).

## Revisão cruzada (segundo modelo)

- **Modelo:** nenhum — desligada por decisão do operador na virada de motorista
  (`REVIEW=0`; o revisor padrão do `review.sh` era o OpenCode, que agora é o
  **autor**; ver `STATUS.md`, nota 33).
- **Achados:** —

## Decisões de arquitetura

**A "tabela de micro-operações" se decidiu — e a resposta é *não*.** O ROADMAP
marcou no 1.4d o fim da espera de quatro sub-itens (0013 → 1.4a/b/c), e com as
quatro formas de endereçamento do `x8/lsm` na mesa o desenho escolhido é:

- `State` continua crescendo **por variante de forma** (`HighPageC`,
  `HighPageImmediate`, `Absolute`), não por micro-operação genérica. Não nasceu
  `enum MicroOp`.
- O que se generalizou é o **último passo**: `Cpu::access(bus, direction,
  address)`, o passo de acesso compartilhado pelas três formas do 1.4d — a
  primeira vez no projeto em que um passo de M-cycle serve a instruções
  diferentes. A generalização aconteceu onde a repetição **existia** (três
  formas convergindo num acesso), não onde se apostava que viria.

Ou seja: a nota 8 venceu mais uma vez — a abstração que nasceu foi a que os
dados pediram, e ela é uma função de quatro linhas, não uma máquina de
micro-ops. Fica registrado porque o 1.5 (loads de 16 bits + stack) e o 1.6
(ALU) vão tentar de novo: `PUSH`/`POP` têm dois acessos no **mesmo**
registrador `SP` modificado entre eles, e a ALU tem efeito em flags que nenhum
load tem.

## Notas

- **A 0012 previu o formato deste sub-item errado, e a previsão está
  registrada para não ser reescrita.** A armadilha (c) do `STATUS.md` dizia
  que não haveria "uma máscara só" para os seis — certo —, mas sugeria
  reconhecimento por pares; o que o código faz é mais simples: seis braços
  literais no `match` do fetch, porque **não há máscara certa para errar
  frouxo** (qualquer uma que pegue `$E0`, `$E2` e `$EA` leva `$E8`/`$F8`, o
  1.7). O controle negativo varre os 256 e trava os seis exatos.
- **`$FF00 | offset` podia ser `+` sem diferença** — a soma nunca estoura
  (deslocamento de 8 bits, base com os 8 baixos zerados). O `|` carrega a
  intenção "concatenação de página", e um teste cobre `$FF00+$FF = $FFFF`
  (o `IE`), o único endereço onde estouro seria visível se existisse.
- **O chamariz de `B`.** `seed_registers` põe em `B` um deslocamento válido
  **diferente** do de `C` (`$FF12` = `NR12`, com célula e valor inicial), de
  modo que uma implementação que indexe por `B` no lugar de `C` erre o
  endereço sem estourar nada — e o teste seja o que separa as duas.
- **O que se perdeu com a sessão 1:** a contagem de tentativas dos erros #1 e
  #2, a ordem em que aconteceram, e se houve um terceiro erro corrigido em
  silêncio antes dos comentários. O registro empírico deste projeto vive
  exatamente dessas miudezas — e a 0017 é a prova de que elas não sobrevivem
  a `STATUS.md`+git sozinhos quando a sessão morre **antes** do passo 7. O
  remédio estrutural é o que o loop novo faz: uma iteração por processo,
  documentação **dentro** da iteração, e commit como ponto de não-retorno.
