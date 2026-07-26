# Iteração 0050 — TIMA, TMA, TAC (timer completo)

- **Data:** 2026-07-26
- **Item do roadmap:** 2.1

## Objetivo

Implementar os registradores TIMA ($FF05), TMA ($FF06) e TAC ($FF07) com
incremento por falling-edge detection nos 4 clocks, overflow com atraso de
1 M-cycle (reload de TMA e flag de timer em IF), e cancelamento de overflow
por escrita em TIMA.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | Timer and Divider Registers | `docs/reference/04-timers.md` |
| Pan Docs | Timer Obscure Behaviour | `docs/reference/04-timers.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | A ordem das operações em `tick_timer` não importa: detectar falling edge e depois tratar reload dá o mesmo resultado que o contrário. | O reload acontece **um M-cycle depois** do overflow. Se a detecção de edge vier antes do tratamento do reload, o overflow e o reload ocorrem no mesmo `tick_timer`. O exemplo da spec mostra TIMA=$00 durante o ciclo inteiro após o overflow, e só no ciclo seguinte TIMA = TMA. | Releitura da spec antes de rodar os testes: a tabela M-cycle do § Timer overflow behavior mostra `TIMA = 00` no ciclo A e `TIMA = 23` no ciclo B, com `IF` só mudando no ciclo B. |
| 2 | timing | Os bits do clock select seguem uma progressão óbvia (bit 0, 1, 2, 3 para as 4 frequências). | A spec mapeia `00→bit 9, 01→bit 3, 10→bit 5, 11→bit 7`. Esses índices não são contíguos nem ordenados por frequência — bit 3 gera 262144 Hz (o mais rápido) e bit 9 gera 4096 Hz (o mais lento). | Confirmei cada mapeamento contra a tabela `Increment every` × `Frequency (Hz)` da spec, calculando `2^(bit+1)/4` M-cycles por tick e casando com a coluna DMG. |
| 3 | flags | O timer só precisa de um flag booleano `tima_overflow_pending` para controlar o reload. | Há três estados: `Idle` (normal), `Overflowed` (ciclo após overflow — TIMA=$00, escrita cancela), `Reloading` (ciclo do reload — TIMA=TMA, escrita em TIMA é ignorada). Um booleano não distingue "escrita durante overflow cancela" de "escrita durante reload é ignorada". | Pensei no teste `writing_to_tima_during_overflow_cycle_cancels_reload` e percebi que um bool só não modela a diferença entre os dois ciclos da spec. |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 10/12 | 10/12 |

O placar não mudou: os sub-testes de interrupção do `cpu_instrs.gb` continuam
falhando porque o controlador de interrupções (2.2) ainda não existe. O timer
agora seta o bit 2 de IF, mas ninguém o consome.

## Bateria de mutação

**Mutantes: 6/6 pegos, controles: 2/2 verdes.**

| # | Mutação | Alvo | Pego? | Teste que matou |
|---|---|---|---|---|
| 1 | `if false` no lugar de `if old_and && !new_and` | TIMA nunca incrementa | sim | todos os 10 testes de tick/overflow |
| 2 | `0 => 8` no lugar de `0 => 9` (clock 00 errado) | bit 9 trocado por bit 8 | sim | `tima_increments_at_4096hz_with_clock_00` |
| 3 | remover `self.prev_and_result = new_and` | falling edge detectado todo ciclo | sim | todos os 10 testes de tick/overflow |
| 4 | remover `self.io[TIMA_IDX] = self.io[TMA_IDX]` | TIMA não recarrega após overflow | sim | `tima_overflow_reloads_from_tma_after_one_m_cycle_delay`, `tima_overflow_sets_if_timer_bit_after_delay` |
| 5 | `0x08` no lugar de `0x04` (bit 3 em vez de 2) | IF bit errado | sim | `tima_overflow_sets_if_timer_bit_after_delay` |
| 6 | remover `self.tima_overflow = TimerOverflow::Idle` do braço `Overflowed` | escrita em TIMA durante overflow não cancela reload | sim | `writing_to_tima_during_overflow_cycle_cancels_reload` |

| # | Controle | Passou? |
|---|---|---|
| 1 | remover transição `Reloading → Idle` no topo de `tick_timer` | sim — o estado `Reloading` persiste mas sem efeito observável (nenhum teste escreve em TIMA durante o ciclo de reload) |
| 2 | trocar `_ => 9` por `_ => 0` no default de `clock_bit` | sim — o braço `_` só é atingido para `select ≥ 4`, e `clock_bit` só é chamada com `tac & 0x03` |

## Decisões de arquitetura

- O timer é implementado diretamente em `bus/mod.rs`, sem módulo separado. O
  estado cabe em 2 campos (`prev_and_result: bool`, `tima_overflow: TimerOverflow`)
  e as funções são curtas. Se o timer crescer com o comportamento obscuro do DMG
  (glitch de disable + falling edge do AND gate), extrair para `bus/timer.rs` é
  mudança local.
- `TimerOverflow` é um enum de 3 estados (`Idle`, `Overflowed`, `Reloading`),
  não um booleano. O terceiro estado é necessário para distinguir "escrita em
  TIMA durante o ciclo de overflow cancela o reload" de "escrita em TIMA durante
  o ciclo de reload é ignorada" — a spec descreve os dois comportamentos como
  distintos, e um `bool` colapsaria os dois casos.
- A falling-edge detection usa `prev_and_result` guardado do ciclo anterior. O
  cálculo do AND gate (`TAC.enable AND sys_counter[selected_bit]`) é recomputado
  a cada `tick_timer` com o `sys_counter` já avançado; a comparação `old == 1 &&
  new == 0` detecta a borda de descida.
- As escritas em DIV e TAC também checam falling edge (antes e depois da
  alteração do estado), conforme o § Timer Obscure Behaviour. A escrita em DIV
  zera `sys_counter` e recalcula o AND; a escrita em TAC atualiza o registrador
  e depois recalcula. Ambas podem disparar `increment_tima`.

## Notas

- O `cpu_instrs.gb` agregado continua sem passar. O timer agora seta o bit 2
  de IF, mas sem o controlador de interrupções (2.2) o bit é escrito e ignorado.
  A próxima iteração (2.2 — IE/IF/IME) é o que destrava os sub-testes de
  interrupção.
- A ROM `instr_timing` (blargg) deve passar a reconhecer os timings de timer
  quando o 2.4 rodar, mas ainda falta HALT (2.3) e o controlador de interrupções
  (2.2) para que os testes de timing condicional funcionem.
- O comportamento obscuro completo (DMG glitch: disable do timer com bit setado
  causa tick; TAC write com troca de clock select causa tick) **não** foi
  implementado — o falling-edge detection simples cobre o caso básico mas pode
  falhar em edge cases que a mooneye cobre. Fica como dívida para quando a suíte
  mooneye começar a rodar.
