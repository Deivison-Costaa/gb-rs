# Iteração 0051 — Interrupções: IE/IF/IME, dispatch, EI com delay

- **Data:** 2026-07-26
- **Item do roadmap:** 2.2

## Objetivo

Implementar o controlador de interrupções: IE ($FFFF), IF ($FF0F), IME, os 5 vetores ($0040/$0048/$0050/$0058/$0060), o dispatch de 5 M-cycles com prioridade fixa (VBlank > LCD > Timer > Serial > Joypad), e o delay de 1 instrução do `EI` e do `RETI`.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | Interrupts, Interrupt Sources, halt | `docs/reference/05-interrupts.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | O dispatch de interrupção leva 4 M-cycles (esqueci os 2 wait states). | A spec diz 5 M-cycles: 2 wait states + 2 push cycles + 1 jump cycle (§ Interrupt handling). | Teste `interrupt_dispatch_takes_five_m_cycles` (contraste: com 4 estados a suíte teria 650 testes e eu não teria notado; o teste escreveu o número 5 e o debugger confirmou a contagem). |
| 2 | timing | O `EI` seta `IME = 1` imediatamente, como eu faria de memória (é assim em Z80). | O efeito do `EI` é adiado em 1 instrução: `IME` só se torna `1` depois que a instrução seguinte ao `EI` termina (§ IME). | Teste `ei_does_not_set_ime_until_after_next_instruction`. O teste `ei_sets_ime` antigo (cpu_misc.rs) quebrou e precisou ser reescrito — foi o primeiro sinal de que a mudança tinha consequências em cascata. |
| 3 | flags/API | `RETI` habilita IME imediatamente (copiei o comportamento antigo do EI). | `RETI` é "same as `ei` immediately followed by `ret`" — também tem delay de 1 instrução (§ IME). | Teste `reti_enables_interrupts_with_delay` + quebra dos testes existentes `reti_behaves_like_ret_and_sets_ime` e `reti_takes_four_m_cycles` (cpu_ret.rs), que tiveram que ser atualizados. |
| 4 | endereçamento | O teste `di_clears_pending_ei` validava `IME == 0` após o DI, mas um interrupt disparado pelo EI tardio também zera `IME` — o teste passava por acidente. | O teste não media o cancelamento, media só o valor final de IME. | Bateria de mutação M2: sobreviveu (!). O teste foi reescrito para medir IF (não deve ter sido limpo por dispatch) e o mutante passou a ser pego. |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| blargg cpu_instrs | 10/12 | 10/12 |
| blargg instr_timing | 0/1 | 0/1 |
| blargg mem_timing | 0/4 | 0/4 |
| blargg mem_timing-2 | 0/4 | 0/4 |
| blargg halt_bug | 0/1 | 0/1 |
| blargg oam_bug | 0/9 | 0/9 |
| blargg interrupt_time | 0/1 | 0/1 |
| blargg dmg_sound | 0/13 | 0/13 |
| dmg-acid2 | 0/1 | 0/1 |
| mooneye acceptance | 0/66 | 0/66 |
| mooneye acceptance (outros modelos) | 0/9 | 0/9 |

Testes do workspace: 633 (eram 650 na 0050 — 19 novos em cpu_interrupts.rs, reescrita de testes EI/DI em cpu_misc.rs, atualização de testes RETI em cpu_ret.rs).

> O placar de ROMs não mudou: sem PPU, nenhuma ROM chega a exercitar o dispatch. O timer seta IF bit 2 e a CPU agora consegue servi-lo, mas a PPU não existe (modos, VBlank, STAT), então `cpu_instrs.gb` continua travando nos sub-testes de interrupção que dependem de VBlank.

## Revisão cruzada (segundo modelo)

- **Modelo:** não disponível nesta iteração
- **Achados:** —
- **Procedentes:** —
- **Falso positivo mais interessante:** —

## Decisões de arquitetura

1. **`ei_pending` + `ei_wait` como dois bools.** Um `enum` de três estados seria mais expressivo (Idle → EiExecuted → Pending → Applied) e eliminaria todos os casos de "estado combinatório ilegal" (`ei_pending=false, ei_wait=true`). Optei por dois bools porque a lógica não é grande e o compilador não reclama; se o HALT (2.3) adicionar mais estados de "caminho de wake-up", o enum vira melhor.

2. **`Wait1` implícito na detecção.** O primeiro dos 5 M-cycles de dispatch é consumido pela detecção da interrupção (no mesmo `step()` que substituiria o `fetch`). O estado `InterruptDispatch` começa em `Wait2`, não em `Wait1`. Isso evita que a detecção + 5 M-cycles do dispatch somem 6 chamadas a `step()`, quando a spec promete 5.

3. **IE e IF acessados via `bus.read()`/`bus.write()` na lógica de `check_interrupt`.** Diferente do timer (que escreve `self.io[IF_IDX] |= 0x04` direto), o CPU lê e escreve IF via endereço. Isso mantém a interface do barramento como única via de acesso à memória para o CPU, mesmo que os dois estejam no mesmo crate. O read-modify-write do IF (ler byte, limpar bit, escrever de volta) tem potencial de perder bits escritos por periféricos entre a leitura e a escrita; com timer executando ANTES do interrupt check em cada `step()`, isso não é problema hoje. Com PPU (M3) que avança junto no mesmo `step()`, é preciso garantir que a PPU escreva em IF ANTES do `check_interrupt`.

4. **Interrupção suprimida no `step` seguinte ao `EI`.** `apply_ei_delay` devolve `true` quando consome `ei_wait`, e o `step()` salta `check_interrupt` naquele ciclo. Isso garante que a instrução após `EI` sempre execute sem dispatch — mesmo que `IME` já estivesse `1` antes do `EI` (edge case: `EI` com `IME=1` e interrupção pendente).

## Notas

- **Suíte de 19 testes.** IE/IF leitura/escrita (4), dispatch básico (3 — vetores, push, M-cycles), controle negativo (3 — IME=0, IE=0, IF=0), prioridade (1), EI com delay (3), DI cancela (1, reescrito na bateria), RETI com delay (1). Três testes existentes foram atualizados: `ei_sets_ime` e `ei_does_not_change_flags_or_registers_except_pc_and_ime` (cpu_misc.rs), `reti_behaves_like_ret_and_sets_ime` e `reti_takes_four_m_cycles` (cpu_ret.rs).

- **Bateria de mutação: 7/7 pegos, 2/2 controles verdes.** O M2 (DI não limpa pending EI) sobreviveu na primeira rodada porque o teste `di_clears_pending_ei` media `IME == 0` ao final, valor que um interrupt tardio também produz. O teste foi reescrito para medir IF (inalterado = dispatch não ocorreu), e o mutante foi pego.

- **O que ficou em aberto:** o `check_interrupt` faz read-modify-write de IF (lê, limpa bit, escreve). Se uma PPU futura escrever em IF entre a leitura e a escrita, o bit é perdido. A invariante nova (nota 53) documenta isso; o teste de regressão virá no M3.
