# Iteração 0015 — imediatos de 8 bits: `LD r8,u8` e `LD (HL),u8`

- **Data:** 2026-07-26
- **Item do roadmap:** 1.4b
- **PR:** #17
- **Duração:** ~35min
- **Custo reportado:** <!-- sessão interativa, sem --output-format json; ver nota 10 do STATUS.md -->
- **Turnos:** 1

## Objetivo

Decodificar o bloco `00 ddd 110` — os sete `LD r8,u8` (`$06 $0E $16 $1E $26 $2E
$3E`) em 2 M-cycles e o `LD (HL),u8` (`$36`) em 3.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| gbops | Linhas `06 0E 16 1E 26 2E 36 3E` da tabela sem prefixo | `docs/reference/03-opcodes.md` |
| Pan Docs | § Block 0, layout de bits de `ld r8, imm8` | `docs/reference/02-cpu.md` |
| Pan Docs | Tabela de placeholders (`r8`), § CPU Instruction Set | `docs/reference/02-cpu.md` |

A leitura foi triangulada como a 0014 (nota 24): o `02-cpu.md` dá a codificação
(`00 ddd 110`, destino nos bits 5-3) e o `03-opcodes.md` dá timing e coluna de
M-cycles. O `grep` da nota 15 pelos oito opcodes no arquivo inteiro não achou
nota de rodapé nem exceção — e a **ausência de exceção** é o achado: a § Block 1
tem o `$76`, a § Block 0 não tem nada equivalente. Os oito são load.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | `$36` lê o imediato **e** escreve em `(HL)` no M2, com um `internal` no M3 fechando os 12 T-cycles | `fetch → read(u8) → write((HL))` — a escrita é o M3 | esqueleto, por `storing_an_immediate_at_hl_is_three_m_cycles_and_the_write_is_the_third` |

O erro é a **nota 26 correndo ao contrário**, e vale detalhar porque é a segunda
iteração seguida em que a lição anterior é a fonte do erro seguinte.

A 0014 aprendeu que `LD r,(HL)` **não** tem um terceiro M-cycle onde o efeito
aconteça — a leitura e a escrita no registrador são o mesmo M2 — e escreveu isso
como invariante em prosa forte. Aplicada ao `$36`, essa lição produz exatamente
o layout errado: "não invente M-cycle para o efeito acontecer" vira "faça tudo
no primeiro M-cycle que puder", e sobra um `internal` no fim. O total de
T-cycles fica certo (12), o estado final fica certo, e a escrita no barramento
sai um M-cycle adiantada — o tipo de divergência que só a Mooneye cobra.

Na 0014 o erro foi acrescentar um M-cycle; aqui foi adiantar um. **A correção da
0014 não é uma regra sobre quando o efeito acontece; é o resultado de ler a
coluna daquela instrução.** Duas iterações, duas direções, mesma causa: prosa de
invariante lida como princípio.

O que não errei, e vale registrar porque a previsão da 0014 apontava para lá: os
oito são `-` nas quatro colunas de flag, `LD r8,u8` são 2 M-cycles, e a
codificação é `00 ddd 110`. As quatro crenças pré-tabela bateram com a tabela.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| todas | 0/121 | 0/121 |

Sem emulador ainda (`gb-cli run` sai `2`). Testes do workspace: **156 → 166**.

## Revisão cruzada (segundo modelo)

Não executada: `scripts/review.sh` continua sem `REVIEWER_CMD` configurado
(nota 5 do `STATUS.md`).

## Decisões de arquitetura

**Máscara, não faixa.** Os oito opcodes andam de 8 em 8, então o reconhecimento
é `opcode & 0b1100_0111 == 0b0000_0110` e não um `..=`. Isso põe um `match` com
guarda no meio do `fetch`, e a ordem dos braços passa a importar de um jeito que
não importava no 1.4a — o controle negativo dos 256 é o que garante que a
máscara não invadiu os vizinhos.

**`INC r8` e `DEC r8` são os vizinhos perigosos.** `00 ddd 100` e `00 ddd 101`
compartilham os bits 7-6 e o campo de destino com este bloco, e diferem só nos
bits 2-0. Uma máscara frouxa num bit engole `RLCA`, `INC`, `DEC` — e nenhum
teste de comportamento acima reclamaria, porque nenhum deles menciona esses
opcodes. Quem pega é a varredura dos 256.

**O latch guarda o imediato do `$36`.** Mesmo campo que o `JP u16` usa para os
dois bytes do endereço. Não virou tabela de micro-operações: o `mcycle.rs` agora
tem duas famílias de forma (um acesso, dois acessos), e a decisão continua
marcada para o 1.4d, quando as formas do 1.4c e do 1.4d existirem.

## Notas

**A bateria de mutação exercitou o que o esqueleto não exercitou, pela terceira
vez.** `no_immediate_load_touches_the_flags` passou contra a implementação
anterior (os oito travavam antes de tocar em `F`) **e** contra o esqueleto (que
não mexe em flag, porque eu não escreveria um `LD` que mexe em flag). Quem o
exercitou foi o mutante que zera `F` de lambuja no `$36`. É a nota 28 confirmada:
guarda de *ausência* de comportamento cai fora do alcance do esqueleto por
construção.

**O esqueleto foi pego por um teste só, e o número é o dado.** 9 dos 10 testes
passaram contra o `$36` errado. Não é suíte fraca: o estado inicial e o estado
final são idênticos nos dois layouts, e nove dos testes só olham o começo e o
fim. **Só a asserção que lê a memória entre o M2 e o M3 separa os dois** — e é
por isso que a R2 pede teste que observe o meio da instrução, e não só o
resultado. Um teste "normal" de `$36` fecharia verde contra a versão errada.

**O buraco da nota 25 fechou.** A volta do `PC` em `$FFFF` estava medida como
não coberta desde a 0013: a bateria daquela iteração trocou `wrapping_add` por
`saturating_add` e a suíte inteira ficou verde. Este é o primeiro item desde
então com operando lido do fluxo de instruções, e
`an_immediate_load_reads_its_operand_across_the_program_counter_wrap` cobre
agora — opcode em `$FFFF` (que é o `IE`, o único byte gravável ali), operando
em `$0000`, que é ROM.

**Dois testes de iterações anteriores tiveram de mudar, e os dois são guardas
funcionando.** O controle negativo do 1.4a reprovou `$06` como "deveria estar
`UndecodedOpcode`", e `an_opcode_this_emulator_has_not_reached_is_not_an_illegal_one`
usava `$06` como exemplo de opcode não implementado. Os dois são a mesma
categoria: **teste que afirma a fronteira do que existe envelhece quando a
fronteira anda.** Não é atrito acidental — é o preço de ter fronteira testada, e
é barato. O segundo passou a usar `$04` (`INC B`), com a mudança de exemplo
documentada no próprio teste, e virou de quebra um guarda da máscara: `INC B` é
o vizinho de `$06` na tabela.

**`decoded_elsewhere` mora no arquivo do 1.4a de propósito.** A tentação era
extrair uma lista compartilhada de "opcodes já decodificados". Isso faria a
atualização acontecer sozinha em cada sub-item novo, e o controle negativo
perderia a propriedade que o justifica: obrigar quem acrescenta opcode a vir
declarar o que acrescentou.
