# Iteração 0077 — Mixer: NR50/NR51/NR52, panning, DAC enable

- **Data:** 2026-07-27
- **Item do roadmap:** 6.6

## Objetivo

Mixer que combina as saídas digitais dos 4 canais via NR51 (panning) e escala com NR50 (master volume). NR52 com bits de status dinâmicos e power-off que zera registradores.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | Audio Registers (NR50/NR51/NR52) | `docs/reference/07-apu.md` § Global control registers |
| Pan Docs | DACs | `docs/reference/07-apu.md` § DACs |
| Pan Docs | Mixer | `docs/reference/07-apu.md` § Mixer |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | flags | NR50 volume 0 significa mute (fator de escala 0) | A spec diz que o amplificador **nunca muta**: volume 0 = escala 1 (muito baixo), volume 7 = escala 8 (sem redução). A fórmula é `(volume_field + 1)`, não `volume_field`. | Teste `nr50_volume_0_escala_1_nao_muta` — sem o +1, sairia 0 e o teste falharia pelo assert `left > 0`. |
| 2 | flags | Os bits 3–0 do NR52 aceitam escrita para ligar/desligar canais | A spec é explícita: "Writing to those does **not** enable or disable the channels, despite many emulators behaving as if". São read-only. | Teste `escrita_nos_bits_0_a_3_do_nr52_nao_liga_canais` — escrever 0x8F não liga canal nenhum. |
| 3 | timing | O DAC check em `ch1_digital_output` seria independentemente testável via mixer_sample | DAC off (NRx2 & $F8 == 0) implica volume = 0, então a saída já é 0 sem o check. Para CH3, DAC off impede o trigger, então o canal nunca liga. O check em `chX_digital_output` é redundante com o estado do canal/volume — não tem como ser exercitado pelo mixer. | Bateria de mutação: remover `!self.ch1_dac_enabled()` de `ch1_digital_output` sobreviveu a todos os testes. |

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
| mooneye acceptance | 0/66 | 0/66 |
| mooneye nondmg | 0/9 | 0/9 |

Testes do workspace: **871** (eram 847 na 0076 — 24 novos em `apu_mixer.rs`).

## Bateria de mutação

| Mutação | Alvo | Pegou? | Quem pegou |
|---|---|---|---|
| M1: NR50 volume sem +1 (usa índice direto) | `left_vol = (nr50 >> 4) & 7` | sim (2/2) | `nr50_volume_0_escala_1_nao_muta`, `nr50_volume_7_escala_8_sem_reducao` |
| M2: CH1-left roteado para variável `right` | `if nr51 & 0x10 { right += ch1 }` | sim (1/1) | `nr51_ch1_apenas_na_esquerda` |
| M3: Remove DAC check de `ch1_digital_output` | `if !self.ch1.pulse.enabled` (sem DAC) | **não** | Nenhum — DAC off implica volume=0, saída já é 0. Inobservável via mixer. |
| M3b: Remove DAC check de `ch3_digital_output` | `if !self.ch3.enabled` (sem DAC) | **não** | Nenhum — DAC off do CH3 impede trigger, canal nunca liga. |
| M4: Waveform 50% usa padrão de 75% | `DUTY_WAVEFORMS[2] = 0b01111110` | sim (1/1) | `ch1_duty_50_pct_com_envelope_15_produz_saida_15` |
| Controle: `cargo test --all` verde com código original | — | verde | — |

**Placar: 4/4 pegos, 2 sobreviventes analisados (DAC check redundante), 1/1 controles verdes.**

## Decisões de arquitetura

- **`mixer_sample()` retorna `(u16, u16)` de soma digital pós-NR51/NR50, não amostra de áudio.** A saída é a soma dos valores digitais (0–15) de cada canal, com panning e master volume aplicados. O downsample para 48 kHz e a conversão analógica (DAC, HPF) vêm no 6.7.
- **Digital output de pulso usa tabela de waveforms de 8 bits indexada por `duty_step`.** Os 4 padrões são `[0b00000001, 0b10000001, 0b10000111, 0b01111110]`, onde o LSB corresponde a duty_step=0. Verificado contra o SVG do Pan Docs e contra valores canônicos da comunidade.
- **NR52 bits 3–0 são dinâmicos**, computados de `chX.enabled` a cada leitura. Isso muda o valor de boot de $F1 para $F0 — a 0077 é a primeira iteração em que o NR52 lido reflete o estado real dos canais, e o bit 0 ($F1 = CH1 on) era um artefato de copiar a tabela sem inicializar o canal. Testes atualizados (bus_boot_state, apu_frame_sequencer) para $F0.
- **Power-off (`NR52 bit 7 = 0`) zera registradores e recria canais**, exceto wave RAM e DIV-APU. Só dispara na transição powered→unpowered (não reaplica se já estava off).

## Notas

- **Achado da bateria:** O DAC check nas funções `ch1_digital_output`/`ch2_digital_output`/`ch4_digital_output` é inobservável pelo mixer. Para CH1/CH2/CH4, `NRx2 & $F8 == 0` implica volume inicial 0, e a saída já seria 0 sem o check. Para CH3, DAC off impede o trigger, então o canal permanece desligado. O check é correto do ponto de vista de hardware mas não tem cobertura de teste via `mixer_sample`. Se um dia o envelope puder aumentar o volume a partir de 0 com DAC off, o check passaria a ser necessário — mas o trigger também verifica DAC.
- **Perda das edições na bateria:** O `git checkout -- crates/gb-core/src/apu.rs` do M1 reverteu todas as edições da iteração (não só a mutação). As 8 edições tiveram de ser reaplicadas. Lição: bateria com `sed -i` + `git checkout` destrói trabalho não commitado; usar `git stash` ou backup explícito resolve.
- **CH3 output level 0 → mute é testado diretamente.** Ao contrário do DAC check dos canais de pulso, o output level 0 do CH3 (NR32 bits 5-6 = 00) é independentemente testável porque o volume da wave RAM pode ser não-zero e o mute vem do level, não do volume.
- O handoff do STATUS.md dizia "Os 4 canais já produzem output digital (volume)" — não produziam. A 0077 adicionou `digital_output()` em todos os 4 canais como pré-requisito do mixer.
