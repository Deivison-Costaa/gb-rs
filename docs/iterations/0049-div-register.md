# Iteração 0049 — registrador DIV ($FF04)

- **Data:** 2026-07-26
- **Item do roadmap:** 1.14b (preparação)

## Objetivo

Implementar o registrador DIV ($FF04) do timer: contador de sistema visível que
incrementa a cada M-cycle. É a micro-funcionalidade mínima do timer (ROADMAP 2.1)
que o `cpu_instrs.gb` agregado precisa para destravar os loops de timeout dos
sub-testes de interrupção. Sem DIV incrementando, os testes que dependem de
timer/interrupções queimam 250M de ciclos em loops infinitos e o ROM nunca chega
ao código que imprime o veredito final.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | § FF04 — DIV: Divider register | `docs/reference/04-timers.md` |
| Pan Docs | § Timer obscure behaviour (system counter) | `docs/reference/04-timers.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | DIV = `system_counter >> 8` com contador de 16 bits incrementando **1 por M-cycle**. Isso daria DIV a 4096 Hz (1 048 576 / 256), e não a 16384 Hz. | O system counter avança **4 por M-cycle** (taxa de T-cycle, não de M-cycle). Com isso DIV muda a cada 64 M-cycles e a taxa é 1 048 576 / 64 = 16384 Hz. | `div_changes_after_enough_m_cycles_have_passed` — com o erro, depois de 64 passos o contador avança só 64 unidades e o byte alto não muda (DIV permanece $AB). |
| 2 | endereçamento | Escrever em DIV guarda o valor escrito (comportamento de registrador comum) | Escrever qualquer valor em $FF04 **zera** o contador de sistema. O valor escrito é ignorado. | `writing_to_div_resets_the_system_counter_to_zero` — escreve $42 e espera ler $00; com o erro, lê $42. |

O erro #1 é o mesmo que a nota 48 documentou sobre DIV vs. taxa de atualização:
sem a spec, a intuição diz "contador de M-cycles" e a leitura `>> 8` sai com
4096 Hz em vez de 16384 Hz. A diferença de fator 4 é silenciosa (DIV muda a
1/4 da velocidade correta) e nenhuma ROM de CPU-only a detectaria — só testes
de timing como os da suíte Mooneye `timer/`.

O erro #2 também é um clássico de "registrador comum": escrever guarda o byte.
A especificação é explícita e a divergência aparece em qualquer teste que
escreva e leia de volta.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 10/12 | 10/12 |
| mooneye/acceptance | 0/66 | 0/66 |

O placar de ROMs não mudou: o DIV sozinho não resolve o `cpu_instrs.gb` agregado
porque os sub-testes de interrupção não usam DIV como timeout — usam loops que
esperam `IF` mudar, o que depende de TIMA/TAC/MBC e do controlador de
interrupções (M2 completo). O agregado continua com status `crash` (max-cycles)
em vez de `fail` ou `pass`.

A suíte de **testes unitários** subiu de 643 para **650** (+7 testes do
`timer_div.rs`).

## Bateria de mutação

| # | Mutante | Pego por | Veredito |
|---|---|---|---|
| 1 | `wrapping_add(0)` em vez de `(4)` | 4 testes: `div_changes`, `div_increments`, `div_resumes`, `div_wraps` | Pego |
| 2 | Escrever em DIV guarda o valor (`io[index] = value`) | 3 testes: `writing_to_div_resets`, `div_resumes`, `div_wraps` | Pego |
| 3 | Ler DIV do `io[4]` estático em vez de `sys_counter >> 8` | 5 testes: `div_changes`, `div_resumes`, `div_increments`, `writing_to_div_resets`, `div_wraps` | Pego |
| 4 | `wrapping_add(1)` em vez de `(4)` (fator de escala errado) | 3 testes: `div_changes`, `div_resumes`, `div_wraps` | Pego |
| C1 | `wrapping_add(2).wrapping_add(2)` (equivalente a `+4`) | — | Verde (7/7) |
| C2 | `sys_counter: 43776` em vez de `0xAB00` (representação equivalente) | — | Verde (7/7) |

**Placar: 4/4 pegos, 2/2 controles verdes.**

## Revisão cruzada (segundo modelo)

- **Modelo:** não disponível (iteração single-model; note 5).
- **Achados:** 0
- **Procedentes:** 0
- **Falso positivo mais interessante:** —

## Decisões de arquitetura

- O `sys_counter` mora no `Bus`, não num módulo `timer` separado. O timer
  completo (TIMA/TMA/TAC) deve seguir o mesmo padrão, com `tick_timer()` sendo
  chamado de `Cpu::step()`. É a mesma arquitetura do `Serial`: estado no `Bus`,
  acesso por endereço no `read`/`write`, avanço explícito por método público.
- A constante `tick_timer` avança 4 por chamada (1 M-cycle = 4 T-cycles).
  Se o STOP mode (que pausa o system counter) for implementado, o `tick_timer`
  recebe um guarda de `!stopped`.
- O `wrapping_add(4)` é propositalmente wrapping: o contador de 16 bits dá a
  volta naturalmente, e o DIV visível (8 bits) também.

## Notas

- O `cpu_instrs.gb` agregado **não** passa com só o DIV. Os sub-testes de
  interrupção usam `EI` + loop em `IF`, não `DIV` como timeout. O ROM queima
  ~200M+ ciclos nos loops e bate no `max_cycles` antes de imprimir o veredito.
  Para o agregado passar, é necessário TIMA/TAC (timer completo) e o
  controlador de interrupções (IF/IE/IME) — que são os itens M2 inteiros.
- O teste `the_named_registers_have_storage_and_no_read_semantics_yet` em
  `bus_boot_state.rs` foi renomeado para `ly_and_tac_have_storage...` e perdeu
  o `DIV` da lista — o DIV agora tem semântica, como o teste já previa ("Se o
  componente dono chegou, este teste é que está velho").
- A invariante "Valor inicial não é semântica, e a fronteira está fixada por
  teste" permanece: o `sys_counter = 0xAB00` garante `DIV = $AB` no boot, e o
  teste `div_starts_at_the_boot_hand_off_value_ab` fixa essa fronteira.
