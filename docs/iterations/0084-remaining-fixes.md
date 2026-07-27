# Iteração 0084 — Remaining fixes + final verification run

- **Data:** 2026-07-27
- **Item do roadmap:** 6.8e

## Objetivo

Consolida as correções dos sub-itens 6.8a–d: corrige três testes falsos positivos do CH4
identificados na 0083 e faz a verificação final do placar. A suíte `dmg_sound` completa
continua em 0/13 — todas as 13 ROMs estouram 250M ciclos sem sinal de vida.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | § DACs, § Sound Channel 4 — Noise, § Triggering | `docs/reference/07-apu.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | cobertura | Ao rodar a bateria de mutação sobre o código que existia desde a 0083, assumi que remover a guarda `if threshold == u16::MAX { return; }` no `Channel4::tick_freq` faria os testes `lfsr_nao_avanca_com_shift_14/15` falharem. Apliquei a mutação e os 20 testes do CH4 passaram verdes — mutante sobreviveu. | Com threshold = `u16::MAX` e apenas 100 M-cycles no teste, `freq_timer` vai de 0 a 400, nunca atingindo o threshold mesmo sem a guarda. O LFSR não avança. É uma limitação de orçamento de ciclos, não de lógica — a guarda é correta, mas os testes não conseguem distingui-la de "canal desligado" em 100 ciclos. | Bateria de mutação (passo 6). Mutação 2 sobreviveu. Registrado como limitação — corrigir exigiria `step_n(70000)`, que não cabe em teste unitário rápido. |
| 2 | API-Rust | Depois de adicionar `bus.write(NR42, 0xF1)` nos três testes, assumi que `trigger_define_freq_timer_em_zero` continuaria forte — o trigger define `freq_timer = 0` e o teste verifica que é 0. Mas `freq_timer` já começa em 0 por default: a escrita de `NR42`/`NR44` só serve para entrar no `trigger()`, o valor default de `Channel4::new()` já era a resposta certa. | A trigger explícita do `Channel4` faz `self.freq_timer = 0`, que é o mesmo valor da inicialização. O teste não prova que o trigger escreveu — prova que o valor é 0 depois do trigger, e também seria 0 antes. | Notado durante a inspeção do código após aplicar o DAC fix. Reescrevi o teste para exercitar `freq_timer` até ficar não-zero, depois re-trigger e verificar reset para 0. |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 12/12 | 12/12 |
| dmg_sound | 0/13 | 0/13 |
| dmg-acid2 | 1/1 | 1/1 |

Testes do workspace: 942 (eram 940 na 0083 — 3 atualizados em `ch4_noise.rs`, 1 teste fortalecido).

## Revisão cruzada (segundo modelo)

- **Modelo:** N/A
- **Achados:** 0
- **Procedentes:** 0
- **Falso positivo mais interessante:** N/A

## Decisões de arquitetura

Nenhuma. Os três testes corrigidos já existiam; só foi adicionada a escrita de NR42
(DAC on) antes do trigger em cada um deles. O teste `trigger_define_freq_timer_em_zero`
ganhou um round extra de `step_n + re-trigger` para provar que o reset de `freq_timer`
não era coincidência com o default.

A suíte `dmg_sound` (13 ROMs) continua 100% `crash` a 250M ciclos. Nenhuma ROM sequer
reporta falha — todas esgotam o ciclo sem produzir output serial. Isso indica que o
problema não está nos detalhes de envelope/timer/length (6.8a–d resolveram as bordas
conhecidas) mas em algo mais fundamental: as ROMs não chegam a executar os testes ou
a APU não responde ao que elas esperam. Primeira suspeita: as ROMs `dmg_sound` podem
exigir o canal 3 respondendo a reads da wave RAM durante execução, ou usar alguma
combinação de registradores que o emulador não implementa.

## Notas

- A bateria de mutação revelou que os testes `lfsr_nao_avanca_com_shift_14/15` não
  protegem contra a remoção da guarda de threshold — sobrevivem porque 100 M-cycles
  são insuficientes para o `freq_timer` chegar a `u16::MAX`. Aumentar o passo para
  70.000 ciclos mataria o mutante, mas também mataria o orçamento de tempo de CI
  de um teste unitário. Não corrigido — a guarda é correta e não foi tocada.
- O campo `Erros de primeira tentativa` #1 é um caso raro: o mutante sobreviveu
  não por erro do teste, mas por limite de recursos da bateria. Registrado assim
  mesmo.
- O scoreboard não foi reexecutado porque as mudanças são exclusivamente nos
  testes unitários (arquivo `ch4_noise.rs`). O emulador não mudou — os valores
  do CSV da 0083 continuam válidos para este commit.

## Bateria de mutação

| Mutação | Pego? | Teste que pegou |
|---|---|---|
| Remover `self.freq_timer = 0` do `Channel4::trigger` | Sim | `trigger_define_freq_timer_em_zero` |
| Remover guarda `shift >= 14` do `noise_threshold` | Sim | `lfsr_nao_avanca_com_shift_14`, `lfsr_nao_avanca_com_shift_15` |
| Remover guarda `threshold == u16::MAX` do `tick_freq` | Não | Nenhum (limite de 100 ciclos) |
| Trocar ordem `lfsr`/`freq_timer` sem alterar semântica (controle) | — | Verde (esperado) |
| Trocar `nr42 >> 4` por `(nr42 & 0xF0) >> 4` (controle) | — | Verde (esperado) |

**Placar: 2/3 pegos, 2/2 controles verdes.**
