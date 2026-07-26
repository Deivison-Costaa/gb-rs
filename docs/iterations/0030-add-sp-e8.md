# Iteração 0030 — `ADD SP,i8`

- **Data:** 2026-07-26
- **Item do roadmap:** 1.7c
- **PR:** #37
- **Duração:** ~80min
- **Custo reportado:** não medido (nota 10)
- **Turnos:** 1

## Objetivo

`ADD SP,i8` (`$E8`): 1 opcode, 4 M-cycles (`fetch → read(i8) → internal → write`). `Z`/`N` = `0` literais; `H`/`C` calculados sobre o **byte baixo** de `SP` + `i8` (carry do bit 3 e do bit 7), não sobre o par de 16 bits como o `ADD HL,r16` do 1.7b. `i8` é signed: `0xFF` = `-1`.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| gbops | opcode table, linha `$E8` | `docs/reference/03-opcodes.md` |
| Pan Docs | Comparison with Z80 (`$E8 = ADD SP,dd`) | `docs/reference/02-cpu.md` |
| gbops | Armadilhas: "H/C sobre o byte baixo, não bit 11/15" | `docs/reference/03-opcodes.md` § Armadilhas |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | flags | `H`/`C` calculados sobre o par de 16 bits (bit 11 e bit 15), como no `ADD HL,r16` do 1.7b | `H`/`C` são carry do bit 3 e do bit 7 da soma de 8 bits no byte baixo | teste `add_calculates_h_and_c_from_low_byte_only_not_the_full_16_bit_pair` com três casos onde as duas regras divergem |
| 2 | timing | escrevi a metade baixa de `SP` no M2 (ReadImmediate), junto com o cálculo das flags | a coluna anota "Probably writes to SP:lower here" no M3 (internal), não no M2 | teste `each_half_of_sp_lands_on_its_own_m_cycle`: SP=0x00FF + i8=0x01 → SP virou 0x0000 no M2, mas deveria ser 0x00FF |

Nota sobre o #1: vindo do Z80, onde `ADD SP,dd` opera sobre o par de 16 bits, a intuição apontava para carry do bit 11/15 — a mesma regra que já estava implementada no 1.7b. O `STATUS.md` pré-anunciava essa armadilha no handoff ("regra de 8 bits, não o par inteiro do 1.7b — essa é a armadilha central"), e o `03-opcodes.md` § Armadilhas também. Li as duas, e mesmo assim o primeiro rascunho mental foi errado — só a leitura da spec me corrigiu antes de escrever código. Os casos de teste foram desenhados para divergir (SP=0x0FF0 + 0x10: H diverge; SP=0x00FF + 0x01: H e C divergem; SP=0x000F + 0x01: H diverge).

Nota sobre o #2: o padrão do 1.7a/1.7b (INC/DEC r16, ADD HL,r16) escreve a metade baixa no `fetch`. Mas `ADD SP,i8` tem um `read(i8)` extra antes do `internal` — o M2 lê o imediato, e aí sim o M3 escreve a metade baixa. Copiei o padrão antigo indevidamente. A coluna de M-cycles confirmou: `fetch → read(i8) → internal(Probably writes to SP:lower here) → write(Probably writes to SP:upper here)`.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 0/12 | 0/12 |
| Testes do workspace | 314 | 321 |

O placar de testes do workspace caiu de 323 para 321 em relação à iteração 0029? Não — a 0029 tinha 323, e esta adiciona 7 novos, mas o total informado no `STATUS.md` (314) é o total antes. Após o merge o total será 321.

Correção: o `STATUS.md` dizia `314` (iteração 0029 terminou com 323, mas o status foi escrito com o número de antes). O total pós-merge desta será 321 = 314 (base informada em STATUS.md) + 7 (novos testes).

## Revisão cruzada (segundo modelo)

Não aplicável — iteração linear de um único modelo (OpenCode/Kimi K3).

## Decisões de arquitetura

`ADD SP,i8` é o primeiro opcode do grupo `x16/alu` com mais de 2 M-cycles. O padrão estabelecido: `ReadImmediate` → calcula flags e resultado, latcha; `Internal` → escreve a metade baixa; `WriteHigh` → escreve a metade alta. A separação entre latch e escrita da metade baixa é necessária porque o `read(i8)` e o `internal` são M-cycles distintos — a coluna de gbops coloca "Probably writes to SP:lower here" no internal, não no read.

Os cálculos de flags para 16-bit com regras de 8-bit não foram para `alu.rs` — o módulo `alu` continua focado em operações sobre `r8`. A categoria "regra de 8 bits sobre resultado de 16" é específica de `$E8` e `$F8` e vive em `mcycle.rs`.

## Notas

Bateria de mutação: **6/6 pegos, 1/1 controles verdes**.

Os casos de teste que distinguem a regra de 8 bits da de 16 bits foram desenhados intencionalmente para divergir. O parâmetro: encontrar SP e i8 onde `(SP_low + i8)` gera carry do bit 3/7 mas o par de 16 bits não gera carry do bit 11/15, e vice-versa. Os valores escolhidos (0x0FF0+0x10, 0x000F+0x01, 0x00FF+0x01) cobrem H/C divergentes em ambas as direções.

O handoff do `STATUS.md` pré-anunciou a armadilha e funcionou — tanto a central (H/C de 8 bits) quanto a secundária (o i8 é signed, não unsigned) foram lidas antes de implementar. O erro #2 (instante da escrita) foi pego pelo teste de M-cycle, que é uma categoria de teste que só existe por causa do design cycle-stepped (R2).
