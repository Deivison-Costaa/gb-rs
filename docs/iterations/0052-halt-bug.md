# Iteracao 0052 — HALT + o bug do HALT

- **Data:** 2026-07-26
- **Item do roadmap:** 2.3

## Objetivo

Implementar HALT ($76) — pausar a CPU e acordar quando IE & IF != 0 — mais
o bug do HALT (IME=0 com IE & IF != 0 no momento do HALT, causando falha de
incremento do PC).

## Spec consultada

| Fonte | Secao | Arquivo local |
|---|---|---|
| Pan Docs | halt, halt bug | `docs/reference/05-interrupts.md` |
| gbops | Linha $76 (HALT) — 1 M-cycle | `docs/reference/03-opcodes.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | Eu teria implementado `halted` como um `State::Halted` separado, com logica de wake-up no match do estado — gerando atraso de 1 M-cycle entre IE&IF!=0 e o dispatch. | A spec diz que o wake-up e imediato: "the CPU simply wakes up, and before executing the instruction after the halt, the interrupt handler is called normally". Usar flag `halted: bool` + `State::Fetch` permite wake-up e check_interrupt no MESMO step(). | Raciocinio de design: o teste `halt_with_ime_1_wakes_and_dispatches_interrupt` conta 5 M-cycles do dispatch sem atraso adicional. |
| 2 | timing | Eu teria implementado o halt_bug suprimindo o incremento de PC **durante o proprio fetch do HALT** (read_at_pc nao incrementar), nao no fetch seguinte. | O PC ja foi incrementado por read_at_pc durante o fetch do HALT; o bug suprime o incremento do **proximo** fetch. O efeito observavel "the byte after the halt to be read a second time" so acontece se o primeiro fetch pos-HALT mantiver o PC estacionado. | Teste `halt_bug_does_not_halt_when_ime_0_and_pending` — PC fica em $0101 apos o primeiro fetch pos-HALT. Se a supressao fosse no fetch do HALT, PC seria $0100 ou $0102. |

## Bateria de mutacao

| # | Mutacao | Teste que pegou | Pego? |
|---|---|---|---|
| M1 | HALT como NOP (self.halted = true comentado) | halt_pauses_cpu_and_pc_does_not_advance | sim |
| M2 | Wake-up nunca limpa halted (self.halted = false comentado) | after_wake_up_cpu_is_no_longer_halted, halt_with_ime_1_wakes_and_dispatches_interrupt | sim |
| M3 | Bug nunca dispara (self.halt_bug = true → self.halted = true) | halt_bug_does_not_halt_when_ime_0_and_pending, halt_bug_byte_after_halt_executed_twice | sim |
| M4 | halt_bug nunca limpa em read_at_pc | halt_bug_byte_after_halt_executed_twice, halt_bug_does_not_halt_when_ime_0_and_pending | sim |
| M5 | Condicao do bug invertida (IME=1 dispara bug) | halt_bug_does_not_halt_when_ime_0_and_pending, halt_bug_byte_after_halt_executed_twice | sim |
| M6 | Wake-up requer IME (self.ime && ... != 0) | halt_wakes_up_when_ie_and_if_non_zero, halt_with_ime_0_wakes_and_resumes_normal_fetch | sim |
| M7 | read_at_pc decrementa PC com halt_bug (wrapping_sub) | halt_bug_byte_after_halt_executed_twice, halt_bug_does_not_halt_when_ime_0_and_pending | sim |
| C1 | Pre-computa pending em HALT (equivalente) | (todos verdes) | controle ok |
| C2 | Reestrutura wake-up (if pending == 0 return) | (todos verdes) | controle ok |

**Placar: 7/7 pegos, 2/2 controles verdes.**

> M2 e M4 sobreviveram na primeira rodada. M2: so halt_with_ime_1_wakes pegaria;
> halt_wakes_up_when_ie_and_if_non_zero era falso positivo (CPU caia pela condicao
> sem limpar halted, PC ainda avancava). Adicionado after_wake_up_cpu_is_no_longer_halted
> que zera IE/IF apos wake-up e verifica que a CPU nao re-halta. M4: so o teste de
> PC em halt_bug_does_not_halt pegaria; halt_bug_byte_after_halt_executed_twice
> nao checava PC — so B. Adicionada verificacao de PC nas duas assercoes.

## Placar

| Suite | Antes | Depois |
|---|---|---|
| cpu_instrs | 10/12 | 10/12 |
| halt_bug | 0/1 | 0/1 |

> halt_bug continua 0/1: o mecanismo esta implementado mas a ROM de teste
> (blargg) depende de timer e PPU. O teste real e em 2.4.

## Revisao cruzada (segundo modelo)

- **Modelo:** nao configurado (nota 5)
- **Achados:** —
- **Procedentes:** —
- **Falso positivo mais interessante:** —

## Decisoes de arquitetura

1. **Flag `halted: bool` no Cpu, nao um `State::Halted`.** O estado continua
   `State::Fetch` durante o halt, e o flag e checado no inicio de `step()` antes
   do dispatch de interrupcao. Isso evita atraso de 1 M-cycle entre wake-up e
   dispatch (erro #1). A invariante relevante e a do `Cpu::step` como M-cycle
   atomico — o halt e um internal da instrucao, nao uma mudanca de estado da
   maquina.

2. **`halt_bug: bool` no Cpu.** Suprime o incremento de PC em `read_at_pc` por
   exatamente um fetch. O mecanismo e: HALT detecta `!IME && IE & IF != 0` →
   `halt_bug = true`; no proximo `read_at_pc`, PC nao incrementa e
   `halt_bug = false`. O `halt_bug` e ignorado em todos os demais M-cycles.

3. **Wake-up nao zera IE/IF nem dispara dispatch por conta propria.** O bloco
   `if self.halted` apenas desliga o flag e cai para a logica normal de
   `check_interrupt` no topo do `step()`. Se `IME=0`, `check_interrupt` retorna
   `None` e a CPU continua fetch normal — a interrupcao fica pendente (IF bit
   setado) ate que `IME=1`.

## Notas

- O ultimo `UndecodedOpcode` do projeto foi eliminado: todos os 256 opcodes
  agora sao ou decodificados ou `IllegalOpcode` (os 11 da spec). O teste
  `an_opcode_this_emulator_has_not_reached_is_not_an_illegal_one` em
  `cpu_mcycle_loop.rs` foi reposto como `an_illegal_opcode_is_not_mistaken_for_undecoded_one`,
  testando `$D3` como `IllegalOpcode`.

- `decoded_elsewhere` em `tests/support/mod.rs` perdeu a exclusao explicita de
  `$76` da faixa `0x40..=0x7F`. A condicao passou a `(0x40..=0x7F).contains(&opcode)`,
  sem a clausula `&& opcode != 0x76`.

- O blargg `halt_bug.gb` depende de timer e PPU funcionais para seus testes;
  a ROM continua com `crash` ate que M2.4 avalie com o pipeline completo.
