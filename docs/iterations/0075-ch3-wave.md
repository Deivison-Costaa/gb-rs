# Iteração 0075 — Canal 3: wave RAM

- **Data:** 2026-07-27
- **Item do roadmap:** 6.4

## Objetivo

Máquina de estados do Canal 3 (wave): `Channel3` com period counter (2097152 Hz, 2× pulse), varredura da wave RAM (16 bytes × 2 nibbles, sample 0 pulado), acesso bloqueado durante playback, DAC via NR30, sem envelope.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | § Sound Channel 3 — Wave output | `docs/reference/07-apu.md` |
| Pan Docs | § Wave channel (CH3) | `docs/reference/07-apu.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | Wave RAM cairia naturalmente no `apu.read/write` como as outras regiões APU | O `Bus` não roteava `0xFF30..=0xFF3F` para a APU — caía no catch-all `_` do `IoRegisters` e retornava `OPEN_BUS`/engolia escrita. A tabela `HARDWARE_REGISTERS` do `boot.rs` também não listava esse range. | 2 testes de acesso à wave RAM inativa falharam (retornavam $FF em vez do valor escrito). O teste `the_addresses_the_table_never_mentions_have_no_owner_yet` também quebrou (contava 72 sem dono; agora são 56). |
| 2 | timing | A wave RAM seria lida/escrita normalmente com CH3 ativo, como as outras regiões (VRAM/OAM bloqueiam, mas channel registers, não) | A spec diz: "On monochrome consoles, wave RAM can only be accessed on the same cycle that CH3 does. Otherwise, reads return $FF, and writes are ignored." | Teste `wave_ram_retorna_ff_na_leitura_quando_ch3_ativo` e `wave_ram_escrita_ignorada_quando_ch3_ativo` |
| 3 | API-Rust | O teste `wave_ram_escrita_ignorada_quando_ch3_ativo` via leitura $FF provaria que a escrita foi engolida | Ler $FF durante playback prova bloqueio de *leitura*, não de escrita. Para provar que a escrita foi engolida, é preciso desabilitar o canal (NR30=0) e ler de volta o valor original. | Mutação M4 sobreviveu (removeu o `if` da escrita e nenhum teste quebrou). Corrigi o teste para desligar o canal e ler o valor preservado. |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| blargg cpu_instrs | 12/12 | 12/12 |
| blargg instr_timing | 1/1 | 1/1 |
| blargg mem_timing | 4/4 | 4/4 |
| blargg mem_timing-2 | 0/4 | 0/4 |
| blargg halt_bug | 0/1 | 0/1 |
| blargg oam_bug | 0/9 | 0/9 |
| blargg interrupt_time | 0/1 | 0/1 |
| blargg dmg_sound | 0/13 | 0/13 |
| dmg-acid2 | 1/1 | 1/1 |
| mooneye acceptance | 0/66 | 0/66 |
| mooneye acceptance (outros) | 0/9 | 0/9 |

Testes do workspace: **946** (eram 940 na 0074 — 17 novos em `ch3_wave.rs`, menos ajustes em `bus_boot_state.rs`).

## Bateria de mutação

| # | Mutação | Pego? | Quem pegou |
|---|---|---|---|
| M1 | `trigger_ch3` sem checagem `nr30 & 0x80` | Sim | `trigger_com_dac_desligado_nao_liga_o_canal` |
| M2 | `sample_index = 0` em vez de `1` | Sim | 5 testes: `trigger_define_sample_index_em_1_nao_em_0`, `ch3_le_sample_da_wave_ram_no_overflow`, `ch3_le_nibble_alto_no_sample_par`, `sample_index_avanca_no_overflow_do_freq_timer`, `sample_index_volta_ao_zero_ao_completar_32_samples` |
| M3 | `wrapping_add(4)` em vez de `8` | Sim | 5 testes: `freq_timer_do_ch3_avanca_8_por_m_cycle_e_sofre_overflow`, `sample_index_avanca`, `sample_index_volta`, `ch3_le_sample`, `ch3_le_nibble_alto` |
| M4 | Wave RAM sempre escreve (sem guarda `!ch3.enabled`) | Sim (após corrigir teste) | `wave_ram_escrita_ignorada_quando_ch3_ativo` |
| C1 | `Channel3::new` com `sample_index: 0` (trigger seta para 1) | Controle verde | 17/17 passam |

**4/4 pegos, 1/1 controles verdes.**

## Decisões de arquitetura

- `Channel3` é struct próprio (não reusa `PulseChannel`) porque o contador de frequência avança 8 por M-cycle (vs 4), o "duty step" equivalente tem 32 posições (vs 8) com leitura de nibble da wave RAM (vs padrão fixo), e não há envelope.
- Wave RAM (`0xFF30..=0xFF3F`) passa a ser roteada explicitamente pelo `Bus::read/write` para `apu.read/write`, em vez de cair no catch-all que retornava `OPEN_BUS`.
- `NR30 bit 7 = 0` desliga o canal imediatamente (`ch3.enabled = false`), sem esperar trigger ou frame sequencer — mesmo comportamento do DAC off dos outros canais.
- O last sample buffer não é limpo no trigger (spec: "retriggering the channel does not clear nor refresh this buffer").

## Notas

- O roteamento `0xFF30..=0xFF3F` no `Bus` destravou 16 endereços que antes eram `OPEN_BUS` — a partição de endereços sem dono do `bus_boot_state.rs` caiu de 72 para 56.
- A correção do teste `wave_ram_escrita_ignorada_quando_ch3_ativo` foi o achado real da bateria de mutação: o teste original testava bloqueio de leitura, não de escrita.
- O contador de frequência do CH3 avança 8 por M-cycle (2097152 Hz = 2 dots por tick, vs 4 dots dos canais de pulso). O cálculo confere: 96 M-cycles × 8 = 768 + 1280 ($0500) = 2048 = overflow.
