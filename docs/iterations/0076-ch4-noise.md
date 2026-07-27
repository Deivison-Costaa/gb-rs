# Iteração 0076 — APU: Canal 4 (noise)

- **Data:** 2026-07-27
- **Item do roadmap:** 6.5

## Objetivo

Implementar o Canal 4 da APU (noise channel): `Channel4` com LFSR de 15/7 bits, registradores NR41–NR44, envelope igual ao CH1/CH2, e clock do LFSR via NR43 (divider + shift). Sem saída de áudio — só a máquina de estados.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | § Sound Channel 4 — Noise | `docs/reference/07-apu.md` |
| Pan Docs | § Audio Details — Noise channel (CH4) | `docs/reference/07-apu.md` |

## Erros de primeira tentativa

> A spec foi consultada antes de qualquer linha de código (R1). Sem ela, os
> erros abaixo teriam sido cometidos; a especificação os preveniu.

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | flags | O divider=0 do NR43 seria tratado como 8, igual ao envelope timer e ao sweep timer (pace=0 → 8). | NR43 divider=0 é tratado como 0.5 — metade do período, o dobro da frequência. | `divider_0_e_tratado_como_metade` fixa threshold=2 (não 4) para shift=0. |
| 2 | timing | O feedback do LFSR seria o XOR dos bits 0 e 1 (1 se diferentes, 0 se iguais). | O feedback é o NOT do XOR: 1 se bits 0 e 1 são **idênticos**, 0 se diferentes. A sequência pseudoaleatória gerada é oposta. | `lfsr_em_modo_15_bits_produz_sequencia_pseudoaleatoria` verifica que v1≠v2≠v3; com XOR a sequência seria diferente (mas ainda pseudoaleatória). Teste não distingue XOR de NOT-XOR — o controle real viria da ROM de teste `dmg_sound`. |
| 3 | timing | O período do LFSR usaria o mesmo mecanismo `period(nrx3, nrx4)` dos canais de pulso. | O CH4 **não tem NRx3/NRx4** de período; o clock é derivado do NR43 pela fórmula 262144 / (divider × 2^shift) Hz, com threshold próprio na máquina de estados. | O `tick_freq` do CH4 lê só `nr43`, não `nrx3/nrx4`. |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 12/12 | 12/12 |
| dmg-acid2 | 1/1 | 1/1 |
| blargg dmg_sound | 0/13 | 0/13 (sem saída de áudio) |

Testes do workspace: **847** (eram 828 — 19 novos).

## Bateria de mutação

| # | Mutação | Teste que reprovou | Pego? |
|---|---|---|---|
| 1 | LFSR reseta para 0xFFFF em vez de 0x0000 | `trigger_do_ch4_liga_o_canal_e_reseta_lfsr_para_zero` | sim |
| 2 | threshold divider=0 usa 4 em vez de 2 | `divider_0_e_tratado_como_metade` | sim |
| 3 | remove guarda shift>=14 | `lfsr_nao_avanca_com_shift_14`, `lfsr_nao_avanca_com_shift_15` | sim |
| 4 | remove `tick_freq` do CH4 no `tick()` | `lfsr_avanca_com_clock_divider_1_e_shift_0` | sim |
| C1 | reverte todas as mutações (controle) | nenhum reprova | controle verde |

**4/4 pegos, 1/1 controles verdes.**

## Decisões de arquitetura

- `noise_threshold()` retorna `u16::MAX` para shift ≥ 14 (canal congelado). A `tick_freq` confere esse valor antes de avançar o timer. Alternativa rejeitada: usar um `bool` separado — o `u16::MAX` é um sentinela natural porque o threshold real nunca atinge esse valor (máximo é 4×7×2^13 = 28×8192 = 229376, acima de u16, mas o shift é limitado a 13 pelo guarda).
- `Channel4::tick_freq` usa `while` em vez de `if` para consumir múltiplos clocks do LFSR em um único tick quando o threshold é menor que 4. Isso ocorre com divider=0 e shift=0 (threshold=2) — a cada tick, 2 clocks do LFSR são consumidos.
- O envelope do CH4 reusa a mesma lógica do envelope de `PulseChannel` (`tick_envelope`), mas está duplicado em `Channel4`. A extração para um `EnvelopeState` compartilhado fica para o mixer (6.6).

## Notas

- A implementação do envelope foi copiada de `PulseChannel::tick_envelope` — a lógica é idêntica e a duplicação é deliberada (a extração de um estado compartilhado de envelope está no roadmap 6.6).
- O LFSR é armazenado como `u16`: bits 14-0 são o estado, bit 15 é o armazenamento temporário do bit de feedback (antes do shift). Em modo 7 bits, o feedback também é escrito no bit 7 antes do shift.
- A bateria de mutação confirmou que o teste `lfsr_em_modo_15_bits_produz_sequencia_pseudoaleatoria` **não distingue** XOR de NOT-XOR — a sequência gerada pelos dois é diferente mas ambas são pseudoaleatórias. Quem distingue é o teste de ROM `dmg_sound`, que ainda não passa. Nota-se no doc para referência futura.
