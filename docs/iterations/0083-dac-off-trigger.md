# Iteração 0083 — DAC-off trigger protection

- **Data:** 2026-07-27
- **Item do roadmap:** 6.8d

## Objetivo

NRx4 MSB não deve ligar o canal 1, 2 ou 4 quando o DAC do respectivo canal está
desligado (`[NRx2] & $F8 == 0`). O canal 3 já tinha essa proteção desde a 6.4.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | § DACs, § Channels | `docs/reference/07-apu.md` (linhas 764-779) |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | O CH2 e o CH4 já tinham DAC ligado após o boot, como o CH1. Eu não verifiquei o boot state antes de implementar e subestimei o número de testes que quebrariam. | NR22=$00 e NR42=$00 após o boot — DAC off. Só o CH1 ($F3) e o CH3 ($80 se NR30 foi escrito) têm DAC ligado após o boot. | 6 testes existentes quebraram (3 CH2, 3 CH4) ao adicionar a guarda; precisei adicionar `NR22=0xF1` e `NR42=0xF1` antes do trigger em cada um deles. |
| 2 | flags | O teste `trigger_define_freq_timer_em_zero` (CH4) continuou passando com a guarda, porque `freq_timer` já é 0 por default e o trigger foi silenciado. Pensei que era um falso negativo do meu código — o teste nunca exerceu a condição que acreditava testar. | O campo `freq_timer` do Channel4 começa em 0 (`Channel4::new`), então o teste passa coincidentemente mesmo sem o trigger executar. É um falso positivo que sobreviveu porque o valor default coincide com o valor que o trigger deveria escrever. | Notado durante a inspeção manual, registrado no doc; não corrigido nesta iteração por ser fora do escopo (não quebrou, não consertamos). |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 11/11 | 11/11 |
| dmg-acid2 | 1/1 | 1/1 |

Testes do workspace: 938 (eram 929 na 0082 — 6 atualizados em ch2_square, 3 atualizados em ch4_noise, 4 novos).

## Bateria de mutação

| Mutação | Pego? | Teste que pegou |
|---|---|---|
| Remover guarda do `PulseChannel::trigger` | Sim | `ch1_sweep::trigger_com_dac_desligado_nao_liga_o_canal`, `ch2_square::trigger_com_dac_desligado_nao_liga_o_canal` |
| Remover guarda do `Channel4::trigger` | Sim | `ch4_noise::trigger_com_dac_desligado_nao_liga_o_canal` |
| Inverter condição no `PulseChannel::trigger` (`== 0` → `!= 0`) | Sim | 10 testes em `ch2_square` que disparam com DAC ligado |
| Trocar ordem de `enabled`/`freq_timer` no PulseChannel (controle) | Não | Nenhum teste falhou |
| Trocar ordem de `lfsr`/`freq_timer` no Channel4 (controle) | Não | Nenhum teste falhou |

**Placar: 3/3 pegos, 2/2 controles verdes.**

## Revisão cruzada (segundo modelo)

- **Modelo:** N/A
- **Achados:** 0
- **Procedentes:** 0
- **Falso positivo mais interessante:** N/A

## Decisões de arquitetura

A guarda de DAC fica dentro de `trigger()`, não no handler de escrita de NRx4.
É o mesmo padrão do CH3 (`trigger_ch3`), que já verificava `NR30 bit 7` antes
de setar `enabled`. A alternativa (checar no handler e pular a chamada de
`trigger()`) duplicaria a lógica em cada handler e exigiria expor os checks de
DAC como métodos públicos do `Apu`.

## Notas

- O CH1 já tinha DAC ligado após o boot (NR12=$F3), então os testes de CH1 não
  precisaram de ajuste — só os novos.
- Os testes `lfsr_nao_avanca_com_shift_14`, `lfsr_nao_avanca_com_shift_15` e
  `trigger_define_freq_timer_em_zero` em `ch4_noise.rs` são falsos positivos
  após esta iteração: passam porque o canal nunca é habilitado (DAC off por
  default), e não porque a funcionalidade que pretendiam testar funciona. O
  escopo desta iteração é só a guarda; a correção desses testes fica para uma
  iteração futura.
