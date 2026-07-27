# Iteração 0057 — OAM ($FE00–$FE9F) acessível pelo barramento

- **Data:** 2026-07-26
- **Item do roadmap:** 3.1d

## Objetivo

Adicionar o array de 160 bytes de OAM ao `Bus` e rotear leituras e escritas para ele.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | Memory Map | `docs/reference/01-memory-map.md` |
| Pan Docs | Object Attribute Memory (OAM) | `docs/reference/06-ppu.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | O teste `oam_nao_vaza_para_fora_da_regiao` comparava `bus.read(0xFDFF)` com `OPEN_BUS`, como o teste análogo de VRAM faz com `$7FFF` (que é CartridgeRom) | `$FDFF` é Echo RAM, que espelha WRAM — inicia em `$00`, não `OPEN_BUS`. A região adjacente ao OAM do lado baixo não é ROM, é echo de RAM | execução do teste (assert batia em `0` vs `255`) |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 11/12 | 11/12 |

Scoreboard inalterado (16/121 — o emulador ainda não renderiza). Testes do workspace: **692** (eram 687 na 0056 — 5 novos em `bus_oam.rs`).

## Bateria de mutação

**Placar: 5/5 pegos, 2/2 controles verdes.**

| # | Mutação | Testes que pegaram |
|---|---|---|
| M1 | OAM inicializa com `$FF` | `oam_comeca_zerada_por_escolha` |
| M2 | Escritas são engolidas (no-op) | `oam_guarda_escrita_e_devolve_na_leitura`, `oam_tem_160_bytes_sem_aliasing`, `oam_aceita_escrita_e_leitura_em_todo_o_range` |
| M3 | `oam_index` subtrai `0xFE01` (off-by-one) | todos (panic: index out of bounds em qualquer acesso) |
| M4 | `OAM_LEN = 128` (40×4 errado) | 4/5 (`oam_comeca_zerada_por_escolha`, `oam_tem_160_bytes_sem_aliasing`, `oam_guarda_escrita_e_devolve_na_leitura`, `oam_nao_vaza_para_fora_da_regiao`) |
| M5 | Leitura retorna `OPEN_BUS` em vez do array | 4/5 (`oam_nao_vaza_para_fora_da_regiao` ilesa: não lê OAM) |

Controles:
- C1: implementação correta — 5/5 verdes
- C2: inicialização com `[0; OAM_LEN]` em vez de `[0x00; OAM_LEN]` — 5/5 verdes

## Decisões de arquitetura

OAM fica no `Bus`, não no `Ppu`, seguindo o mesmo padrão da VRAM (0056). O índice
é `(addr as usize) - 0xFE00`. O array tem `40 * 4 = 160` bytes, inicializado com
`$00` por escolha (hardware real tem OAM não-inicializada).

Semânticas de PPU (bloqueio de acesso nos modos 2 e 3, DMA via `$FF46`,
corrupção de OAM) são do ROADMAP 3.2/3.6 — esta iteração só expõe a RAM.

A lista `pending` do teste `the_regions_without_an_owner` esvaziou: OAM era o
último ocupante. As 12 regiões do mapa de memória agora têm dono no `Bus`.

## Notas

A diferença entre os adjacentes de VRAM e OAM surpreendeu: as bordas de VRAM são
ROM ($7FFF, região do cartucho) e RAM externa ($A000, idem) — ambas devolvem o
que o cartucho diz. As bordas de OAM são Echo RAM ($FDFF, espelho de WRAM) e
NotUsable ($FEA0, zero constante). O padrão de teste copiado da VRAM presumia
`OPEN_BUS` na borda baixa e errou.
