# Iteração 0023 — `SUB a,r8`, `SBC a,r8` e `CP a,r8`: a mesma letra, empréstimo em vez de carry

- **Data:** 2026-07-26
- **Item do roadmap:** 1.6b
- **PR:** #27
- **Duração:** ~1 sessão
- **Custo reportado:** n/d — sessão interativa de Claude Code, fora do `loop.sh`
- **Turnos:** 1

## Objetivo

Os 24 opcodes `$90`–`$9F` (`10 010 rrr` e `10 011 rrr`) e `$B8`–`$BF`
(`10 111 rrr`): `SUB`, `SBC` e `CP` sobre os oito `r8`. `N` passa a ser `1`
literal, `H`/`C` viram empréstimo em vez de carry, e `CP` é a primeira das oito
operações da ALU que não escreve em `A`.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| gbops | linhas `90`–`9F`, `B8`–`BF` (flags, T-cycles, coluna de M-cycles) | `docs/reference/03-opcodes.md` |
| Pan Docs | § The Carry Flag (*"lower than zero […] like in Z80 and x86"*) | `docs/reference/02-cpu.md` |
| Pan Docs | § The BCD Flags (*"H indicates carry for the lower 4 bits of the result"*, sem dizer de qual operação) | `docs/reference/02-cpu.md` |
| `STATUS.md` | Handoff da 0022 (`Próxima tarefa`): cinco armadilhas pré-anunciadas | — |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | flags/timing | *(nenhum de conta)* — `N=1` literal, `H`/`C` como empréstimo, o `C` de entrada do `SBC` contando para o nibble, `CP` sem escrita e o par `CP A,A`/`SUB A,A` foram implementados direto pela leitura da spec e do handoff, sem passar por uma versão errada. | idem | — |
| 2 | processo (R7) | `alu.rs` com os comentários explicando a inversão de magnitude e a distinção `writes_result` no mesmo estilo de prosa do 1.6a. | R7: teto de 12% de comentário por arquivo — o arquivo é curto (52 linhas de código) e três blocos de prosa já estouram a razão. | `crates/gb-core/tests/comment_density.rs` reprovou (`alu.rs: 9/61 = 14%`) antes do `cargo test --all`. |
| 3 | processo (controle negativo) | Que os oito arquivos de sub-item anteriores (1.4a–1.5d, 1.6a) continuariam verdes sem alteração. | `decoded_elsewhere`/`previously_decoded` é lista **duplicada de propósito** por arquivo (nota do `STATUS.md`) — quem decodifica opcode novo tem de vir atualizar as listas dos vizinhos. | `cargo test --all` reprovou em `cpu_add_adc_r8.rs`, depois em `cpu_ld_absolute_ff00.rs`, e a varredura manual dos sete arquivos restantes achou o oitavo (`cpu_ld_r8_u8.rs`, que usa uma variável inline `previously_decoded` em vez da função `decoded_elsewhere` — nome diferente, mesmo papel, quase escapou do `grep`). |

O #1 é o que a **nota 41** manda registrar como o que é: o handoff da 0022
pré-anunciou as cinco armadilhas letra por letra (`N` literal, `H` invertido,
`SBC` consumindo o `C` de entrada no nibble, `CP` sem escrita, o par `CP
A,A`/`SUB A,A`), então um `nenhum` de conta aqui mede o aviso, não uma
descoberta nova. O achado real da iteração é processual: **#3 é o mesmo atrito
que a 0022 já tinha sofrido** (lá foram oito `Edit`s de uma linha; aqui foram
oito de novo, mais um nono arquivo que quase escapou por ter nome de variável
diferente da função-padrão) — ver § Notas.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| todas as 11 | 0/121 | 0/121 |
| testes do workspace | 252 | 266 |

## Revisão cruzada (segundo modelo)

Não houve. `scripts/review.sh` continua sem `REVIEWER_CMD` configurado
(`STATUS.md`, nota 5).

## Decisões de arquitetura

**Nenhuma nova.** `AluOp` ganhou três variantes (`Subtract`, `SubtractWithCarry`,
`Compare`) e `alu::apply` passou a despachar para duas funções internas,
`add`/`subtract` — mas o desenho de `mcycle.rs` não mudou: os seis novos opcodes
de `(HL)` (`$96 $9E $BE`) reusam o `State::AluFromHl(AluOp)` que a 0022 já havia
generalizado prevendo exatamente este sub-item (*"os sítios chegam no 1.6b e no
1.6c, e aí `AluFromHl(AluOp)` já os cobre sem linha nova"* — confirmado). A
única decisão nova é local ao `alu.rs`: `subtract` recebe um booleano
`writes_result` em vez de uma quarta variante de função, porque `SUB`/`SBC`/`CP`
têm a mesma conta e só a escrita em `A` diverge.

## Notas

### O atrito dos controles negativos, pela segunda vez — e por que ele não foi extraído

A 0022 já havia registrado este exato atrito (oito arquivos com uma lista
`decoded_elsewhere` que teve de ganhar `(0x80..=0x8F)` quando o 1.6a saiu do
`UndecodedOpcode`). Esta iteração repetiu o padrão: nove arquivos (os mesmos
oito, mais `cpu_ld_r8_u8.rs`, que usa uma variável local `previously_decoded`
em vez da função `decoded_elsewhere` — o `grep -rl "fn decoded_elsewhere"`
inicial não a pegou, e só a comparação manual contra a lista de arquivos que
fazem varredura de `0x00..=0xFF` achou a nona). A tentação de extrair a lista
para um módulo compartilhado apareceu de novo e foi recusada de novo, pela
mesma razão que o `STATUS.md` já registra: **um ponto de verdade que se
atualiza sozinho para de forçar quem adiciona opcode a vir declarar o que
adicionou** — e é essa obrigação, não a lista em si, que o teste protege.

O procedimento que vale para o 1.6c (que vai repetir isto pela terceira vez,
com `$A0`–`$B7`): **procurar por `0x00..=0xFF` e por `for opcode in`, não só por
`fn decoded_elsewhere`** — nomes de função divergem entre arquivos mais antigos
e mais novos, e só o padrão de uso (varrer os 256 opcodes) é estável.

### O risco de instante do `(HL)` já estava coberto antes de este PR existir

A 0022 gastou a maior parte do esforço de mutação achando o M16 (ler `(HL)`
dentro do fetch em vez de no M2) porque aquele era o primeiro opcode de ALU com
operando em memória. Aqui `$96`/`$9E`/`$BE` chamam a **mesma** `alu_from_hl`,
já corrigida — não há decisão de instante nova para errar. O teste
`the_bus_access_of_96_9e_be_happens_in_the_m2_and_not_during_the_fetch` troca
a memória entre os dois `step`, como o teste homônimo da 0022, e serve para
confirmar que o compartilhamento de função também compartilha a correção — não
para caçar um mutante novo.

### Atrito

- **R7 reprovou `alu.rs` de primeira, de novo** (nota da 0022, mesma causa): um
  arquivo curto onde três blocos de prosa já estouram 12%. Reduzido a três
  comentários de uma linha (5%), apontando para o `STATUS.md`/ROADMAP em vez de
  reexplicar a fórmula no código.
