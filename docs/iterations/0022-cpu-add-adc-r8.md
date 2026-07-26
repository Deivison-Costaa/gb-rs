# Iteração 0022 — `ADD a,r8` e `ADC a,r8`: as primeiras flags calculadas

- **Data:** 2026-07-26
- **Item do roadmap:** 1.6a (e a quebra do 1.6 em cinco)
- **PR:** #26
- **Duração:** ~1 sessão
- **Custo reportado:** n/d — sessão interativa de Claude Code, fora do `loop.sh`
- **Turnos:** 1

## Objetivo

Os dezesseis opcodes `$80`–`$8F` (`10 000 rrr` e `10 001 rrr`), e com eles a
primeira ALU do projeto: `Z`/`N`/`H`/`C` calculadas, depois de 254 linhas em que
as quatro colunas eram `-`.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| gbops | linhas `80`–`8F` (flags, T-cycles, coluna de M-cycles) | `docs/reference/03-opcodes.md` |
| Pan Docs | § The Flags Register (posição dos bits) | `docs/reference/02-cpu.md` |
| Pan Docs | § The Carry Flag (*"higher than $FF"*; lista `ADC` entre quem usa `C`) | `docs/reference/02-cpu.md` |
| Pan Docs | § The BCD Flags (*"H indicates carry for the lower 4 bits of the result"*) | `docs/reference/02-cpu.md` |
| Pan Docs | § Block 2: 8-bit arithmetic (layout `10 ooo rrr`) | `docs/reference/02-cpu.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | A nota 34 do projeto ("seta ausente na coluna = latch") vale onde ela aparece. O `$86` é `fetch → read((HL))`, **sem seta** — pela regra, o byte fica num latch e aterrissa num passo seguinte. | Não há passo seguinte: a linha tem **8** T-cycles, dois M-cycles, e os dois estão escritos. A seta falta porque o destino não é registrador nenhum — é a ALU. | Antecipado e escrito como teste antes de implementar (`the_operand_of_86_..._the_sum_lands_there_too`). Confirmado pelo mutante M9. |
| 2 | timing | *(não escrito, mas não testado)* — que "o acesso é do M2" estivesse coberto pelos testes que já havia. | A invariante do 1.3 diz que uma chamada de `Cpu::step` faz **no máximo um** acesso, e o M1 já gastou o dele no opcode. | **Não foi pego por teste nenhum.** O mutante M16 — ler `(HL)` dentro do fetch e gastar o M2 aplicando — passou verde nos **251** testes do workspace. Ver § Notas. |
| 3 | flags | `nenhum` no cálculo. `H` do nibble baixo, `C` do byte, `N` = 0, `Z` do resultado truncado, e o carry de entrada do `ADC` contando para os três — foi o que escrevi de memória e é o que a spec diz. | idem | — |

Sobre o #3, e é o ponto que a **nota 41** manda registrar: o campo
`Próxima tarefa` do `STATUS.md` da 0021 pré-anunciou seis armadilhas de flag,
entre elas *"`ADC`/`SBC` consomem o `C` de entrada e o half-carry tem de
contá-lo — `A=$0F` com operando `$00` e `C=1` é o caso que separa a versão certa
da que ignora o carry-in"*. Esse caso virou teste antes de a ALU existir. Um
`nenhum` obtido com o aviso na tela mede o aviso, não o agente — por isso a
bateria de mutação abaixo não é opcional, e é dela que sai o achado da iteração.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| todas as 11 | 0/121 | 0/121 |
| testes do workspace | 240 | 252 |

## Revisão cruzada (segundo modelo)

Não houve. `scripts/review.sh` continua sem `REVIEWER_CMD` configurado
(`STATUS.md`, nota 5), e o loop roda com `REVIEW=0`. O substituto desta iteração
foi a bateria de mutação, com a ressalva conhecida da nota 8(b): mutante escrito
por quem escreveu o teste herda o ponto cego do teste — o que aconteceu aqui de
forma legível, ver § Notas.

## Decisões de arquitetura

**A quebra do 1.6 não seguiu a regra do 1.4 e do 1.5.** Os dois anteriores foram
quebrados por **regra de decodificação**, com uma forma de M-cycle por sub-item.
Aqui isso não separa nada: 88 opcodes, quatro blocos, e só **três** formas de
M-cycle no grupo inteiro (`fetch`; `fetch → read`; `fetch → read((HL)) →
write((HL))`). A dimensão que de fato muda é a **coluna de flags** — `N` literal
`0` × literal `1`, `H` carry × empréstimo × literal `1` × literal `0`, `C`
calculada × literal `0` × **não afetada**. O corte ficou por semântica de flag
(1.6a/b/c) e os dois últimos por bloco (1.6d/e). A divergência está escrita no
próprio ROADMAP, não só aqui.

**`alu.rs` é módulo próprio, e `apply` é função livre sobre `Registers`.** A ALU
não precisa do `Bus` nem do estado da máquina — só do banco e de um operando.
Deixá-la fora do `mcycle.rs` mantém a fronteira "quem decide o instante" separada
de "quem faz a conta", que é a fronteira em que os erros deste projeto moram: o
#1 e o #2 acima são de instante, o #3 seria de conta, e eles não se olham.

**A quarta função de M-cycle compartilhada não nasceu.** Depois do `Cpu::access`
(1.4d), do `push_byte` (1.5b) e do `pop_byte` (1.5c), o candidato natural aqui
seria "ler `(HL)` e entregar à ALU". Com um sítio só, não foi extraído — a
doutrina do 1.4d é que a abstração nasce onde a repetição **existe**. Os sítios
chegam no 1.6b e no 1.6c, e aí `AluFromHl(AluOp)` já os cobre sem linha nova.

## Notas

### O achado: o mutante silencioso, e por que a suíte não o via

A bateria mediu **16 mutantes e 3 controles**. Quinze mutantes morreram de
primeira. O décimo sexto é o que vale a iteração:

> **M16** — ler `(HL)` **dentro do fetch** e gastar o M2 aplicando a soma.

Ele dá o mesmo `A`, as mesmas quatro flags, o mesmo `PC`, os mesmos **dois**
M-cycles e os mesmos **8** T-cycles. O que ele quebra é a invariante do 1.3 —
*uma chamada de `Cpu::step` faz no máximo um acesso ao barramento* — porque o M1
já gastou o dele lendo o opcode. **Passou verde nos 251 testes do workspace**,
inclusive no teste que eu havia escrito de propósito para medir o instante
(`the_operand_of_86_is_read_in_the_second_m_cycle_and_the_sum_lands_there_too`,
que observa `A` e `F` depois de cada M-cycle).

A razão de ele escapar é precisa: o operando não tem destino observável entre os
passos. Nos casos anteriores do projeto havia sempre uma testemunha — a memória
entre duas escritas (0015, 0021), o `HL` entre dois acessos (0016), o `SP` entre
dois passos (0019, 0020), o `PC` entre dois bytes de operando (0021/M11). Aqui o
byte lido vai direto para a ALU e some. **A única testemunha possível é a
memória de fora: trocar o conteúdo de `(HL)` entre o `step` do M1 e o do M2.**
Escrito assim, `the_bus_access_of_86_happens_in_the_m2_and_not_during_the_fetch`
mata o M16 e é seu **único** algoz.

Isto é a nota 43 numa terceira forma. Ela nasceu dizendo "asserção entre
M-cycles sobre a memória"; a 0021 acrescentou "e sobre o `PC`". A forma daqui é
a que faltava: **quando o valor lido não fica em lugar nenhum, quem observa o
instante é o estímulo, não a asserção** — muda-se a fonte no meio da instrução e
vê-se qual das duas versões o resultado denuncia.

### O erro de instante mudou de regime, e a causa é a coluna de T-cycles

Da 0015 à 0021 a proporção medida foi sempre a mesma: 9/10, 10/11, 7/8, 8/10,
14/15 — o erro de instante deixava quase toda a suíte verde. Aqui a mesma classe
de erro se partiu em dois regimes com detecções opostas:

- **M9** (o `$86` em **3** M-cycles, o erro que a nota 34 lida como regra geral
  produziria): **4 algozes**, barulhento. Ele muda o total de 8 para 12
  T-cycles, e o total está na coluna.
- **M16** (o `$86` em 2 M-cycles com o acesso adiantado): **0 algozes** entre os
  251 testes, e 1 depois do teste novo.

A diferença não é a suíte: é que o erro barulhento **gasta um passo a mais** e o
silencioso não. A classe silenciosa exige um passo sobrando onde o efeito possa
se esconder. `ADD A,(HL)` tem exatamente dois passos e os dois são acesso — o
único lugar onde o erro cabe é *dentro* do M1, empilhado com o fetch. **Corolário
para o 1.6e:** `INC (HL)`/`DEC (HL)` são três passos, com um read e um write no
mesmo endereço — lá cabem as duas formas, e a asserção que vale é a que lê a
memória entre o M2 e o M3.

### O ponto cego herdado (nota 8(b)), visto de perto

Os quinze primeiros mutantes foram escritos por quem escreveu os testes, e os
quinze morreram. O décimo sexto só existiu porque, ao anotar *"M9 foi pego por 4
testes, e isso destoa das cinco medições anteriores"*, ficou evidente que o
mutante construído não era o análogo dos anteriores — ele era barulhento por
mudar a contagem. Procurar o análogo **silencioso** foi o que produziu o M16.

O procedimento que sai daí é barato e vale para as próximas: **quando um mutante
de instante morre com muitos algozes, desconfie de que ele não é o mutante da
classe.** A classe é a que preserva o total de T-cycles. Se o mutante escrito não
preserva, ele ainda não foi escrito.

### Testes que passaram por vacuidade no RED (nota 8)

Na primeira execução, 8 dos 11 testes falharam e **3 passaram**. Um deles é
legítimo (`the_block_2_layout_...`, aritmética de bits pura, sem CPU). Os outros
dois — `an_add_or_adc_touches_only_a_...` e `no_opcode_of_this_item_writes_to_
memory` — passaram porque a CPU travava em `UndecodedOpcode` e CPU travada não
escreve em lugar nenhum. É a nota 8 em forma pura, e os dois só passaram a valer
alguma coisa com os mutantes M14 e M15, escritos exatamente para eles (nota 35).
Cada um tem **um** algoz, e é o teste certo.

### Atrito

- **R7 reprovou o `alu.rs` de primeira**: 5/30 = 16%, acima do teto de 12%. O
  arquivo é curto, e num arquivo curto três linhas de comentário já estouram a
  razão. A prosa foi para cá, que é onde o `CLAUDE.md` § R7 diz que ela mora; no
  código ficaram duas linhas com o mecanismo e um ponteiro.
- **Oito controles negativos dos 256 quebraram** quando `$80`–`$8F` deixou de
  ser "não implementado" — nota 31 pela quarta vez, oito `Edit` de uma linha. A
  tentação de extrair a lista para um lugar só apareceu de novo e foi recusada de
  novo: guarda que se atualiza sozinho não guarda. `cpu_mcycle_loop.rs`, que usa
  `$04` (`INC B`) como exemplo de opcode não implementado, continuou verde —
  `INC r8` é o 1.6e, e o exemplo sobrevive mais quatro sub-itens.
- **Um controle da bateria não compilou** (renomear a variante `AluOp::Add`: o
  `replace` pegou `AluOp::AddWithCarry` junto). Reescrito como renomeação do
  parâmetro. Controle que não compila não é controle — é ruído com cara de
  resultado.
