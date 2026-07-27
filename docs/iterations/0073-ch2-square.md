# Iteração 0073 — Canal 2: square sem sweep

- **Data:** 2026-07-27
- **Item do roadmap:** 6.2

## Objetivo

Estrutura do canal 2 (square sem sweep): frequency timer de 11 bits, duty step counter (3 bits) e envelope de volume com timer e direção. Sem saída de áudio ainda.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | Sound Channel 2 — Pulse | `docs/reference/07-apu.md` § Sound Channel 1 (NR11–NR14 → NR21–NR24), § Audio details § Pulse channels, § Audio details § DIV-APU |
| Handoff | STATUS.md § Próxima tarefa | `STATUS.md:7–8` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | O frequency timer do canal de pulso é um contador decrescente (viés Z80 — `CTC` decrementa até zero) | É um contador **crescente** de 11 bits. Cada T-cycle incrementa de 1; quando ultrapassa 2047 ($7FF), recarrega do período em NRx3/NRx4. O valor no registrador de período é tratado como complemento de 2: `$500` significa −$300, i.e. 768 passos até o overflow | `freq_timer_do_ch2_avanca_4_por_m_cycle_e_sofre_overflow` — se fosse decrescente, o timer iria para 0 e o overflow nunca aconteceria com o período testado |
| 2 | flags | O `duty_step` reseta a 0 no trigger do canal | A spec diz: "The duty step counter cannot be reset, except by turning the APU off". O trigger reseta o **freq_timer**, não o duty_step. | Não virou código — percebi durante a leitura da spec. Se tivesse virado, o teste `duty_step_avanca_no_overflow_do_freq_timer` verificaria um reset que não acontece. |
| 3 | timing | O envelope do volume dispara a 64 Hz (frequência nominal) | O envelope é clockado nos passos **2** e **6** do frame sequencer (512 Hz / 4 = 128 Hz entre clocks consecutivos dentro do mesmo ciclo, mas a cadência efetiva é 64 Hz porque são 2 clocks por ciclo de 8 passos a 512 Hz). Na primeira versão, o envelope disparava em **todos** os passos (512 Hz), o que fazia o volume cair 4× mais rápido. | `envelope_do_ch2_carrega_volume_do_trigger_e_diminui_no_passo_2` — esperava volume=14 mas recebia 13 (tinha disparado no passo 1 também). `envelope_do_ch2_diminui_de_novo_no_passo_6` — esperava 13 e recebia 9 (6 disparos em vez de 2). |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 12/12 | 12/12 |
| dmg-acid2 | 1/1 | 1/1 |
| blargg (todas) | 18/121 | 18/121 |
| Testes unitários | 907 | 921 (+14) |

## Revisão cruzada (segundo modelo)

- **Modelo:** —
- **Achados:** —
- **Procedentes:** —
- **Falso positivo mais interessante:** —

## Decisões de arquitetura

- `Channel2` é um struct interno de `Apu`, não um tipo público. O `Bus` expõe getters individuais (`ch2_enabled`, `ch2_frequency_timer`, etc.) em vez de emprestar o canal. Isso evita emprestar `&Apu` para teste e mantém o encapsulamento.
- O `tick_freq` só avança quando `ch2.enabled && nr52_power_on`. A alternativa — avançar sempre — causava overflow a cada 512 M-cycles mesmo com canal desligado (freq_timer começava em 0, NR23=$FF → período=2047 → overflow a cada tick após o primeiro ciclo), e foi removeida por custo de desempenho no scoreboard.
- A função `const fn period(nr23, nr24)` calcula o período de 11 bits combinando os dois registradores. Fica como `const fn` para ser usada também nos getters públicos sem overhead.
- O `prev_frame_sequencer_step` existe no struct mas não é usado nesta iteração; só serve de scaffolding para futuro (length timer, que precisa detectar a transição de passo).

## Notas

- O `envelope_timer` no trigger é carregado com `pace` (ou 8 se `pace == 0`, conforme o obscure behavior que trata período 0 como 8). Se `pace` é 0, o envelope fica desabilitado (guarda `if pace == 0 { return; }`).
- O envelope com direção `increase` para em 15, com `decrease` para em 0 — sem wrap.
- O DAC do canal 2 é habilitado quando `NR22 & 0xF8 != 0`. Sem DAC ligado, o trigger ainda ligaria o canal em hardware? A spec diz que trigger com DAC off não liga o canal. **Isso não foi implementado (não estava no escopo "máquina de estados" desta iteração) — fica para 6.3 ou quando o DAC for relevante.**
- O scoreboard rodou sem regressão (18/121), mas ficou mais lento devido ao `tick_freq` por M-cycle antes da correção da guarda `enabled`. A correção (adicionar `&& self.ch2.enabled`) resolveu.

### Bateria de mutação

**3/3 pegos, 2/2 controles verdes.**

| Mutação | Esperado | Resultado |
|---|---|---|
| Envelope dispara em todos os passos (remove `step == 2 \|\| step == 6`) | Falha em `envelope_do_ch2_carrega_volume...` e `envelope_do_ch2_diminui...` | **Pego** — 2 testes falharam |
| FREQ_MAX = 2028 em vez de 2048 | Falha em `freq_timer_do_ch2_avanca...` (overflow no M-cycle errado) | **Pego** — 1 teste falhou |
| Remove `self.freq_timer = period(...)` do trigger | Falha em `trigger_do_ch2_liga...` (freq_timer fica 0) | **Pego** — 1 teste falhou |
| Controle: `let _noop = 1;` em `tick()` | Todos passam | **Verde** — 14/14 |
| Controle: `0 + u16::from_le_bytes(...)` no `period()` | Todos passam | **Verde** — 14/14 |
