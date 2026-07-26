# Iteração 0025 — `alu a,imm8`: a mesma ALU, o operando vindo de outro lugar

- **Data:** 2026-07-26
- **Item do roadmap:** 1.6d
- **PR:** #30
- **Duração:** ~1 sessão
- **Custo reportado:** n/d — sessão interativa de Claude Code
- **Turnos:** 1

## Objetivo

Os 8 opcodes `$C6 $CE $D6 $DE $E6 $EE $F6 $FE` (`11 ooo 110`): as mesmas oito
operações do 1.6a/1.6b/1.6c (`Add`/`AddWithCarry`/`Subtract`/
`SubtractWithCarry`/`Compare`/`And`/`Xor`/`Or`, já reunidas em `AluOp`), com o
operando vindo do `PC` em vez de um `r8`. Fecha o quarto dos cinco sub-itens
do 1.6.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| gbops | linhas `C6`/`CE`/`D6`/`DE`/`E6`/`EE`/`F6`/`FE` (flags, T-cycles, coluna de M-cycles) | `docs/reference/03-opcodes.md` |
| `STATUS.md` | Handoff da 0024 (`Próxima tarefa`): nota 43 antecipada | — |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing/teste | O `PC` sendo testemunha (nota 43) era o único risco de instante a cobrir — copiei a estrutura de `cpu_add_adc_r8.rs` sem reparar que o operando de teste (`0x0C`) dava o mesmo resultado para `XOR` e `OR` (`0x21^0x0C == 0x21\|0x0C == 0x2D`, porque `A` e o operando não compartilham bit nenhum) e também não distinguia `AND` de `CP` sem escrita. | Nenhuma linha de spec — achado da própria bateria de mutação: mutar `XOR_A_IMM8`↔`OR_A_IMM8` e `AND_A_IMM8` para `Compare` não quebrava teste nenhum com aquele operando. | Bateria de mutação (M5, M6, M7): troquei o operando de teste por `0x2A`, que compartilha bits com `SEED_A` sem ser subconjunto nem superconjunto — os três resultados (`0x20`/`0x0B`/`0x2B`) ficam distintos entre si e do `A` original. |
| 2 | cobertura | Assumi que testar `ADC`/`SBC` consumindo o carry de entrada bastava — não escrevi o controle inverso (`ADD`/`SUB` **ignorando** o carry), ao contrário de `cpu_add_adc_r8.rs`/`cpu_sub_sbc_cp_r8.rs`, que têm os dois lados. | idem | Bateria de mutação (M1, M3): trocar `ADD_A_IMM8`/`SUB_A_IMM8` para as variantes "com carry" passava em todos os 8 testes originais — escrevi `add_and_sub_ignore_the_incoming_carry` depois de ver o mutante sobreviver. |

Nenhum erro de hardware: a coluna de flags do `alu a,imm8` é letra por letra a
mesma dos três sub-itens anteriores (`AluOp`/`alu::apply` não mudaram nada),
e o risco de instante (nota 43, `PC` como testemunha) já vinha identificado
no handoff da 0024. Os dois erros desta iteração são de **cobertura de
teste**, não de spec — e os dois só apareceram porque a bateria de mutação é
obrigatória, não porque alguém os percebeu lendo o código.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| todas as 11 | 0/121 | 0/121 |
| testes do workspace | 279 | 288 |

Bateria de mutação: **10/10 pegos, 2/2 controles verdes** (8 trocas de
`AluOp` por opcode + 2 mutantes de instante — esquecer o pós-incremento do
`PC` e ler de `(HL)` em vez do `PC`).

## Revisão cruzada (segundo modelo)

Não houve. `scripts/review.sh` continua sem `REVIEWER_CMD` configurado
(`STATUS.md`, nota 5).

## Decisões de arquitetura

**Nenhuma nova.** `mcycle.rs` ganhou um estado (`State::AluImmediate(AluOp)`)
e um método (`alu_immediate`), espelhando `AluFromHl`/`alu_from_hl` — a única
diferença é a origem do byte (`self.read_at_pc(bus)` em vez de
`bus.read(self.registers.hl())`). Os oito opcodes são casados por literal
(`ADD_A_IMM8 => ...`), não por máscara: ao contrário do `10 ooo rrr` do
1.6a/b/c — onde uma máscara isola o campo `r8` e deixa `ooo` livre —, aqui não
há campo variável nenhum a isolar (`11 ooo 110` tem os 8 bits fixos por
opcode), então uma máscara só reintroduziria a extração de bits que o projeto
evita (invariante: não há tabela de micro-operações).

## Notas

### O `PC` é testemunha, o `(HL)` não é — e o teste principal usa isso

A nota 43 (0021) e a nota 45 (0022) descrevem os dois lados da mesma classe de
erro de instante. No 1.6a, o operando vinha de `(HL)` e não deixava rastro —
quem pegava o M16 era trocar a memória **entre** os dois `step`. Aqui o
operando vem do `PC`, e o `PC` **é** legível entre os passos: se o imediato
fosse lido dentro do fetch (dois acessos no mesmo `step`, quebrando a
invariante do 1.3), o `PC` já estaria em `entry+2` depois do primeiro `step`,
não em `entry+1`. `the_immediate_is_read_in_the_second_m_cycle_and_the_pc_proves_it`
não precisa trocar memória no meio — só ler o `PC` depois do M1.

### O operando de teste importa tanto quanto a asserção

Os dois erros de primeira tentativa desta iteração vieram do mesmo lugar: um
operando de teste (`0x0C`) escolhido sem verificar se ele **distingue** as
operações que o teste alega cobrir. `0x0C` contra `SEED_A = 0x21` não
compartilha bit nenhum, então `XOR` e `OR` colapsam no mesmo resultado — a
bateria de mutação (não a leitura do teste) foi o que expôs isso. Formaliza
o que a nota 25 já dizia sobre controle negativo, agora sobre dado de entrada:
escolher o operando que separa os casos é parte de escrever o teste, não um
detalhe.

### O controle negativo, pela quinta vez

`decoded_elsewhere`/`previously_decoded` ganhou
`0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE` nos dez arquivos que
já tinham a função — nenhum arquivo novo precisou de `(0xA0..=0xB7)` (a 0024
já havia fechado essa lacuna em todos). O procedimento da 0023/0024
(`grep` por `0x00..=0xFF`/`for opcode in`) continua sendo o que acha os dez de
uma vez.
