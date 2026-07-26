# Iteração 0013 — o laço de M-cycles

- **Data:** 2026-07-26
- **Item do roadmap:** 1.3
- **PR:** #15
- **Duração:** ~35min
- **Custo reportado:** _(sessão interativa, sem `--output-format json` — ver `STATUS.md`, nota 10)_
- **Turnos:** 1

## Objetivo

`Cpu::step()` avança **um** M-cycle e retorna, com fetch/decode/execute como
máquina de estados — a R2 do `CLAUDE.md`. Junta o `Registers::after_boot_rom`
(1.2b-i) ao `Bus::new` (1.2a + 1.2b-ii), que até aqui não se conheciam.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| gbops | Tabela de opcodes, coluna *M-cycles (passo a passo)* — linhas `00` (`NOP`) e `C3` (`JP u16`) | `docs/reference/03-opcodes.md` |
| gbops | Linhas `unused` (`D3 DB DD E3 E4 EB EC ED F4 FC FD`), 0 T-cycles | `docs/reference/03-opcodes.md` |
| Pan Docs | § CPU Comparison with Z80 → *Moved, Removed, and Added Opcodes* | `docs/reference/02-cpu.md:855-885` |
| Pan Docs | § CPU Instruction Set (tentada, inútil — ver Notas) | `docs/reference/02-cpu.md:84-814` |

## Erros de primeira tentativa

Procedimento da nota 20: os testes primeiro, depois **dois** esqueletos
descartáveis com a versão de memória — com a assinatura final, para o vermelho
medir asserção e não `E0432` — e a suíte rodada contra cada um. Dois esqueletos
porque o primeiro erro (arquitetural) esconde os outros: uma implementação
instruction-stepped não tem M3 nem M4 onde o erro de timing possa aparecer.

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | `step()` executa a instrução inteira e o número de ciclos é o resultado. É o desenho que sai sozinho da cabeça — e é literalmente o que a R2 proíbe em texto | A coluna *M-cycles (passo a passo)* lista o que o barramento faz **em cada** M-cycle; cada passo é uma parada | Esqueleto A: 3 dos 11 testes reprovaram |
| 2 | timing | Depois de `read(u16:upper)` o endereço de destino está inteiro na CPU, logo o `PC` já pode virar o alvo; o quarto M-cycle seria enchimento | `C3` é `fetch → read(u16:lower) → read(u16:upper) → internal(branch decision?)`: o desvio é o **quarto** | Esqueleto B: `each_step_advances_one_m_cycle_not_one_instruction` |
| 3 | comportamento | Opcode que a CPU não reconhece não faz nada — segue para o próximo byte | `02-cpu.md:885`: *"The unused (-) opcodes will lock up the Game Boy CPU when used."* E os onze têm **0 T-cycles** em gbops: instrução sem timing é instrução que não termina | Esqueleto B: `an_opcode_this_emulator_has_not_reached_is_not_an_illegal_one` |

**O que não foi medido, e por quê.** A lista dos onze opcodes inexistentes saiu
certa de memória — mas isso *não* conta como acerto: ela está escrita em
`docs/reference/README.md` § As armadilhas, em um lugar só, que é leitura do
Passo 0 deste mesmo protocolo. Memória contaminada por leitura recente não mede
memória. Quem mediu a lista foi a bateria de mutação, abaixo.

### Bateria de mutação — 6 mutantes, 4 pegos, 2 controles verdes

Os dois esqueletos deixaram `the_unused_opcodes_are_exactly_the_eleven_the_spec_names`
passar sem nunca ter tido nada para reprovar. A nota 8 diz para tratar isso como
suspeita, não como notícia boa, e a suspeita procedia em parte:

| Mutante | Esperado | Resultado |
|---|---|---|
| `$D9` entra na lista de ilegais (é `RETI` no GB, `EXX` no Z80) | pego | **só** `..._are_exactly_the_eleven...` pegou |
| `$FD` sai da lista | pego | pegaram os dois testes de opcode ilegal |
| CPU travada conta como "entre instruções" | pego | `the_opcodes_the_spec_calls_unused_lock_the_cpu` |
| Opcode não decodificado vira `NOP` | pego | `an_opcode_..._is_not_an_illegal_one` |
| `latch` começa em `$FFFF` em vez de `0` | verde | verde |
| `wrapping_add` → `saturating_add` no `PC` | verde | **verde — e não devia ser controle** |

O primeiro mutante é o achado: acrescentar um opcode à lista é pego por **um só**
teste, o do controle negativo. O outro só confere que os onze travam, e onze
opcodes travando continua verdade quando são doze. Sem o controle negativo, o
erro de Z80 mais provável desta seção — `D9` — entraria sem nada ficar vermelho.

O último mutante estava classificado como controle e passou por engano, o que é
o resultado mais útil da bateria: `wrapping_add` no `PC` é comportamento real e
**nenhum teste o cobre**. Não é bug (dar a volta é o que um registrador de 16
bits faz), é buraco de cobertura medido. Fica anotado para o 1.4, que é quem
primeiro terá instrução com operando para atravessar `$FFFF`.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| todas as 121 ROMs | 0/121 | 0/121 |

Sem mudança, e não podia haver: `gb-cli run` ainda sai `2` (o item é o 1.12).
Testes do workspace: **131 → 142**.

MSRV conferida à mão pela quarta vez (nota 13): `cargo +1.85 test --all` dá
**142/142**. Esta iteração introduziu `const fn` com `match` sobre enum e
`matches!` em contexto const — nada mais novo que a MSRV.

## Revisão cruzada (segundo modelo)

Não executada: `scripts/review.sh` continua sem `REVIEWER_CMD` configurado
(`STATUS.md`, nota 5).

## Decisões de arquitetura

1. **A CPU não é dona do `Bus`; recebe `&mut Bus` por M-cycle.** É o que o
   `CLAUDE.md` § Arquitetura manda, e evita ter de inventar um tipo `GameBoy`
   antes de haver quem o queira — quem possui os dois hoje são os testes, e a
   partir do 1.12 será o `gb-cli run`.

2. **Não foi preciso extrair interface de memória do `Bus`.** O `STATUS.md`
   previa que testar opcode sem cartucho pediria isso ("este é o 'então'"). Não
   pediu: um cartucho de teste de 32 KiB com o programa em `$0100` custa seis
   linhas de `fn machine()`, e o `Bus` continua `struct` concreto, sem vtable no
   caminho mais quente do emulador. A previsão do 1.2a estava certa quanto ao
   custo de extrair *depois*; errou ao supor que seria preciso.

3. **Nenhuma tabela de micro-operações.** Um `enum MicroOp { ReadImmediate,
   Internal, … }` é o desenho que os 245 opcodes restantes vão querer, e escrevê-lo
   agora seria a nota 8 pela sexta vez: abstração sem nada que a exercite passa
   verde por vacuidade. O `State` de hoje nomeia os M-cycles das duas instruções
   que existem, o `match` é total sem `_ =>`, e o 1.4 vai ter de mexer aqui — que
   é quando haverá três casos de onde generalizar em vez de um palpite.

4. **`Lockup` tem duas variantes com o mesmo efeito e origens opostas.**
   `IllegalOpcode` é hardware (a spec diz que trava); `UndecodedOpcode` é este
   emulador estar em 2 de 256. Colapsar as duas faria o `gb-cli` reportar "a ROM
   executou lixo" quando a resposta é "falta implementar" — e as duas se
   consertam em lugares diferentes. Parar, em vez de entrar em pânico, é o que
   mantém o `gb-core` como máquina de estados (mesma escolha do `NoMbc`, 0.4).

## Notas

**`JP u16` entrou no 1.3, e o item não pedia.** O 1.3 pede o laço; o 1.10 pede
os jumps. Mas com só `NOP` decodificado — 1 M-cycle, cujo único M-cycle é o
fetch — a máquina de estados **não tem estado**: instruction-stepped e
cycle-stepped dão exatamente o mesmo resultado, e a R2 fica sem teste que a
separe do que ela proíbe. Isso não é argumento, é medição: contra o esqueleto A,
`a_run_of_nops_advances_the_pc_one_byte_per_step` **passou**. O PR teria fechado
verde com a R2 violada.

`JP u16` é a instrução mais barata que fecha esse buraco — quatro M-cycles, três
tipos de passo (`fetch`, `read`, `internal`), zero flags, zero condição. O que o
1.10 tem de difícil é o timing condicional (`8 / 12`, `12 / 24`), e isso ficou lá
inteiro. O ROADMAP foi anotado nos dois itens.

**A § CPU Instruction Set do `02-cpu.md` não serve para implementar opcode.** A
conversão HTML→Markdown do `fetch-reference-docs.sh` achatou as tabelas de
instrução em listas de layout de bits soltas: `nop` aparece como oito linhas
`| 7 | 0 |`…`| 0 | 0 |`, sem uma palavra sobre o que a instrução faz, e os
placeholders `r8`/`r16`/`cond` viraram uma tabela única com valores repetidos e
sem cabeçalho que diga a qual dos quatro grupos cada bloco pertence. Quem for
implementar 1.4–1.11 lendo aquele arquivo não vai achar semântica nenhuma — a
fonte útil é o `03-opcodes.md`, e para prosa o link que a própria seção dá
(`gbz80(7)`), que não está em `docs/reference/`.

Isso é um modo de falha novo da R1, e o quarto: não é "não leu" (original), nem
"a spec é omissa" (nota 19), nem "a spec é ambígua" (nota 21) — é **a spec local
está corrompida na conversão e parece completa**. O arquivo tem 890 linhas e
tabelas bem formadas; nada nele avisa que o conteúdo se perdeu. Não foi
consertado aqui porque `01-`…`09-` são gerados e mexer à mão contraria o
`docs/reference/README.md`; o conserto é no gerador, e é decisão de projeto, não
contrabando de iteração (mesma linha que a 0008 traçou na nota 17).
