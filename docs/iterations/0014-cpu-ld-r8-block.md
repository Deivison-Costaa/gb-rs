# Iteração 0014 — o bloco `LD r8,r8`

- **Data:** 2026-07-26
- **Item do roadmap:** 1.4a (sub-item criado nesta iteração)
- **PR:** #16
- **Duração:** ~50min
- **Custo reportado:** — <!-- sessão interativa; ver nota 10 do STATUS.md -->
- **Turnos:** 1

## Objetivo

Decodificar `$40`–`$7F` **sem** `$76`: 63 opcodes de load entre registradores de
8 bits, em três formas de M-cycle.

## A quebra do 1.4, antes de qualquer código

O ROADMAP dizia "1.4 Opcodes: loads 8-bit". O grupo `x8/lsm` da tabela de gbops
tem **85** opcodes e cinco modos de endereçamento; um PR só passaria de longe do
limite do protocolo de iteração. O primeiro commit desta branch quebra o item em
quatro, **por regra de decodificação e não por quantidade**:

| Sub-item | Bloco | Opcodes |
|---|---|---|
| 1.4a | `01 ddd sss` | 63 |
| 1.4b | `00 ddd 110` (imediatos) | 8 |
| 1.4c | indireto por par (`BC`, `DE`, `HL±`) | 8 |
| 1.4d | absoluto e página `$FF00` | 6 |

63 + 8 + 8 + 6 = 85.

**Consequência que precisa ficar registrada:** a 0013 previu que o 1.4 seria
onde a tabela de micro-operações nasceria, por ser "a primeira iteração com
casos suficientes para generalizar em vez de chutar". Com a quebra, o 1.4a tem
três formas de M-cycle e todas da mesma família (`fetch`, e depois no máximo um
acesso ao barramento) — um terço dos dados. A decisão foi **movida para o 1.4d**
e está escrita no ROADMAP, não só aqui. Generalizar agora seria a nota 8 com
menos evidência do que a 0013 já tinha.

O que nasceu, e é bem menor, é o `R8`: o operando de três bits da § Block 1.
Não é antecipação — é a codificação que a spec dá, e os 63 opcodes a exercitam
inteira.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| gbops | linhas `40`–`7F` da tabela sem prefixo | `docs/reference/03-opcodes.md` |
| Pan Docs | § Block 1: 8-bit register-to-register loads | `docs/reference/02-cpu.md` |
| Pan Docs | § CPU Instruction Set, tabela de placeholders (`r8`) | `docs/reference/02-cpu.md` |

A § Block 1 dá a codificação (`5-3` destino, `2-0` fonte) e **a exceção**; a
tabela de gbops dá o timing e enumera os 63 opcodes um a um. Foram precisas as
duas: a seção do Pan Docs não tem timing, e a de gbops não tem a regra — ela
lista `LD B,C` sessenta e três vezes sem dizer que há uma fórmula.

**A nota 24 se confirmou, e com uma saída.** A tabela de placeholders do
`02-cpu.md` está corrompida na conversão: emenda `r8`, `r16`, `r16stk`,
`r16mem` e `cond` numa lista só, sem cabeçalho, com os índices 0–3 repetidos
quatro vezes. Os oito valores de `r8` são o primeiro bloco — e o que autoriza a
ler assim **não** é confiar na conversão, são dois testemunhos independentes: a
tabela de gbops enumera os 63 opcodes (`46` é `LD B,(HL)`, logo o índice 6 é
`[hl]`), e a própria exceção da § Block 1 só fecha com `[hl]` no índice 6, já
que `0b01_110_110` é `$76`. Onde a spec local está quebrada, o conserto é
triangular, não adivinhar.

## Erros de primeira tentativa

Procedimento da nota 20: os testes primeiro, lidos da spec; depois um esqueleto
descartável com a versão de memória; a suíte rodada contra ele. O RED vira uma
lista de nomes de teste em vez de uma impressão.

**Esqueleto A reprovou 6 dos 14 testes**, apontando dois erros:

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | `$40`–`$7F` é regular: decodifiquei os 64 direto dos bits, e `$76` virou um `LD (HL),(HL)` | § Block 1: *"**Exception**: trying to encode `ld [hl], [hl]` instead yields the `halt` instruction"* | `opcode_76_is_halt_and_not_a_load_from_hl_into_hl` e o controle negativo |
| 2 | timing | `LD r,(HL)` em **3** M-cycles: `fetch → read((HL)) → internal(escreve o registrador)` | 8 T-cycles, e a coluna é `fetch → read((HL)->B)` — a leitura no barramento e a escrita no registrador são o **mesmo** M-cycle | 4 testes, entre eles `loading_from_hl_takes_two_m_cycles_and_the_register_changes_on_the_second` |

O erro #2 merece o nome que tem: é a lição da 0013 aplicada onde ela não vale.
A 0013 descobriu que `JP u16` desvia no M4 e não no M3 — "o efeito acontece
depois do que a intuição diz" — e eu levei isso adiante como regra geral. Não é
regra geral, é o que a coluna de **cada** instrução diz. Ali havia quatro passos
e o quarto era `internal`; aqui há dois, e não sobra M-cycle onde pôr a escrita.
**Aprender a correção anterior como princípio é um jeito novo de errar**, e é
mais difícil de notar do que o erro original, porque vem com a sensação de estar
aplicando uma lição.

O erro #1 é o modo de falha que a § Block 1 existe para evitar, e ele não vem de
ignorância: eu sei que `$76` é `HALT`. Vem da **forma do código** — escrever
`0x40..=0x7F => { let dest = opcode >> 3; let source = opcode; … }` é o idioma
natural, e ele não tem onde perguntar pelo buraco. Conhecimento correto não se
converte sozinho em código correto quando o desenho não pede que ele apareça.

**O que saiu certo, e é dado também:** as 8 asserções que o esqueleto passou —
`LD r,r'` em 1 M-cycle, `LD (HL),r` em 2, a ordem `b c d e h l [hl] a`, os
flags intocados. A intuição de Z80 acertou o lado das escritas e errou o das
leituras; se o esqueleto tivesse errado tudo, não daria para dizer isso.

### Um vacuidade pega antes do esqueleto

Rodar a suíte contra a implementação *anterior* (nada decodificado) mostrou 3
testes verdes, e um deles não devia estar: `storing_l_to_hl_writes_the_low_byte_
of_the_address_it_wrote_to` usava `SCRATCH = $C000`. O byte baixo é `$00`, a
WRAM começa zerada, e o teste afirmava `$00 == $00` sem nada ter sido escrito.
`$C000` é o começo redondo da WRAM — o endereço que se escreve sem pensar.
Corrigido para `$C0A7`, com os **dois** bytes diferentes de zero.

Isso é a nota 8 na forma mais barata de cair, e o que a pegou foi rodar o RED
antes de implementar em vez de depois. Vale como regra pequena: **em teste de
memória, endereço e valor não podem compartilhar o zero com o estado inicial.**

## Bateria de mutação

12 mutantes, com o cuidado da nota 14 (escrita com mtime explícito, padrão
conferido para casar exatamente uma vez). **10 pegos, 2 controles verdes.**

| Mutante | Veredito |
|---|---|
| `HALT` = `$77` | pego por 3 |
| índice 6 de `r8` vira `A` | pego por 9 |
| destino e fonte trocados | pego por 9 |
| `LD r,r'` ganha um M-cycle | pego por 4 |
| `load_from_hl` lê de `(BC)` | pego por 4 |
| `store_to_hl` escreve em `(BC)` | pego por 4 |
| bloco termina em `$7E` | pego por 3 |
| bloco começa em `$41` | pego por 3 |
| máscara de `r8` vira 4 bits | pego por 10 |
| `LD r,r'` zera `F` de lambuja | pego por 3 |
| *(controle)* linha sem efeito em `load_from_hl` | verde |
| *(controle)* arm `HALT` removido | **verde** |

Duas leituras.

**O controle que interessa é o segundo, e ele não era controle.** Eu o escrevi
esperando que remover `HALT => State::Locked(…)` derrubasse a suíte. Não
derruba: `load_r8_r8` tem um braço `(MemoryAtHl, MemoryAtHl)` que devolve o
mesmo `UndecodedOpcode`, posto ali só para o `match` ser total sem `_ =>`. Os
dois caminhos são redundantes e **nenhum teste os distingue** — o comportamento
está garantido duas vezes e a intenção, uma vez nenhuma. Não é bug, e não vou
remover nenhum dos dois (o `const HALT` é onde a citação da spec mora, e o braço
total é a invariante de `match` do projeto). É redundância medida em vez de
suposta, e fica escrita.

**A segunda leitura fecha um buraco que o esqueleto não fechava.**
`no_load_in_the_block_touches_the_flags` passou contra a implementação anterior
*e* contra o esqueleto A — nunca tinha tido nada para reprovar, exatamente a
categoria "guarda vacuoso" da nota 8. O mutante que zera `F` de lambuja é o
primeiro objeto que ele rejeitou. **O esqueleto e a bateria não medem a mesma
coisa:** o esqueleto exercita o que eu erraria, a bateria exercita o que
qualquer um poderia quebrar depois. Um teste que sobrevive aos dois foi
exercitado; um que só passa nos dois pode não ter sido exercitado por nenhum.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| todas (121 ROMs) | 0/121 | 0/121 |

Sem mudança, e não podia haver: `gb-cli run` ainda sai `2` até o 1.12.
Testes do workspace: **142 → 156**.

MSRV: `cargo +1.85 test --all` deu **156/156** (`rustc 1.85.1`). Quinto ponto de
dado da nota 13, que continua aberta.

## Revisão cruzada (segundo modelo)

Não houve. `scripts/review.sh` continua sem `REVIEWER_CMD` configurado — nota 5
do `STATUS.md`.

## Decisões de arquitetura

- **`R8` e `ByteRegister` são dois tipos, não um enum de oito variantes.** O
  oitavo valor de `r8` é memória, não registrador. Separar deixa
  `State::LoadFromHl(ByteRegister)` carregar um destino que **é** registrador,
  sem `unreachable!` — pânico que a R6 não quer no `gb-core` — e faz o `match`
  das três formas de M-cycle sair sozinho dos dois campos.
- **O mapeamento `ByteRegister` → campo mora no decodificador, não em
  `Registers`.** O 1.1 decidiu campos públicos e nenhum acessor por
  registrador, "porque só engrossaria o decodificador do 1.4". Quem precisa de
  um nome de três bits para um campo é quem decodifica opcode; é lá que a
  tradução fica.

## Notas

O item mais barato desta iteração foi também o mais produtivo: rodar a suíte
nova contra o código **velho**, antes de escrever qualquer implementação. Em
0,00s ela devolveu três coisas — o RED com o motivo certo (opcodes não
decodificados), a confirmação de que dois guardas de ausência passam por
vacuidade, e um teste de verdade quebrado. Nenhuma das três aparece se o
primeiro `cargo test` do dia já for contra a implementação.
