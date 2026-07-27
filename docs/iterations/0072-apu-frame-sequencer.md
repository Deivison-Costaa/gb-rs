# Iteração 0072 — APU frame sequencer 512 Hz

- **Data:** 2026-07-27
- **Item do roadmap:** 6.1

## Objetivo

Estrutura do módulo APU com frame sequencer de 512 Hz, registradores de controle e acesso via Bus.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | Audio, Audio Registers, Audio Details (§ DIV-APU) | `docs/reference/07-apu.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | O range `$FF10`–`$FF3F` inteiro pertence à APU e pode ser despachado para `apu.read`/`apu.write` | O range tem lacunas (`$FF15`, `$FF27`–`$FF2F`) que não têm dono e devem ler `OPEN_BUS` / engolir escrita | Testes `bus_boot_state` e `bus_memory_map` panickaram com `unreachable!` no APU para `$FF15`. Corrigido restringindo o dispatch ao range `$FF10`–`$FF26` (registradores, sem wave RAM) |
| 2 | flags | NR52 bits 0–3 (status dos canais) podiam ser computados dinamicamente a partir do estado interno de cada canal | No boot hand-off, NR52 = `$F1` (bit 0 = CH1 ativo) é o valor deixado pela boot ROM. Como o emulador pula a boot ROM, NR52 deve armazenar e devolver o valor da tabela de boot diretamente — bits 0–3 são read-only, mas o valor de boot já os reflete | Teste `apu_registers_tem_os_valores_do_hand_off_da_boot_rom` esperava `$F1` e recebeu `$80` (só o bit 7, sem os status bits que a boot ROM deixou) |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 12/12 | 12/12 |
| dmg-acid2 | 1/1 | 1/1 |
| Tests do workspace | 900 | 907 (7 novos) |

## Revisão cruzada (segundo modelo)

- **Modelo:** —
- **Achados:** —
- **Procedentes:** —
- **Falso positivo mais interessante:** —

## Decisões de arquitetura

1. **Módulo APU segue o padrão da PPU**: `pub(crate)` em `gb-core`, struct `Apu` com `read`/`write`/`tick`, conectada ao Bus por dispatch de range de endereços e método `tick_apu()` chamado de `Cpu::step()`.
2. **Range de dispatch**: `$FF10`–`$FF26` (registradores NRxx), não o range completo `$FF10`–`$FF3F` que inclui wave RAM (`$FF30`–`$FF3F`). Wave RAM entra no dispatch quando o canal 3 for implementado (6.4).
3. **DIV-APU**: Contador interno de T-cycles que avança 4 por M-cycle e gera um tick do frame sequencer a cada 8192 T-cycles (2048 M-cycles = 512 Hz). O passo do frame sequencer é um contador de 0–7 que avança a cada tick.
4. **NR52**: Armazenado como byte completo (`$F1` no boot), com o bit 7 (power on/off) R/W e os bits 0–3 (status dos canais) read-only preservando o valor de boot.

## Notas

- O frame sequencer com 8 passos a 512 Hz gera os clocks para: length counter a cada 2 passos (256 Hz), sweep a cada 4 passos (128 Hz), envelope a cada 8 passos (64 Hz). Esses eventos serão implementados nos itens seguintes do roadmap.
- A bateria de mutação confirmou que os testes pegam erros de timing (threshold, M-cycle count, ausência de tick), mas não pegam a substituição de `-= 8192` por `= 0` — as duas formas são equivalentes nos múltiplos exatos testados. Isso é aceitável: o teste verifica o comportamento nos pontos de interesse, e `-= 8192` é a forma correta que modela o contador contínuo.
- O NR52 bit 7 write habilita/desabilita a APU, mas o efeito de desligar (zerar registradores) não foi implementado — será feito com os canais.
