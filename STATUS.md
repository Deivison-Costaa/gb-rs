# STATUS

> Este arquivo é a **memória do projeto entre iterações**. O contexto do agente
> é descartado a cada iteração; este arquivo não. Mantenha-o curto e verdadeiro.

**Última iteração concluída:** 0000 (nenhuma — projeto não iniciado)
**Próxima tarefa:** ROADMAP 0.1 — workspace Cargo
**Marco atual:** M0 — Fundação

## Placar de ROMs de teste

| Suíte | Passando | Total |
|---|---|---|
| blargg cpu_instrs | 0 | 11 |
| blargg instr_timing | 0 | 1 |
| blargg mem_timing | 0 | 2 |
| blargg dmg_sound | 0 | 12 |
| dmg-acid2 | 0 | 1 |
| mooneye acceptance | 0 | — |

## Invariantes já estabelecidas

_(preencher conforme decisões forem tomadas — coisas que iterações futuras
não devem reabrir sem motivo forte)_

- CPU é cycle-stepped (M-cycle). PPU é scanline renderer, não pixel FIFO.
- `gb-core` não tem dependência de I/O.

## Bloqueios

_(nenhum)_

## Notas para a próxima iteração

_(nada ainda)_
