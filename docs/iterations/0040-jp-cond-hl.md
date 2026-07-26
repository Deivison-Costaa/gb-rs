# Iteração 0040 — JP cc,u16 + JP HL

- **Data:** 2026-07-26
- **Item do roadmap:** 1.10b

## Objetivo

Decodificar `JP cc,u16` (`$C2 NZ`, `$CA Z`, `$D2 NC`, `$DA C`) e `JP HL` (`$E9`).
Os condicionais reaproveitam a forma de M-cycle do `JumpImmediate` ($C3, incondicional)
com uma decisão condicional após a leitura dos dois bytes do operando: sempre leem os
2 bytes (12 T) e gastam mais 1 M-cycle de `internal` (16 T) se a condição bater.
`JP HL` copia `HL` para `PC` em 1 M-cycle.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| gbops | control/br (C2, CA, D2, DA, E9) | `docs/reference/03-opcodes.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | Poderia ler os bytes do operando só se a condição batesse, economizando 1 M-cycle (estilo Z80) | Os dois bytes do operando são sempre lidos, com ou sem desvio — o extra é o `internal` depois | teste `jp_conditional_takes_three_m_cycles_when_not_taken` (PC avança sobre o operando mesmo sem desvio) |
| 2 | timing | `JP HL` poderia ser 2 M-cycles (fetch + internal como o `$F9`) | 1 M-cycle: fetch faz tudo | teste `jp_hl_consumes_one_m_cycle` (termina no M1) |
| 3 | nenhum | — | — | — |

> O erro #1 foi previsto antes de ler a spec — medi a minha suposição de que o Z80
> condiciona a leitura do operando. A coluna do `03-opcodes.md` mostra claramente
> que as duas leituras aparecem nos dois ramos (com/sem desvio). O #2 é o oposto do
> erro do `$F9` na 0021: lá, o `internal` era um passo a mais que parecia natural;
> aqui, o `fetch` único parecia curto demais.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 0/11 | 0/11 |

Nenhuma ROM de teste passa ainda — o M1 está em construção e não há código gráfico.

## Bateria de mutação

**6/6 pegos, 2/2 controles verdes.**

| # | Mutação | Falhas | Testes que pegaram |
|---|---|---|---|
| M1 | `decode_jp_condition`: NZ → Z | 4 | `jp_nz_taken_when_z_is_clear`, `jp_nz_not_taken_when_z_is_set`, +2 timing |
| M2 | `JP_HL` não atualiza PC | 1 | `jp_hl_copies_hl_to_pc_in_one_m_cycle` |
| M3 | `ReadHighByte` sempre retorna `Fetch` (nunca toma desvio) | 7 | todos os `*_taken_*`, `jp_u16_*`, + timing |
| M4 | `ReadHighByte` sempre toma desvio | 5 | todos os `*_not_taken_*`, + timing |
| M5 | Byte order trocado (low como high, high como low) | 7 | todos os `*_taken_*`, `jp_u16_*` (endereço errado) |
| M6 | Bloco `JP_COND_MASK` removido do fetch | 11 | todos os JP condicionais + `all_jp_opcodes_are_recognized` |
| C1 | `NOP` alterado para `0x01` | 0 JP / 1 controle* | `jp_opcodes_the_rest_of_block...` (controle negativo) |
| C2 | `RLA` zera Z incorretamente | 0 JP / 2 rotate | `cpu_rotate_accum` (fora da suíte JP) |

*O controle negativo pegou a inconsistência com `decoded_elsewhere`, que diz que
`0x00` é decodificado — mas com NOP = 0x01, `0x00` não é mais. Isso é esperado
e prova que o controle não é cego.

## Decisões de arquitetura

- **`State::JumpImmediate` ganhou um campo `Condition`.** O `$C3` (incondicional)
  passa `Condition::Always` e o fluxo de M-cycles é idêntico ao de antes. Os
  condicionais passam a condição decodificada e a decisão acontece no
  `ReadHighByte`: se a condição bater, transita para `SetProgramCounter`
  (+1 M-cycle); senão, vai direto para `Fetch`.

- **`$E9` é resolvido direto no fetch**, sem estado intermediário. É o caso
  mais simples do 1.10 inteiro: uma linha.

- As constantes `JP_COND_MASK = 0xE7` e `JP_COND_PATTERN = 0xC2` cobrem os
  quatro opcodes condicionais sem colidir com `$C3` (que já é tratado antes).
  `$E9` não casa com a máscara (`$E9 & $E7 = $E9 ≠ $C2`), então também não
  colide.

- `decode_jp_condition` extrai bits 3-4 do opcode (`(opcode >> 3) & 0b11`),
  a mesma posição do JR — mas o `decode_jr_condition` usa máscara de 3 bits
  (`0b111`) porque o bit 5 difere entre os grupos. Mantive funções separadas
  para não mexer no que já funciona.

## Notas

- O `Condition` enum e `evaluate_condition` vieram prontos da 0039. Zero reescrita.
- A tentativa anterior (0040 abandonada) deixou 17 testes em `iter/0040-jp-cond-hl`
  que não foram reaproveitados — os 20 testes desta iteração foram escritos do zero
  seguindo o padrão do `cpu_jr.rs`.
- O `decoded_elsewhere` precisou de +5 opcodes (uma linha), sem mexer em nada
  existente. O teste `decoded_elsewhere_single_source` confirma que não criei
  cópia em arquivo de teste.
- O `JumpImmediate` existente virou condicional sem código duplicado: a única
  diferença é o `if Self::evaluate_condition(...)` no `ReadHighByte`, que o
  `Always` trivializa.
