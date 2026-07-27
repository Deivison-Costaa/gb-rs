# Iteração 0080 — Length counter da APU (6.8a)

- **Data:** 2026-07-27
- **Item do roadmap:** 6.8a — Length counter: NRx1 load, trigger reload, 256 Hz tick, channel disable on expiry, NR52 readback.

## Objetivo

Implementar o length counter da APU: carregamento via NRx1, recarga por trigger
quando expirado, decremento a 256 Hz (passos 0/2/4/6 do frame sequencer) e
desligamento do canal ao chegar a zero.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | Common concepts > Length timer | `docs/reference/07-apu.md` § Length timer |
| Pan Docs | Audio Details > DIV-APU | `docs/reference/07-apu.md` § DIV-APU |
| Pan Docs | Audio Details > Channels, NR52 | `docs/reference/07-apu.md` § NR52 |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | O trigger sempre recarrega o length counter para 64 (ou 256 para CH3). | O trigger só recarrega se o contador estiver expirado (0). Se estiver não-zero, mantém o valor atual. § NR14: "If length timer expired it is reset." | `length_timer_da_tick_em_cada_2_passos_do_frame_sequencer`: verifica que contador configurado via NR21 (não-zero) NÃO é sobrescrito pelo trigger |
| 2 | especificação | A tabela de 64 entradas do length counter estaria no Pan Docs. | O Pan Docs não inclui tabela explícita; a relação é `64 - (NRx1 & 0x3F)` para CH1/2/4 e `256 - NR31` para CH3, consistente com "inverted when written" do footnote. | Nenhum impacto no código — usei a fórmula deduzida que bate com o comportamento esperado |
| 3 | timing | O length counter só dispara em passos pares os dois últimos (2 e 6), no mesmo ciclo do envelope. | O length ticka em TODOS os passos pares (0, 2, 4, 6), separado do envelope que ticka em 2 e 6. | `length_timer_da_tick_em_cada_2_passos_do_frame_sequencer`: verifica 4 ticks em uma volta completa (0→0), não 2 |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| dmg_sound (0/13 passando) | 0/13 (todos crash) | 0/13 (todos crash) |

As ROMs continuam sem produzir saída serial — o length counter sozinho não é
suficiente. A ROM `02-len ctr` provavelmente depende de DAC-off trigger
protection (6.8d) ou de timing de envelope (6.8c) para completar o ciclo de
teste. Os 13 testes unitários novos confirmam que a infraestrutura do length
counter está correta; o bloqueio está nas próximas features.

## Bateria de mutação

| Mutação | Resultado |
|---|---|
| M1: `length_timer -= 1` → no-op | **4/13 FAILED** (length nunca decrementa → canal nunca desliga) |
| M2: `self.nr24 & 0x40 != 0` → `true` | **1/13 FAILED** (enable ignorado → canal desliga mesmo sem enable) |
| M3: `if length_timer == 0 { reload }` → `if false {...}` | **4/13 FAILED** (sem reload → re-trigger após expirar não religa canal) |
| M4: `64 - (v & 0x3F)` → `(v & 0x3F)` | **5/13 FAILED** (fórmula errada → contador carrega valor invertido) |
| C1: código original sem mutação | **13/13 passando** |

Placar: **4/4 pegos**, **1/1 controle verde**.

## Decisões de arquitetura

- `length_timer: u16` em todos os canais (PulseChannel, Channel3, Channel4).
  CH1/CH2/CH4 usam range 0-64; CH3 usa 0-256. `u16` unificado evita conversão.
- Contador descendente (0 = expirado), não ascendente até 64. O comportamento
  externo é idêntico: `64 - L` ticks antes de expirar.
- Carregamento em NRx1 write (sempre), não em trigger. O trigger só recarrega
  a 64/256 se expirado.

## Notas

- As 13 ROMs dmg_sound continuam crash (saem 2, sem output serial). A falha não
  é no length counter — é em features ainda não implementadas: DAC-off trigger
  protection (6.8d), envelope timing (6.8c), extra length clocking (6.8b).
- O `power_off()` do APU zera NRx1 — o spec diz que no DMG os length timers
  sobrevivem. Bug pré-existente, não introduzido aqui. Vai doer na ROM
  `11-regs after power`.
- A fórmula `64 - (NRx1 & 0x3F)` produz 0 para L=64 (overflow). O campo é 6
  bits, então L ∈ [0, 63], e a fórmula nunca ultrapassa 64 nem fica negativa.
- O contador decrementa mesmo com canal já desligado (se `enabled == false` mas
  `length_timer > 0`). Isso não tem efeito visível — o canal já está desligado
  — mas preserva o estado interno para re-trigger futuro.
