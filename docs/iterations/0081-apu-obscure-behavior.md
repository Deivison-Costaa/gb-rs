# Iteração 0081 — APU extra length clocking + obscure behavior

- **Data:** 2026-07-27
- **Item do roadmap:** 6.8b

## Objetivo

Implementar os dois casos de obscure behavior descritos em `07-apu.md` § Obscure Behavior: extra length clocking ao escrever NRx4 (caso 1) e recarga de length timer com 63/255 em vez de 64/256 ao disparar (caso 2).

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | Obscure Behavior | `docs/reference/07-apu.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | O extra length clocking no NRx4 dispara sempre que bit6 transiciona 0→1, independente do passo do DIV-APU | Só dispara quando o próximo passo do DIV-APU **não** é de length clock (passos ímpares: 1, 3, 5, 7) | M1 da bateria: remover o guarda do `next_step_is_not_length_clock()` fez o controle `extra_length_clocking_nao_decrementa_quando_next_step_e_length` falhar |
| 2 | timing | O trigger obscuro (recarga 63/255) acontece em qualquer trigger com length=0 e bit6=1 | Só acontece quando o próximo passo do DIV-APU **não** é de length clock | Inicialmente não codificado com guarda — corrigido antes do commit |
| 3 | timing | 2 × 2048 M-cycles levam o frame sequencer do passo 0 ao passo 0 (ciclo completo de 8 passos) | Cada tick avança 1 passo; 2 × 2048 = 2 ticks → passo 0 → 1 → **2**, não 0 | Testes do trigger obscure e do trigger normal quebravam com `frame_sequencer_step` diferente do esperado; ajustados |
| 4 | — | Escrever NR24=0xC7 como primeiro comando não muda o comportamento dos testes existentes porque o canal se comporta como antes | O init NR24=0xBF (bit6=0) faz a primeira escrita com bit6=1 disparar Case 1, que decrementa o length e depois Case 2 recarrega com 63 em vez de 64 — 7 testes do `apu_length_timer.rs` quebraram | `cargo test --all` após implementar; todos os 7 exigiam trigger com bit6=1 partindo de step=0 (next=1, non-length). Corrigidos posicionando o frame sequencer em step=1 (next=2 IS length) antes do trigger |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 12/12 | 12/12 |
| instr_timing | 1/1 | 1/1 |
| mem_timing | 4/4 | 4/4 |
| dmg_sound | 0/13 | 0/13 |

Testes do workspace: **915** (eram 899 — 16 novos em `apu_obscure_behavior.rs`).

## Revisão cruzada (segundo modelo)

- **Modelo:** —
- **Achados:** 0
- **Procedentes:** 0
- **Falso positivo mais interessante:** —

## Decisões de arquitetura

- A lógica de obscure behavior foi duplicada nos 4 handlers de NRx4 (NR14, NR24, NR34, NR44) em vez de extraída para métodos. Tentar passar `&mut self.chN.length_timer` e `&mut self.chN.enabled` por parâmetro cria conflito de borrow com `&mut self`. O mesmo padrão já existe em `tick_length_timers()`.

- Método `next_step_is_not_length_clock()` exposto como `pub(crate)` no `Apu`. Steps que clockam o length timer são os pares (0, 2, 4, 6); o método retorna `true` para steps ímpares (1, 3, 5, 7).

## Notas

- O init NRx4 = 0xBF (bit6=0) é uma armadilha silenciosa: a primeira escrita que seta bit6 causa a transição 0→1, e se o passo atual for par (next ímpar = non-length), o Case 1 dispara. Isso quebrou 7 testes que usavam `NR24=0xC7` direto de step=0. A correção foi posicionar step=1 antes do trigger, onde next=2 IS length clock, evitando os dois casos de obscure behavior.

- A bateria de mutação encontrou um buraco: o teste `extra_length_clocking_nao_decrementa_quando_bit6_nao_transicionou` original posicionava step=1 (next=IS length) e testava a não-transição, mas o guarda externo `next_step_is_not_length_clock()` escondia a mutação de `prev_off && now_on → true`. Foi adicionado o teste `extra_length_clocking_nao_decrementa_segunda_vez_com_bit6_sem_transicao_e_next_non_length` que testa bit6 sem transição com next=NON-length.
