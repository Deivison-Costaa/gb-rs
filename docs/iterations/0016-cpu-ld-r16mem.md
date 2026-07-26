# Iteração 0016 — o indireto por par de registradores

- **Data:** 2026-07-26
- **Item do roadmap:** 1.4c
- **PR:** #18
- **Duração:** ~30min
- **Custo reportado:** — <!-- sessão interativa, sem --output-format json; ver nota 10 -->
- **Turnos:** 1

## Objetivo

Os oito opcodes que endereçam memória por um par de registradores —
`LD (BC),A`, `LD A,(BC)`, `LD (DE),A`, `LD A,(DE)` (`$02 $0A $12 $1A`) e as
quatro formas com `HL+`/`HL-` (`$22 $2A $32 $3A`) — e com eles o primeiro
operando do projeto que **modifica** o registrador que usou como endereço.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| gbops | linhas `02 0A 12 1A 22 2A 32 3A` | `docs/reference/03-opcodes.md` |
| Pan Docs | § Block 0, layouts de `ld [r16mem], a` e `ld a, [r16mem]` | `docs/reference/02-cpu.md` |
| Pan Docs | § CPU Instruction Set, tabela de placeholders (`r16mem`) | `docs/reference/02-cpu.md` |
| Pan Docs | § OAM Corruption Bug | `docs/reference/06-ppu.md` |

As oito linhas de gbops têm a **mesma** forma, e é a primeira vez que isso
acontece num sub-item do 1.4: `fetch → write(A->(rr))` ou
`fetch → read((rr)->A)`, 1 byte, 8 T-cycles. O conceito novo não é uma forma de
M-cycle, é o `++`/`--` escrito **dentro** do passo do acesso.

A quarta linha da tabela acima é a nota 15 do `STATUS.md` pagando outra vez, e
foi a leitura mais produtiva da iteração. A § Block 0 dá a codificação e não diz
nada sobre ordem; quem diz é a § OAM Corruption Bug, num arquivo que o
`docs/reference/README.md` mapeia para o **M3**, três marcos adiante:

> …if the following instructions are used while their 16-bit content
> **(before the operation)** is in the range $FE00–$FEFF and the PPU is in mode 2

O `(before the operation)` é a confirmação independente de que o endereço é o
valor anterior à modificação — o mesmo fato que o `++` postfixo de gbops afirma,
dito por uma seção escrita para outra finalidade.

E de lambuja ela desmente uma leitura natural que eu tinha e que **não** está no
código: que o `HL±` seja aritmética de registrador sem contrapartida no
barramento. Não é. A IDU põe o valor nas linhas de endereço mesmo sem leitura nem
escrita assertada, e é por isso que essas quatro instruções corrompem a OAM
*duas* vezes — uma pelo acesso, outra pelo incremento. Nada disso é implementado
(é o 7.2), mas está registrado no doc de `address_from_r16_mem` para que quem
chegar lá não descubra do zero.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | O endereço do par pode ser resolvido no **fetch** — o decodificador já tem `&mut self` e o `latch` existe para carregar operando entre M-cycles —, e com ele o `HL±`. O M2 só faria o acesso. | A coluna escreve o efeito **dentro** do passo do acesso (`write(A->(HL++))`). `HL` muda no M2. Depois do fetch o par ainda vale o que valia. | Esqueleto A, **1 teste de 11** |

**Um erro, e o número ao lado dele é o achado.** Foi a terceira iteração seguida
a errar *em qual* M-cycle o efeito cai (0014 atrasou, 0015 adiantou, esta
adiantou), e a nota 30 previu literalmente este caso — "toda instrução com mais
de um acesso ao barramento precisa de um teste que observe o estado *entre* os
acessos". A previsão estava certa e **a suíte escrita a partir dela quase não a
honrou**: 10 dos 11 testes passam contra o esqueleto.

O que sobrevive ao erro: o valor final na memória, o valor final em `HL`, o
total de 8 T-cycles, `A`, as flags, o `PC`, a varredura dos 256 opcodes. O que o
reprova é uma asserção só —
`the_hl_side_effect_lands_on_the_second_m_cycle_and_not_on_the_fetch`, na linha
que lê `HL` *entre* os dois `step`:

```
$22: no M1 o `HL` ainda não se moveu — o `HL±` é do M2, junto com o acesso
  left: 49866
 right: 49865
```

A 0015 mediu 9 de 10 para o erro equivalente. A 0016 mediu 10 de 11. **A razão
de a proporção ser essa não é a suíte ser fraca — é que o erro não tem efeito
observável fora do instante do acesso**, e é exatamente a classe que a Mooneye
mede e que uma suíte de antes/depois não vê. Duas medições independentes com o
mesmo resultado transformam a regra prática da nota 30 de conselho em dado.

Não houve erro nas outras quatro perguntas do sub-item, e vale dizer quais eram
para que a ausência conte: a leitura de `r16mem` como `bc de hl+ hl-` (e não
`bc de hl sp`, a armadilha (a) anotada pela 0015), os índices 2 e 3 na ordem
`hl+`/`hl-`, o pós-incremento, e a direção do bit 3. As quatro estavam certas de
memória e as quatro têm mutante próprio na bateria — quatro de onze.

**A armadilha (c) anotada pela 0015 não existe.** O aviso era que
`LD (HL+),A` com `HL` apontando para o próprio destino seria o caso em que a
ordem de operações vira valor observável. Fui procurar e não há: `HL` não é
memória mapeada, então "escreve em `(HL)` e depois incrementa" e "incrementa e
depois escreve em `(HL)-1`" são indistinguíveis em qualquer endereço, inclusive
`$FFFF`. É a nota 22 outra vez — a previsão de qual armadilha vai doer erra, e a
que doeu foi a (b), a do M-cycle.

## Bateria de mutação

11 mutantes, **11 pegos**; 2 controles, **2 verdes**. Suítes: as quatro que
tocam o decodificador (`cpu_ld_r16mem`, `cpu_ld_r8_block`, `cpu_ld_r8_u8`,
`cpu_mcycle_loop`). mtime explícito e alvo conferido como ocorrência única
(notas 14 e 18).

Os mutantes: pré-incremento, pré-decremento, índices 2 e 3 trocados, `r16mem`
lido como `r16` (índice 2 sem efeito colateral), direção do bit 3 invertida,
`saturating` no incremento, `saturating` no decremento, campo do par em `>> 3`,
máscara frouxa no bit 3 (as duas famílias colapsam numa), máscara frouxa no
bit 0 (engole `INC r16`), e efeito colateral zerando `F` de lambuja.
Controles: `wrapping_sub(1)` reescrito como `wrapping_add(0xFFFF)`, e a extração
do campo do par reescrita com o mesmo valor.

**Três testes só foram exercitados aqui, e cada um por um mutante só:**

- `no_load_or_store_through_a_pair_touches_the_flags` — passou contra o código
  velho *e* contra o esqueleto. Quem o exercitou foi o mutante que zera `F`.
  É a nota 28 reproduzida com precisão: guarda de *ausência* de comportamento
  não cai no grupo de erros que eu cometeria, cai no grupo de erros que qualquer
  um introduz depois.
- `the_block_this_item_decodes_is_exactly_the_eight_opcodes_of_00_mm_x010` — o
  único a pegar a máscara frouxa no bit 0, que engole `INC r16` (`00 mm 0011`,
  do 1.7). Nenhum teste de comportamento do sub-item menciona `$03`. Nota 25.
- `the_side_effect_wraps_and_does_not_saturate` — o único a pegar os dois
  mutantes de `saturating`.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| todas as 11 | 0/121 | 0/121 |

Sem mudança, e é o esperado: não existe `gb-cli run` até o 1.12. O
`scoreboard.sh` anexou as 121 linhas (2058 no CSV, eram 1937).

Testes do workspace: **177** (eram 166). +11 do arquivo novo.

MSRV (nota 13), **sétimo ponto de dado à mão**: `cargo +1.85 test --all` deu
**177/177** em `cargo 1.85.1`. Nada mais novo que a MSRV entrou — `const fn`
sobre enum de dois bits e `wrapping_add`/`wrapping_sub` em `u16`.

## Revisão cruzada (segundo modelo)

Não houve. `scripts/review.sh` continua sem `REVIEWER_CMD` configurado
(`STATUS.md`, nota 5); não há segundo modelo disponível nesta sessão.

## Decisões de arquitetura

- **`R16Mem` nasceu; a tabela de micro-operações não.** O operando de dois bits
  da § Block 0 é codificação que a spec dá, e os oito opcodes a exercitam
  inteira — mesmo critério que criou `R8` no 1.4a. Já a tabela genérica de
  micro-operações continua não existindo, e a razão mudou de forma: o 1.4c
  **não trouxe forma nova**. As oito linhas são `fetch` mais um acesso, como o
  1.4a. Três sub-itens, e o que se repete é a mesma família. O que falta para
  decidir o desenho são as formas do 1.4d, que trazem o primeiro operando de
  dois bytes desde o `JP u16`. A decisão segue marcada no ROADMAP, no 1.4d.
- **O par viaja no `State`, não o endereço resolvido.**
  `State::LoadFromR16Mem(R16Mem)` em vez de guardar o endereço no `latch` no M1.
  Isso não é preferência de estilo: é o erro #1 tornado inexprimível. Com o
  endereço no `latch`, o `HL±` **tem** de acontecer no fetch; com o par, ele só
  pode acontecer no M2, que é onde a coluna o põe.
- **`address_from_r16_mem` é `&mut self` de propósito.** Metade dela é o efeito
  colateral. Um `const fn` que só devolvesse o endereço obrigaria o chamador a
  mexer em `HL` por fora, e é justamente aí que a ordem se perde.

## Notas

**O item mais barato desta iteração foi tornar uma dívida agendável.** A nota 13
do `STATUS.md` sobre a MSRV está aberta desde a 0006 e acumulou **sete** pontos
de dado, todos "passou, à mão, porque alguém lembrou". A 0015 diagnosticou o
porquê e o diagnóstico é sobre processo, não sobre custo: *"o item não existe no
ROADMAP, e o protocolo de iteração só executa o que está no ROADMAP. Dívida que
ninguém agenda não é priorizada baixo; é invisível."*

Isso agora é o **7.4**. Não foi implementado aqui — seria uma segunda
micro-funcionalidade, e a R4 proíbe — e foi posto em M7 e não em M0 de propósito,
para não preemptar o 1.4d na regra de "próxima caixa não marcada, em ordem". A
prioridade continua honestamente baixa (o pior caso é um PR vermelho, barato de
consertar); o que mudou é que ela deixou de depender de alguém lembrar.

**Dois testes de iterações anteriores quebraram, e é a nota 31 funcionando:**
os controles negativos dos 256 do 1.4a e do 1.4b, os dois porque `$02` deixou de
ser "não implementado". Dois `Edit`, um em cada `decoded_elsewhere`/
`previously_decoded`. A tentação de extrair a lista para um lugar só apareceu
outra vez e foi recusada outra vez, pelo motivo já registrado: guarda que se
atualiza sozinho não guarda.

**Uma coincidência de máscara que vale anotar antes que alguém a "simplifique":**
os oito opcodes deste sub-item satisfazem `opcode & 0b1100_0111 == 0b0000_0010`,
que é a **mesma máscara** do 1.4b com outro padrão (`0b0000_0110`). É por isso
que os controles negativos usam essa forma e o decodificador usa outra
(`0b1100_1111`, com dois padrões): o controle negativo só precisa saber se o
opcode pertence ao conjunto, e o decodificador precisa saber a **direção**, que
mora no bit 3 e que a máscara de 7 bits apaga. Unificar as duas apagaria a
distinção entre `LD (BC),A` e `LD A,(BC)` — é o mutante "máscara frouxa no
bit 3", que morre em 3 testes.
