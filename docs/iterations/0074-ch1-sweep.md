# Iteração 0074 — CH1 sweep: canal 1 + unidade de sweep de frequência

- **Data:** 2026-07-27
- **Item do roadmap:** 6.3

## Objetivo

Implementar o Canal 1 de áudio (square + sweep de frequência), generalizando a
lógica comum de pulso entre CH1 e CH2 extraída do `Channel2` da iteração 0073
para uma `PulseChannel` compartilhada.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | § Sound Channel 1 — Pulse with period sweep | `docs/reference/07-apu.md` |
| Pan Docs | § Pulse channel with sweep (CH1) | `docs/reference/07-apu.md` |
| Pan Docs | § Audio details — DIV-APU | `docs/reference/07-apu.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | valores | Usei `NR10=0x73` esperando direction=1 (subtract), mas `0x73` tem bit 3 = 0 (addition). Erro de leitura binária: 0b0111_0011 vs 0b0111_1011. | Bit 3 = 1 é subtraction | Teste `nr10_configura_pace_direction_e_step` falhou: direction=0, esperado 1 |
| 2 | valores | No teste de overflow usei `NR10=0x10` (pace=1, direction=0, step=0). Assumi que o overflow check do trigger roda mesmo com step=0. | O cálculo imediato e overflow check só executam se `step != 0` (§ Pulse channel with sweep, trigger event) | Teste `sweep_overflow_acima_de_2047_desliga_canal` falhou: canal continuava ligado |
| 3 | valores | Usei pace=7 no teste de iteração, esperando que 14 frame sequencer steps fossem suficientes. Aritmética correta, mas teste ficou ilegível e frágil. | pace=1 simplifica: uma iteração a cada 2 steps (passo 2 ou 6) | Teste `sweep_iteracao_escreve_novo_periodo_de_volta_em_nr13_nr14` falhou com valores errados de direção. Refatorado para pace=1 |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 12/12 | 12/12 |
| dmg_sound | 0/13 | 0/13 |
| Mooneye acceptance | 0/66 | 0/66 |
| dmg-acid2 | 1/1 | 1/1 |
| Testes unitários | 921 | 940 |

Nenhuma regressão de placar. 19 testes novos em `ch1_sweep.rs`.

## Bateria de mutação

| # | Mutação | Alvo | Resultado |
|---|---|---|---|
| M1 | Inverter direção do sweep (`if direction == 0` → `if direction == 1`) | `sweep_calculate` | **Pego**: 5 testes falharam |
| M2 | Remover cálculo imediato do sweep no trigger | `trigger_ch1_sweep` | **Pego**: 5 testes falharam |
| M3 | Desabilitar iteração do sweep (`if false && ...`) | `tick_ch1_sweep` | **Pego**: 1 teste falhou |
| C1 | Nenhuma mutação | — | **Verde**: 19/19 |
| C2 | Nenhuma mutação (redundante com C1) | — | **Verde**: 19/19 |

**3/3 pegos, 2/2 controles verdes.**

## Decisões de arquitetura

- `Channel2` foi renomeado para `PulseChannel`: struct com `enabled`, `freq_timer`,
  `duty_step`, `envelope_volume`, `envelope_timer`. Métodos `trigger`, `tick_freq`
  e `tick_envelope` recebem `nrx2/nrx3/nrx4` como parâmetros genéricos, funcionando
  tanto para CH1 (NR12/NR13/NR14) quanto para CH2 (NR22/NR23/NR24).
- `Channel1` é uma struct nova que encapsula `pulse: PulseChannel` + campos do
  sweep: `sweep_shadow`, `sweep_timer`, `sweep_enabled`.
- O sweep é clockado nos passos 2 e 6 do frame sequencer (128 Hz), junto com o
  envelope, via `tick_ch1_sweep()`.
- O cálculo imediato no trigger (quando step != 0) está em `trigger_ch1_sweep()`,
  chamado de `write(NR14)` quando o bit 7 está setado.
- O sweep escreve de volta nos registradores NR13/NR14, e o segundo cálculo
  (sem write-back) verifica overflow adicional.

## Notas

- A refatoração `Channel2 → PulseChannel` foi mecânica: renomear, ajustar
  parâmetros de `nr22/nr23/nr24` para `nrx2/nrx3/nrx4`, atualizar todos os
  callers. Zero regressões nos 14 testes do CH2.
- O sweep não tem saída de áudio — só a máquina de estados. Mesma decisão da 0073.
- A `write(NR14)` com trigger primeiro chama `pulse.trigger()` e depois
  `trigger_ch1_sweep()`. A ordem importa: `trigger_ch1_sweep` lê `self.nr13`/`self.nr14`
  para montar o shadow register.
- O `sweep_timer` trata pace=0 como 8 no reload, seguindo a nota de
  "obscure behavior" do Pan Docs: "volume envelope and sweep timers treat a
  period of 0 as 8".
