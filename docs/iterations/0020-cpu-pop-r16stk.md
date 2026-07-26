# Iteração 0020 — `POP r16stk`, o bloco `11 rr 0001`

- **Data:** 2026-07-26
- **Item do roadmap:** 1.5c
- **PR:** #24
- **Duração:** —
- **Custo reportado:** —  <!-- sessão interativa; ver STATUS.md, nota 10 -->
- **Turnos:** 1

## Objetivo

`$C1 $D1 $E1 $F1` em 3 M-cycles, fechando a pilha que o `PUSH` da 0019 abriu.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| gbops | linhas `C1` `D1` `E1` `F1` | `docs/reference/03-opcodes.md` |
| Pan Docs | § CPU Instruction Set, cabeçalho `pop r16stk` | `docs/reference/02-cpu.md` |

A coluna, transcrita: `fetch → read((SP++)->C) → read((SP++)->B)`. E a linha do
`$F1` é a única das quatro com `Z N H C` nas colunas de flag; as outras três têm
`-`.

## Erros de primeira tentativa

> Categorias: `flags`, `timing`, `endereçamento`, `borrow-checker`, `API-Rust`,
> `nenhum`.

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | `Pair::set(&mut Cpu, u16)`, copiado do arquivo do `PUSH` | — | erro de compilação: o teste do snapshot precisa aplicar o par a um `Registers` solto, não a uma `Cpu` |
| 2 | nenhum | — | — | — |

**As três armadilhas de hardware não aconteceram, e o motivo é registrável.**
O campo `Próxima tarefa` do `STATUS.md` nomeou as três antes de eu abrir a spec:
(a) `(SP++)` é pós-incremento e não o espelho literal do `(--SP)`; (b) a seta
`->lower` põe meia metade do par por M-cycle; (c) `POP AF` não mascara o nibble
baixo de `F`. Nenhuma das três chegou a virar código.

Isso é diferente das iterações 0014–0019, onde o handoff descrevia o erro
**anterior**. Aqui ele descreveu o erro **seguinte**, com o mecanismo, e o
resultado foi que a bateria de mutação teve de construir os três à mão para
medir se doíam. Um log honesto tem de dizer que o campo funcionou — e também que
um `nenhum` obtido assim vale menos como evidência de que o código está certo do
que um `nenhum` obtido sem aviso.

**O erro #1 vale menos, mas é o mesmo movimento das notas 26/30/34/36 numa
dimensão nova:** copiar o vizinho. `Pair::set` no arquivo do `PUSH` recebe
`&mut Cpu` porque lá ela só é chamada sobre a CPU viva; aqui o teste do snapshot
compara `cpu.registers` com um `Registers` construído à parte, e o vizinho não
serve. Custou um erro de compilação, que é o modo mais barato de descobrir.

## Bateria de mutação

**11 mutantes, 11 pegos. 2 controles negativos, 2 verdes.**

| # | Mutante | Algozes |
|---|---|---|
| M1 | `pop_byte` pré-incrementa (espelho literal do `push_byte`) | 7 |
| M2 | latcha o byte baixo e escreve o par no fim do M3 | **1** |
| M3 | lê a metade alta primeiro | 7 |
| M4 | `saturating_add` no `pop_byte` | **1** |
| M5 | mascara o nibble baixo de `F` no `POP AF` | 6 |
| M6 | índice 3 do `r16stk` vira `sp` | 6 |
| M7 | máscara frouxa (solta o bit 3): leva `RET`/`RETI`/`JP HL`/`LD SP,HL` | 7 varreduras dos 256 |
| M8 | padrão com o bit 3 ligado: decodifica `C9 D9 E9 F9` no lugar do bloco | 14 |
| M9 | o segundo `(SP++)` não anda | 5 |
| M10 | o `POP` limpa a pilha depois de ler | **1** (guarda de ausência) |
| M11 | todo `POP` zera `F` | 6 |

| # | Controle negativo | Resultado |
|---|---|---|
| C1 | troca a ordem dos braços `PUSH`/`POP` no `fetch` (padrões disjuntos) | verde |
| C2 | `pop_byte` chamado dentro de cada braço em vez de antes do `match` | verde |

**O achado da iteração está em M1 contra a 0019, e é sobre a pilha, não sobre
mim.** É o *mesmo erro de forma* nos dois lados — deslocar o `±SP` em um passo —
e a visibilidade é oposta:

- **`PUSH`** (0019): decrementar no `internal` do M2 escreve **nos mesmos dois
  endereços, na mesma ordem**, e deixa o mesmo `SP` final. Só o instante muda.
  **8 dos 10 testes passam** contra a versão errada.
- **`POP`** (aqui): pré-incrementar lê em `SP+1` e `SP+2` em vez de `SP` e
  `SP+1`. O par sai errado, o `SP` final sai errado, o topo da pilha é ignorado.
  **7 dos 10 testes reprovam.**

A assimetria não é da suíte: é da instrução. O `PUSH` decide o endereço *antes*
do acesso, e adiantar o decremento não muda qual endereço é. O `POP` decide o
endereço *no* acesso, e adiantar o incremento muda o endereço lido. Corolário
para o 1.10, onde `CALL`/`RET` fazem as duas coisas: **o lado que escreve tolera
o erro de instante em silêncio; o lado que lê grita.** Não se pode concluir de
um `RET` verde que o `CALL` está no instante certo.

O erro que ficou caro continua sendo o mesmo de sempre — **M2, o latch, com um
único algoz** (`the_stack_pointer_moves_between_the_two_reads`, o teste com
asserção depois de cada M-cycle). Quinta medição da proporção: 9/10 (0015),
10/11 (0016), 7/8 (0018), 8/10 (0019), **9/10** aqui.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| todas (121 ROMs) | 0 | 0 |
| testes do workspace | 215 | 225 |

Sem regressão no `scoreboard.sh`. MSRV conferida à mão pela oitava vez
(`cargo +1.85 test --all`: **225/225** em `rustc 1.85.1`) — nota 13, que segue
aberta, e cujo item é o 7.4.

## Revisão cruzada (segundo modelo)

Não executada: `REVIEW=0` no `scripts/loop.sh` por decisão do operador
(`STATUS.md`, notas 5 e 33).

## Decisões de arquitetura

- **`R16STK_MASK` é uma constante só, com dois padrões.** `PUSH_R16STK_MASK`
  virou `R16STK_MASK`, no idioma que o `LD_R16MEM_MASK` já usa no mesmo arquivo:
  os dois blocos têm a mesma forma (`11 rr 0xx1`) e o que os separa é o bit 2,
  que mora no padrão. C1 mediu que a ordem dos dois braços não importa —
  os padrões são disjuntos.
- **`pop_byte` é a simétrica de `push_byte`, e a simetria é de papel.**
  `push_byte` decrementa e escreve; `pop_byte` lê e incrementa. Trocar as duas
  linhas de lugar não produz a outra — é isso que faz a pilha fechar, e é o M1
  da bateria.
- **`write_r16_stk_low`/`_high` são funções próprias, e não reúso das do `R16`.**
  A quarta variante é `af` × `sp`, e o `SP` é o único par cuja metade não é um
  campo de 8 bits (invariante do 1.5a). Reusar exigiria converter entre as duas
  tabelas exatamente no lugar onde elas divergem; M6 é o mutante que mede o preço.

## Notas

**No RED, 2 dos 10 testes passaram, e um deles por vacuidade.** O teste do
layout de bits é aritmética pura e não toca na CPU; `a_pop_does_not_write_back_
to_the_stack` passou porque CPU travada não escreve em lugar nenhum. É a leitura
(a) da nota 8 outra vez — e a bateria confirmou o diagnóstico pelo outro lado:
esse mesmo teste é o **único** algoz de M10, o mutante escrito só para ele. Pela
nota 37, ele cai na categoria da nota 35 (guarda de regressão futura, precisa de
mutante próprio): não existe pressão para escrever um `POP` que limpa a pilha.

**A simetria com o `PUSH` acaba nas flags, e o arquivo vizinho tem a armadilha
pronta.** `no_push_touches_the_flags` está certo para os quatro `PUSH` — inclusive
o `PUSH AF`, que *lê* `F`. O espelho literal, `no_pop_touches_the_flags`, estaria
errado em um quarto dos casos, e o que o denuncia não é raciocínio: é a coluna de
flags do `$F1` na tabela, que é a única das quatro linhas do bloco que não é `-`.
O teste que ficou no lugar dele afirma a assimetria (`pop_af_is_the_only_one_of_
the_four_that_writes_the_flags`), o que é mais barato de conferir contra a spec
do que dois testes separados.

**`POP AF` era o encontro marcado com a decisão do 1.1, e não houve encontro.**
`f = value`, sem máscara, como o 1.1 decidiu e como o `PUSH AF` da 0019 já fazia
do lado que escreve. A previsão registrada continua de pé e **não** foi
retroajustada: se a máscara for necessária, quem cobra é a blargg
`cpu_instrs/01-special` no 1.13, e nesse dia a fonte entra em `docs/reference/`
junto com ela. M5 existe para que a máscara não entre antes disso por hábito.
