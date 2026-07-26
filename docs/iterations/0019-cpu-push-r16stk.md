# Iteração 0019 — `PUSH r16stk`: o bloco `11 rr 0101`

- **Data:** 2026-07-26
- **Item do roadmap:** 1.5b
- **PR:** #22
- **Duração:** ~35min
- **Custo reportado:** — <!-- sessão interativa, sem --output-format json (nota 10) -->
- **Turnos:** 1

## Objetivo

`PUSH BC` / `PUSH DE` / `PUSH HL` / `PUSH AF` (`$C5 $D5 $E5 $F5`) em 4 M-cycles,
com o `SP` decrementado **dentro** de cada uma das duas escritas.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| gbops | linhas `C5` `D5` `E5` `F5` — coluna de M-cycles | `docs/reference/03-opcodes.md:245,261,277,293` |
| Pan Docs | § CPU Instruction Set — placeholder `r16stk` | `docs/reference/02-cpu.md:120-123` |
| Pan Docs | § Block 3 — layout de bits `11 rr 0101` | `docs/reference/02-cpu.md:604-610` |

A coluna, literal: `fetch → internal → write(B->(--SP)) → write(C->(--SP))`.
1 byte, 16 T-cycles, `-` nas quatro colunas de flag.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | O `internal` do M2 é onde o `SP` é decrementado pela primeira vez, e cada escrita pós-decrementa. É o desenho do Z80: `PUSH qq` são 11 T-cycles, `M1(5) + M2(3) + M3(3)`, e o T-cycle extra do M1 existe justamente para o decremento. | O `--SP` é **pré**-decremento escrito **dentro** do passo da escrita — `write(B->(--SP))`, a mesma notação do `write(A->(HL++))` do 1.4c. O `internal` do M2 não faz nada observável. | `the_stack_pointer_moves_between_the_two_writes` e `the_internal_m_cycle_of_a_push_changes_nothing_but_the_program_counter` — **e só esses dois**: 8 dos 10 testes passam contra a versão errada (mutante #1 da bateria) |
| 2 | endereçamento | Nenhum — a metade alta no endereço mais alto, e ela primeiro, saiu certa de memória. | Idem. | — |

O erro #1 é a **quarta** aparição da nota 26/30/34, e a primeira em que a fonte
da regra falsa é o **Z80** e não o próprio projeto. A R1 avisa exatamente isso
("você conhece Z80 melhor do que conhece o SM83"), e o aviso não impediu nada: o
que a intuição produz não é uma citação do Z80 que dê para checar, é a sensação
de que **um M-cycle vazio precisa de serviço**. `internal` que não faz nada tem
cara de bug, e é justamente o que a coluna manda escrever.

O `STATUS.md` estava do lado certo desta vez: a invariante do 1.4c — *"a coluna
escreve o efeito **dentro** do passo do acesso"* — cobre `(--SP)` sem alteração,
e a notação é idêntica. Foi a leitura da nota da tarefa, não a intuição, que
produziu o teste que pegou o erro.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 0/11 | 0/11 |
| TOTAL de ROMs | 0/121 | 0/121 |
| Testes do workspace | 205 | 215 |

`cargo +1.85 test --all`: **215/215** (`cargo 1.85.1`) — nono ponto de dado da
nota 13. Nada mais novo que a MSRV entrou (`to_be_bytes` em contexto não-const,
`const fn` sobre enum de dois bits).

## Bateria de mutação

10 mutantes de comportamento, 2 controles negativos. **PEGOS: 10/10**, controles
verdes. mtime explícito e casamento exato de uma vez (nota 14); zero mutante
inválido.

| # | Mutante | Testes que o pegam |
|---|---|---|
| 1 | `--SP` adiantado para o `internal` do M2 | **2** |
| 2 | metade baixa empilhada primeiro | 4 |
| 3 | sem o `internal` (3 M-cycles) | 4 |
| 4 | máscara solta o bit 3 (leva `$CD` e três inexistentes) | **1** |
| 5 | padrão `11 rr 0001` (o `POP` do 1.5c) | 7 |
| 6 | índice 3 do `r16stk` lê o `SP` | 4 |
| 7 | máscara no nibble baixo de `F` ao empilhar `AF` | 4 |
| 8 | sem decremento (as duas escritas no mesmo endereço) | 6 |
| 9 | `wrapping_sub` → `saturating_sub` | **1** |
| 10 | o `PUSH` zera `F` (para exercitar a guarda de ausência) | 2 |

Três mutantes com **um ou dois** algozes, e são os três que importam: o erro de
timing, a máscara frouxa e a volta do `SP`. Os outros sete são pegos por quatro
a sete testes cada — quer dizer que a suíte é redundante onde o erro é fácil e
apertada onde ele é difícil, que é a distribuição errada e a que sempre aparece.

## Decisões de arquitetura

- **`R16Stk` é um tipo separado de `R16`.** Quatro variantes, três em comum, e a
  quarta é a diferença inteira (`Af` × `Sp`). Fundir os dois com um parâmetro
  "qual tabela" poria a distinção num argumento em vez de num tipo, e é essa
  distinção que a § CPU Instruction Set define como duas tabelas.
- **`Cpu::push_byte` é a segunda função de M-cycle compartilhada do projeto**,
  depois do `Cpu::access` do 1.4d. Ela é o passo `write(->(--SP))` inteiro:
  pré-decremento e escrita, indivisíveis por construção — que é o ponto, porque
  é separá-los que produz o erro #1. O `POP` do 1.5c terá a simétrica
  (`read((SP++))`) e o `CALL`/`RST` do 1.10 reusam esta.
- **O `internal` do M2 é um braço de `match` que só troca de estado.** Primeiro
  M-cycle do projeto sem efeito nenhum e sem ser o último passo. Ele parece
  código morto e não é: é o M-cycle que a coluna cobra.

## Revisão cruzada (segundo modelo)

- **Modelo:** nenhum — `scripts/review.sh` continua desligado no loop
  (`REVIEW=0`, decisão do operador; ver nota 5 e nota 33).
- **Achados:** —
- **Procedentes:** —
- **Falso positivo mais interessante:** —

## Notas

**A conversão para Markdown engoliu o cabeçalho do `push r16stk`.** Em
`02-cpu.md` § Block 3, o layout de bits de `11 rr 0101` está **sob o cabeçalho
`pop r16stk`**, junto com o do `11 rr 0001`: o conversor fundiu instruções
consecutivas sob o primeiro título de cada grupo, como a nota 24 já havia
observado para as tabelas de placeholder. `grep -i push 02-cpu.md` devolve
**zero** linhas. Quem confiar no cabeçalho conclui que `11 rr 0101` é uma
variante de `POP`, ou que o `PUSH` não está no Block 3.

O que salvou foi a R1 lida como "leia a spec", plural: o `03-opcodes.md` tem as
quatro linhas com nome e coluna, e a confirmação da codificação veio da tabela
de bits **apesar** do título errado em cima dela. É a nota 15 outra vez, com um
modo de falha novo: lá a informação que faltava estava em outra seção; aqui ela
está no lugar certo, com o rótulo errado.

**A guarda de ausência foi útil pela primeira vez.** A nota 35 registrou, na
0018, que guarda de ausência não reprova mutante de comportamento — foi preciso
escrever um nono mutante só para exercitá-la. Aqui
`the_internal_m_cycle_of_a_push_changes_nothing_but_the_program_counter` é um
dos **dois** algozes do erro #1, o mais caro da iteração. A diferença é o
formato do erro: quando o erro é "algo que devia ser inerte fez algo", a guarda
de ausência é o teste em que ele mora, não um extra. A nota 35 continua valendo
para as guardas de flag (o mutante #10 existe só para o `no_push_touches_the_flags`).

**Cinco controles negativos dos 256 quebraram**, e o `cargo test` sem
`--no-fail-fast` mostrou só o primeiro — quatro dos cinco ficaram escondidos
atrás do binário que morreu antes. Custou uma reexecução e um susto ("só um
quebrou?"). É a nota 31 pela terceira vez, e o único acréscimo é operacional:
para medir o estrago de um opcode novo, `--no-fail-fast` desde a primeira
execução. A tentação de extrair a lista `decoded_elsewhere` para um lugar só
apareceu pela quarta vez e foi recusada pela quarta vez.
