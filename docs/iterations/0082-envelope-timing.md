# Iteração 0082 — envelope timing: 64 Hz (step 7 only)

- **Data:** 2026-07-27
- **Item do roadmap:** 6.8c

## Objetivo

Corrigir o ritmo do envelope da APU de 128 Hz (passos 2 e 6 do frame sequencer) para
64 Hz (somente passo 7), conforme a tabela de eventos do Pan Docs.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | Audio Overview § Volume & envelope | `docs/reference/07-apu.md` |
| Pan Docs | Audio Details § DIV-APU (tabela de eventos) | `docs/reference/07-apu.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | A 0073 havia implementado envelope nos passos 2 e 6 (128 Hz), e seus testes afirmavam "64 Hz" — confiei nos testes existentes em vez de verificar a tabela de eventos. Steps 2 e 6 são rate 4 (128 Hz), não rate 8 (64 Hz). | A tabela § DIV-APU diz "Envelope sweep \| 8 \| 64 Hz". Rate 8 = cada 8 DIV-APU ticks = 1 vez por ciclo do frame sequencer de 8 passos = passo 7. | Teste novo `envelope_do_ch2_diminui_somente_no_passo_7_e_nao_no_passo_2` falhou com volume=14 no passo 2, provando que o envelope disparava cedo demais. |
| 2 | timing | A 0073 chamou a correção de "fix de 512 Hz para 64 Hz", mas a frequência implementada (128 Hz) era o dobro do que a spec manda. O doc da iteração estava errado e o erro sobreviveu 9 iterações (0073 a 0081). | O envelope ticka a **64 Hz**, metade do que os testes da 0073 fixavam. | A leitura direta da tabela no `07-apu.md` linha 740 mostrou a discrepância: rate 8 ≠ rate 4. |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 12/12 | 12/12 |
| instr_timing | 1/1 | 1/1 |
| mem_timing | 4/4 | 4/4 |
| dmg_sound | 0/13 crash | 0/13 crash |
| dmg-acid2 | 1/1 | 1/1 |

Testes unitários: **929** (eram 915 na 0081 — 14 novos/atualizados nos arquivos de envelope).

## Bateria de mutação

| Mutante | Descrição | Pego? | Teste que matou |
|---|---|---|---|
| M1 | Envelope de volta a `step == 2 \|\| step == 6` | Sim | `envelope_do_ch2_diminui_somente_no_passo_7_e_nao_no_passo_2` |
| M2 | Envelope em `step == 0` | Sim | 5 testes de envelope (CH1/CH2/CH4) |
| M3 | Sweep em `step == 0` em vez de 2/6 | Sim | `sweep_timer_do_ch1_decrementa_nos_passos_2_e_6`, `sweep_iteracao_escreve_novo_periodo_de_volta_em_nr13_nr14` |
| C1 | Sweep: `step == 6 \|\| step == 2` (ordem trocada) | Controle | — |
| C2 | Código correto | Controle | — |

**3/3 pegos, 2/2 controles verdes.**

## Decisões de arquitetura

Nenhuma. A mudança é local: separar o bloco `if` do envelope (agora `step == 7`) do
bloco do sweep (permanece `step == 2 || step == 6`) dentro da mesma função
`Apu::tick()`. O clippy pediu para colapsar o `if` aninhado do sweep.

## Notas

A descoberta principal é processual: a iteração 0073 implementou a frequência errada
(128 Hz em vez de 64 Hz), escreveu no doc que era 64 Hz, e os testes fixaram o
comportamento errado. Nove iterações depois, a discrepância entre o doc da iteração
("64 Hz") e o código ("steps 2 e 6") estava visível mas ninguém cruzou com a tabela
de eventos do Pan Docs. A lição é dupla: (1) teste não substitui spec — se o teste
fixa o valor errado, ele vira âncora; (2) o campo "Erros de primeira tentativa" da
0073 não registrou a confusão entre rate 4 e rate 8 porque o autor achou que steps
2/6 implementavam 64 Hz.

Envelope e sweep agora têm blocos `if` separados em vez de compartilharem `step == 2
|| step == 6`. Isso também facilita a leitura: cada evento da tabela § DIV-APU tem
seu próprio bloco com seu próprio passo.
