# Iteração 0021 — `LD SP,HL` e `LD (u16),SP`, os dois avulsos do `x16/lsm`

- **Data:** 2026-07-26
- **Item do roadmap:** 1.5d
- **PR:** #25
- **Duração:** —
- **Custo reportado:** —  <!-- sessão interativa; ver STATUS.md, nota 10 -->
- **Turnos:** 2 sessões (ver abaixo)

## Objetivo

`$F9` em 2 M-cycles e `$08` em 5, fechando o grupo 1.5 e o `x16/lsm`.

## A iteração foi interrompida no meio, e a primeira metade não tem relato

Este é o primeiro registro do projeto escrito por **duas** sessões, e a primeira
morreu sem escrever nada. O que ela deixou em disco, sem nenhum commit: o
`crates/gb-core/tests/cpu_ld_stack_pointer.rs` com 14 testes, as 52 linhas do
`mcycle.rs` e os sete arquivos de teste antigos com `0x08`/`0xF9` acrescentados
ao `decoded_elsewhere`. A segunda sessão (esta) leu a spec, revisou o código dela
contra a spec, corrigiu o que estava errado, rodou o passo 7 e escreveu isto.

**Isso enfraquece o campo `Erros de primeira tentativa` de um jeito que precisa
ficar dito.** O campo mede, normalmente, o que quem escreveu percebeu que
escreveu errado. Aqui ele mede outra coisa: o que **sobreviveu** até um revisor
que chegou depois. Erro cometido e consertado dentro da primeira sessão não
deixou rastro em lugar nenhum e é irrecuperável. A tabela abaixo tem três linhas
e nenhuma delas é de intuição de hardware — o que **não** autoriza concluir que
não houve nenhuma.

O que a evidência sustenta sobre **onde** a primeira sessão morreu é preciso: ela
parou entre o passo 6 e o passo 7. O código compilava e os 14 testes passavam;
`cargo clippy -- -D warnings` reprovava com dois erros (erro #1). Escrever os
testes e a implementação aconteceu; rodar o portão de qualidade, não.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| gbops | linhas `08` e `F9` | `docs/reference/03-opcodes.md` |
| gbops | linhas `33` (`INC SP`), `E8` (`ADD SP,i8`), `F8`, `C3` | `docs/reference/03-opcodes.md` |
| Pan Docs | § CPU Instruction Set, cabeçalhos `ld r16, imm16` e `add sp, imm8` | `docs/reference/02-cpu.md` |

As duas colunas, transcritas:

- `$F9` — 1 byte, 8 T-cycles, flags `- - - -`: `fetch → internal`.
- `$08` — 3 bytes, 20 T-cycles, flags `- - - -`:
  `fetch → read(u16:lower) → read(u16:upper) → write(SP:lower->(u16)) → write(SP:upper->(u16+1))`.

Os dois layouts de bits do `02-cpu.md` são de oito bits constantes, sem campo de
placeholder — e nenhum dos dois mora sob o cabeçalho do próprio mnemônico: o do
`$08` está sob `ld r16, imm16` e o do `$F9` sob `add sp, imm8`, porque a conversão
para Markdown funde instruções consecutivas sob o primeiro cabeçalho do grupo
(nota 38). É o `03-opcodes.md` que confirma a codificação.

## Erros de primeira tentativa

> Categorias: `flags`, `timing`, `endereçamento`, `borrow-checker`, `API-Rust`,
> `nenhum`. Nesta iteração o campo mede o que sobreviveu à primeira sessão, não
> o que ela percebeu — ver a seção acima.

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | quatro variantes de fase terminadas em `Byte`, e `Registers` importado no teste | — | `cargo clippy -- -D warnings`: `enum_variant_names` e `unused_imports`. Não tinha sido rodado |
| 2 | timing | que o `$F8` fosse precedente para "o último `internal` é onde o registrador de 16 bits recebe" | o `$F8` tem `internal` **pelado**. Quem fala do instante do `SP` é o `$33` e o `$E8`, e os dois **partem o par em duas metades, a baixa primeiro**, com `Probably` | revisão contra a tabela, na segunda sessão |
| 3 | timing (suíte) | que ler os dois bytes do endereço em M-cycles separados estivesse coberto | `read(u16:lower)` e `read(u16:upper)` são passos distintos | bateria de mutação: o mutante M11 passou verde nos 14 testes da primeira sessão |

**O erro #1 é o único que diz onde a sessão morreu**, e por isso vale mais como
dado de processo do que como defeito: dois erros de clippy, nenhum deles de
hardware, os dois de trinta segundos de conserto. O que eles atestam é que os
passos 6 e 7 do protocolo são separados de verdade — a primeira sessão fez o
código passar nos testes que ela mesma escreveu e não chegou ao portão.

**O erro #2 é o interessante, e é a nota 21 com uma dobra nova.** A primeira
sessão acertou o diagnóstico (a coluna do `$F9` não decide o instante; escrever o
par no `internal` é escolha) e escreveu isso no teste, com todas as letras. O que
ela errou foi a **justificativa**: citou o `$F8` como precedente de projeto, e o
`$F8` não sustenta nada — a coluna dele é `fetch → read(i8) → internal`, sem seta
e sem anotação, exatamente o mesmo silêncio do `$F9`. Silêncio citado como apoio
vira apoio inventado.

E há mais do que silêncio. Três linhas acima na mesma tabela:

```
33 | INC SP     | fetch(Probably writes to SP:lower here) → internal(Probably writes to SP:upper here)
E8 | ADD SP,i8  | fetch → read(i8) → internal(Probably writes to SP:lower here) → write(Probably writes to SP:upper here)
```

As **únicas** duas linhas do arquivo que dizem quando o `SP` recebe um valor
partem o par em duas metades, em M-cycles diferentes, a baixa primeiro. Se
alguma coisa da spec local fosse aplicada ao `$F9` por analogia, seria essa — e
ela aponta para o **contrário** do que está implementado. Não foi aplicada, e por
dois motivos que o teste agora carrega: as duas linhas são `x16/alu` e não
`x16/lsm`, e as duas vêm com `Probably`, que é o gbops declarando que está
chutando. A escolha do projeto (par inteiro no último passo, como o `JP u16` —
cujo `internal` é o único que a tabela anota de verdade, com `branch decision?`)
ficou onde estava; a mensagem do assert é que foi reescrita.

Nada observável hoje depende disso: não há timer nem PPU para ver o `SP` no meio
de uma instrução. Quando houver, e se a Mooneye cobrar, o lugar de mudar é uma
função de duas linhas e o teste que a prende já existe e diz que ela é escolha.

## Bateria de mutação

**15 mutantes, 15 pegos. 2 controles negativos, 2 verdes.**

| # | Mutante | Algozes |
|---|---|---|
| M1 | escreve a metade alta em `(u16)` e a baixa em `(u16+1)` (a ordem do `PUSH`) | 4 |
| M2 | junta as duas escritas no M4 e gasta o M5 num `internal` | **1** |
| M3 | lê o endereço big-endian (byte alto primeiro) | 4 |
| M4 | `saturating_add(1)` no endereço da segunda escrita | **1** |
| M5 | `$F9` escreve o `SP` no fetch em vez de no `internal` | **1** |
| M6 | `$F9` copia só a metade baixa (`L` → `SP:lower`) | 3 |
| M7 | `$F9` invertido: `HL` recebe `SP` | 3 |
| M8 | `$08` por máscara `00 rrr 000`: leva `STOP` e os cinco `JR` | 8 varreduras dos 256 |
| M9 | `$F9` reconhecido como `$F8` (rouba `LD HL,SP+i8`) | 12 |
| M10 | as duas escritas do `$08` vão para o mesmo endereço | 4 |
| M11 | `$08` lê os dois bytes do endereço no M2 e gasta o M3 num `internal` | **1** (teste novo) |
| M12 | `$F9` em 1 M-cycle: o fetch copia e volta para `Fetch` | 3 |
| M13 | `$08` escreve o endereço no `SP` depois de gravar | 3 |
| M14 | `$08` zera `F` no último M-cycle | 2 (guarda de ausência) |
| M15 | `$F9` zera `F` | 2 (guarda de ausência) |

| # | Controle negativo | Resultado |
|---|---|---|
| C1 | troca a ordem dos dois braços literais no `fetch` | verde |
| C2 | `as`/shift em vez de `to_be_bytes` nas duas escritas | verde |

**O achado é o M11, e ele é sobre a suíte, não sobre o hardware.** Ler os dois
bytes do endereço no M2 e gastar o M3 num `internal` dá o mesmo endereço, a mesma
memória, o mesmo `PC` no fim e os mesmos 20 T-cycles. O que ele quebra é a
invariante do 1.3 — *uma chamada de `Cpu::step` faz no máximo um acesso ao
barramento* —, que é justamente a invariante que nenhum teste de estado final
consegue ver. Contra os 14 testes da primeira sessão o mutante ficou **verde**, e
contra os 239 testes do workspace inteiro também.

O que o pega é uma asserção de `PC` **entre** os M-cycles, e é a nota 32 outra
vez: a primeira sessão seguiu a doutrina da memória (asserção depois de cada um
dos 5 passos, lendo entre as duas escritas — é o teste
`the_two_writes_are_the_last_two_m_cycles_one_byte_each`, único algoz de M2) e
não a aplicou ao **operando**. A instrução tem dois lados que se lêem por
M-cycle, e ela cobriu um. `the_two_operand_bytes_are_read_one_per_m_cycle` é o
outro, e é o único algoz de M11.

Quarta linha de mutantes de instante com **um** algoz só (M2, M4, M5, M11), e a
proporção segue: 9/10 (0015), 10/11 (0016), 7/8 (0018), 8/10 (0019), 9/10 (0020),
**11/15** aqui. A nota 40 se confirma pelo lado que ela previa — o `$08` é a
instrução que mais escreve do projeto, e os dois mutantes de instante dele (M2,
M11) são exatamente os dois que têm um algoz único.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| todas (121 ROMs) | 0 | 0 |
| testes do workspace | 225 | 240 |

Sem regressão no `scoreboard.sh`. MSRV conferida à mão pela nona vez
(`cargo +1.85 test --all`: **240/240** em `rustc 1.85.1`) — nota 13, que segue
aberta, e cujo item é o 7.4.

## Revisão cruzada (segundo modelo)

Não executada: `REVIEW=0` no `scripts/loop.sh` por decisão do operador
(`STATUS.md`, notas 5 e 33). **Mas esta iteração teve, por acidente, a coisa mais
próxima disso que o projeto já produziu:** uma sessão escreveu o código e outra o
revisou contra a spec sem ter visto o raciocínio da primeira. Os erros #2 e #3
saíram daí, e nenhum dos dois teria saído de quem escreveu — o #2 é uma
justificativa que só destoa quando alguém volta à tabela, e o #3 é um ponto cego
que a nota 8(b) prevê explicitamente (mutante escrito por quem escreveu o teste
tende a herdar o mesmo ponto cego).

## Decisões de arquitetura

- **`CopyHlToStackPointer` é o primeiro estado sem carga do projeto.** Não há
  operando para carregar: os oito bits do `$F9` são constantes. O `internal`
  dele é o único M-cycle do projeto que não faz acesso **nem** decide nada — só
  marca o tempo em que a escolha do instante mora.
- **As quatro fases do `$08` mudaram de nome, e não foi cosmética.** Clippy
  (`enum_variant_names`) reprovou `ReadLowByte`/`ReadHighByte`/`WriteLowByte`/
  `WriteHighByte` por sufixo comum. Os nomes que saíram —
  `ReadAddressLow`/`ReadAddressHigh`/`WriteLowHalf`/`WriteHighHalf` — dizem o que
  os antigos escondiam: **são dois valores de 16 bits diferentes na mesma
  instrução**, o endereço que vem do imediato e o `SP` que vai para a memória. As
  quatro fases antigas sugeriam um só.
- **O `$08` não reusa o `Absolute` nem o `Cpu::access`, e a duplicação é
  conhecida.** `Absolute::ReadLowByte`/`ReadHighByte` e
  `StoreStackPointer::ReadAddressLow`/`ReadAddressHigh` têm corpo idêntico (latch
  de dois bytes vindos do `PC`), e o `Cpu::access` não serve porque ele é sobre
  `A`. Não extraí: pela doutrina do 1.4d, a abstração nasce onde a repetição
  **existe**, e o terceiro sítio chega no 1.10 (`CALL u16`, `JP cond`), que é
  quando dá para ver a forma certa em vez de adivinhá-la com dois casos. Fica na
  nota 41.

## Notas

**O `$08` é a primeira instrução do projeto com dois acessos de escrita, e o
único jeito de vê-los separados é a memória entre eles.** É o erro #1 da 0015
(`LD (HL),u8`, os dois acessos em M-cycles diferentes) na instrução que mais
escreve — e o teste que o pega é o mesmo padrão de teste, escrito de novo. A
diferença é que na 0015 havia um acesso de leitura e um de escrita, e aqui os
dois são escrita, no mesmo `SP`, em endereços vizinhos: o estado final é
indistinguível, e é por isso que M2 tem um algoz só.

**O `(u16+1)` é endereço no mapa inteiro, não dentro da região.** `LD ($FFFE),SP`
grava o byte baixo no último byte da HRAM e o alto no `IE` — as duas escritas
atravessam a fronteira de região sem nada de especial acontecer, porque a soma é
de endereço e não de índice. E `LD ($FFFF),SP` dá a volta para `$0000`, que é ROM
e engole a metade alta (`wrapping_add`, como o `SP` do `PUSH` na 0019).
`saturating_add` poria as duas metades no mesmo endereço e é o M4, com um algoz.

**O `$08` e o `PUSH` guardam o mesmo layout little-endian escrevendo em ordens
opostas.** Aqui a metade baixa vai primeiro, para o endereço mais baixo; lá a
metade alta vai primeiro, porque o endereço **desce** a cada escrita. Copiar a
ordem do vizinho — que é o movimento das notas 26/30/34/36 — inverteria o par na
memória, e é o M1, com 4 algozes. Barulhento, ao contrário dos de instante.

**Este é o fim do 1.5 e do `x16/lsm`: os 14 opcodes do grupo estão implementados**
(`01 08 11 21 31 C1 C5 D1 D5 E1 E5 F1 F5 F9`), em quatro formas de M-cycle — 3, 4,
3 e (2, 5). O `x8/lsm` fechou no 1.4d; o que sobra de `lsm` no SM83 é o `$F8`, que
está em `x16/alu` e é o 1.7.
