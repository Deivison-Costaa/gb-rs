# Iteração 0064 — Joypad (P1/JOYP + interrupção)

- **Data:** 2026-07-27
- **Item do roadmap:** 4.1

## Objetivo

Implementar o joypad: registrador P1 ($FF00) com seleção de grupo (buttons/d-pad) ativa-baixa, retorno de estado nos bits 3-0 e interrupção de joypad (IF bit 4, vetor $0060).

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | Joypad Input (FF00 — P1/JOYP) | `docs/reference/09-joypad-serial.md` |
| Pan Docs | Interrupts (IE/IF bit 4) | `docs/reference/05-interrupts.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | As constantes de teste `SELECT_BUTTONS` e `SELECT_DPAD` foram escritas com os valores trocados (0x20 e 0x10 em vez de 0x10 e 0x20) — supus que o bit de seleção é ativo-alto, como o bit 7 de SC. | A spec diz "*If this bit is 0, then buttons can be read from the lower nibble*": 0 = selecionado, 1 = não selecionado. Os bits 5-4 são active-low, não active-high. | 12 passaram, 9 falharam nos testes de pressão de botão — todos os `press_*` liam $F em vez do padrão esperado porque escreviam o grupo errado. |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 12/12 | 12/12 |
| dmg_sound | 0/13 | 0/13 |
| halt_bug | 0/1 | 0/1 |
| instr_timing | 1/1 | 1/1 |
| interrupt_time | 0/1 | 0/1 |
| mem_timing | 4/4 | 4/4 |
| mem_timing-2 | 0/4 | 0/4 |
| oam_bug | 0/9 | 0/9 |
| dmg-acid2 | 1/1 | 1/1 |
| mooneye/acceptance | 0/66 | 0/66 |
| mooneye/acceptance-nondmg | 0/9 | 0/9 |
| **TOTAL** | **18/121** | **18/121** |

Testes do workspace: 740 → 761 (21 novos em `input_joypad.rs`).

## Bateria de mutação

**5/5 pegos, 1/1 controle verde.**

| # | Mutação | Resultado |
|---|---|---|
| 1 | Inverter lógica de seleção (`== 0` → `!= 0`) | 11/21 falham |
| 2 | key_down não altera estado dos botões (match sem corpo) | 11/21 falham |
| 3 | key_down não seta `self.interrupt = true` | 2/21 falham |
| 4 | write ignora valor (`self.select = 0x30` fixo) | 11/21 falham |
| 5 | AND → OR no cálculo do nibble baixo | 11/21 falham |
| C1 | Valor inicial de dpad/buttons = 0xFF (nibble baixo igual) | 21/21 passam |

## Decisões de arquitetura

- **`Joypad` é struct com `select: u8` separado de `dpad: u8` e `buttons: u8`.** A leitura de P1 compõe o valor dos três campos: bits 7-6 = 1, bits 5-4 = select, bits 3-0 = AND dos nibbles dos grupos selecionados. Alternativa de armazenar `p1: u8` único foi descartada porque o write só afeta bits 5-4 e o read recompõe bits 3-0 a cada chamada — separar evita o problema de "bits 3-0 de P1 envelhecerem entre seleções".
- **Interrupção é gerada no `key_down`, propagada para IF em `tick_joypad_interrupt()` chamado de `Cpu::step`.** Segue o padrão da PPU (sinal → IF em `tick_*`) e não o do timer (que escreve IF diretamente no `tick_timer`). O `joypad.interrupt` é `pub(crate)` e o `Bus::tick_joypad_interrupt` o consome, setando `IF |= 0x10`.
- **`Key` é enum público com 8 variantes exportadas por `gb-core`.** O `gb-desktop` vai mapear winit keycodes para esses valores. O `gb-cli` pode expor keys via argumento ou stdin no futuro.

## Notas

- O handoff mencionava reavaliação do 2.4b (halt_bug + mem_timing-2). O placar continua 0/4 e 0/1 — as ROMs seguem rodando até o teto de ciclos sem veredito. O M3 fechado não destravou essas suítes.
- O `check_interrupt` em `mcycle.rs:612-613` faz RMW de IF (lê, limpa bit, escreve). O `tick_joypad_interrupt` escreve `IF |= 0x10` diretamente, sem ler — portanto não há janela de perda entre o check_interrupt e a escrita do joypad, desde que `tick_joypad_interrupt` rode antes de `check_interrupt`. A ordem atual em `step()` é `tick_timer → tick_ppu → tick_joypad_interrupt → check_interrupt`. Se a PPU e o joypad dispararem no mesmo M-cycle, `check_interrupt` vê ambos os bits. OK.
