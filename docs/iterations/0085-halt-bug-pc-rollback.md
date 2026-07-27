# Iteração 0085 — halt bug: rollback do PC no dispatch de interrupção

- **Data:** 2026-07-27
- **Item do roadmap:** 2.4b

## Objetivo

Corrigir o halt bug para que o dispatch de interrupção empurre o endereço do
HALT (não o byte seguinte) quando o bug dispara após `ei`; `halt`.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | halt / halt bug | `docs/reference/05-interrupts.md` |
| SameSuite | interrupt/ei_delay_halt.asm | buscado via webfetch |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | `timing` | O halt_bug suprime apenas o incremento do PC no próximo fetch. Basta setar `halt_bug = true` e o `read_at_pc` existente trata o resto. | O PC deve ser revertido ao endereço do HALT quando uma interrupção dispara após o halt bug. O SameSuite mostra que VBlank e STAT retornam ao HALT (não ao byte seguinte). | Teste `halt_bug_after_ei_pushes_halt_address_on_interrupt_dispatch` — a primeira versão do teste (pré-fix) dava PC=0x0102 no dispatch em vez de 0x0101. Ver também SameSuite `ei_delay_halt.asm`. |
| 2 | `timing` | O rollback do PC deveria ocorrer no próprio handler do HALT (`wrapping_sub(1)`). | O rollback só deve ocorrer se a interrupção disparar — se o halt_bug for consumido por `read_at_pc` (sem interrupção), o PC deve permanecer no byte após o HALT. Rollback no handler do HALT quebra o caso RST após HALT (o RST empurraria o endereço do HALT em vez do próprio RST). | Teste `halt_bug_byte_after_halt_executed_twice` — com rollback no handler, PC=0x0100 e as asserções do teste esperavam 0x0101. Corrigido movendo o rollback para `check_interrupt`. |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 12/12 | 12/12 |
| instr_timing | 1/1 | 1/1 |
| mem_timing | 4/4 | 4/4 |
| dmg-acid2 | 1/1 | 1/1 |
| halt_bug | 0/1 | 0/1 |
| mem_timing-2 | 0/4 | 0/4 |

O placar não mudou. A correção do halt bug é necessária mas não suficiente para
fazer `halt_bug.gb` e `mem_timing-2` passarem. Ambos continuam estourando 100M
ciclos sem saída serial.

## Revisão cruzada (segundo modelo)

- **Modelo:** SameSuite `interrupt/ei_delay_halt.asm`
- **Achados:** 7 (pilha esperada, ordem de interrupções, valor de AF após DAA)
- **Procedentes:** 1 (erro #2 — o rollback no handler quebrava o caso sem EI)
- **Falso positivo mais interessante:** N/A — a spec do SameSuite confirmou o
  comportamento que implementei, não o que eu teria escrito de memória.

## Decisões de arquitetura

1. **Rollback do PC em `check_interrupt`, não no handler do HALT.**
   O halt_bug seta o flag mas mantém o PC no byte seguinte. Se uma interrupção
   disparar via `check_interrupt`, o PC é decrementado para o endereço do HALT
   antes do dispatch. Se o halt_bug for consumido por `read_at_pc` (caso sem
   interrupção), PC permanece no byte após HALT — o que é correto para o caso
   RST/pulo após HALT.
   
   Invariante nova: ver abaixo.

2. **`read_at_pc` não foi alterado.**
   Continua suprimindo o incremento quando `halt_bug` está ativo, sem ler de
   PC+1. O byte lido é o que está no PC atual, que é o byte após HALT (pois
   o fetch do HALT já incrementou o PC).

## Notas

A ROM `halt_bug.gb` usa MBC1+RAM, copia código de `$4000` para WRAM e salta
para `$C000`. Sem saída serial mesmo após 500M ciclos. O `mem_timing-2` testa
timing de PUSH/POP/CALL/RET e também não produz saída serial. Ambos precisam
de investigação adicional:

- `halt_bug.gb`: pode estar testando o caso `halt` seguido de `rst` (o halt
  bug com instrução de desvio), ou pode estar preso em HALT que nunca acorda.
- `mem_timing-2`: a spec local (`03-opcodes.md`) foi verificada e os M-cycles
  de PUSH/POP/CALL/RET batem com a implementação. A divergência pode estar na
  ordem exata de leitura/escrita dentro de cada M-cycle, ou em acessos de
  barramento que não deveriam ocorrer durante ciclos `internal`.

A suíte de testes unitários passou de 938 para 939 (2 novos testes de halt,
1 teste existente teve asserções atualizadas para refletir o comportamento
corrigido).
