# Iteração 0056 — VRAM ($8000–$9FFF) acessível pelo barramento

- **Data:** 2026-07-26
- **Item do roadmap:** 3.1c

## Objetivo

Adicionar o array de 8 KiB de VRAM ao `Bus` e rotear leituras e escritas para ele.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | Memory Map | `docs/reference/01-memory-map.md` |
| Pan Docs | VRAM Tile Data | `docs/reference/06-ppu.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | nenhum | — | — | — |

A spec confirma que VRAM são 8 KiB em $8000–$9FFF e que é "normal RAM". Nenhuma
surpresa. A única decisão foi o valor inicial ($00, como WRAM/HRAM), e a spec
não prescreve valor — é escolha do emulador, feita de forma consciente.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 11/12 | 11/12 |
| instr_timing | 1/1 | 1/1 |
| mem_timing | 4/4 | 4/4 |

O scoreboard do CI vai apontar se houve mudança. Não houve no local (0 passando
antes, 0 passando depois — o emulador ainda não roda ROM gráfica).

## Bateria de mutação

5 mutações aplicadas, 5 pegas, 2 controles verdes:

| # | Mutação | Testes que pegaram |
|---|---|---|
| M1 | VRAM inicializa com `$FF` | `vram_comeca_zerada_por_escolha` |
| M2 | Escritas são engolidas (no-op) | `vram_guarda_escrita_e_devolve_na_leitura`, `vram_tem_8_kib_sem_aliasing` |
| M3 | `vram_index` com overflow (subtraindo 0x8001) | todos (panic) |
| M4 | Escrita em `index + 1`, leitura em `index` | 4/4 escritas (`vram_comeca_zerada_por_escolha` corretamente ilesa) |
| M5 | Leitura retorna `OPEN_BUS` | 4/5 (`vram_nao_vaza_para_fora_da_regiao` não lê de VRAM) |

Controles:
- C1: implementação correta — 5/5 verdes
- C2: mesmo valor inicial ($00) de constante diferente — 5/5 verdes

### Achado de cobertura

`vram_aceita_escrita_e_leitura_em_todo_o_range` usava `step_by(0x400)` com
`pattern(addr) = (addr as u8).wrapping_mul(17)`. Como `addr & 0xFF == 0` em
todos os pontos, `pattern` retornava 0 sempre — o teste passava contra o M2
porque o valor esperado coincidia com o valor inicial zero. Corrigido com step
primo (`0x511`) e `spread(addr) = addr.wrapping_mul(17) as u8`, que produz
valores distintos e não-zero.

## Decisões de arquitetura

VRAM fica no `Bus`, não no `Ppu`. A PPU acessa via `&mut Bus` (como todos os
outros componentes — invariante do STATUS.md). O array é `[u8; 8 * 1024]` com
índice `(addr as usize) - 0x8000`.

Inicializado com `$00` seguindo a mesma convenção de WRAM/HRAM. Hardware real
tem VRAM não-inicializada (aleatória) — a escolha é documentada.

## Notas

A migração de VRAM do teste `the_regions_without_an_owner_are_open_bus_and_swallow_writes`
foi feita: as duas linhas (`$8000` e `$9FFF`) saíram da lista `pending`. OAM
continua listado (próximo item: ROADMAP 3.1d).
