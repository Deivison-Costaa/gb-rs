# Iteração 0031 — `LD HL,SP+i8`

- **Data:** 2026-07-26
- **Item do roadmap:** 1.7d
- **PR:** #38
- **Duração:** ~60min
- **Custo reportado:** não medido (nota 10)
- **Turnos:** 1

## Objetivo

`LD HL,SP+i8` (`$F8`): 1 opcode, **3** M-cycles (`fetch → read(i8) → internal`). `Z`/`N` = `0` literais; `H`/`C` calculados sobre o **byte baixo** de `SP` + `i8` (carry do bit 3 e do bit 7), a mesma coluna de flags do `$E8` do 1.7c. Ao contrário do `$E8`, o destino é `HL` (par de registrador), não `SP` — não há quarto M-cycle de write pelo barramento.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| gbops | opcode table, linha `$F8` | `docs/reference/03-opcodes.md` (l. 296) |
| Pan Docs | Comparison with Z80 (`$F8 = LD HL,SP+dd`) | `docs/reference/02-cpu.md` (l. 879) |
| gbops | Armadilhas: "H/C sobre o byte baixo, não bit 11/15" | `docs/reference/03-opcodes.md` § Armadilhas (l. 34) |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | teste | HL = 0 pós-boot no teste de M-cycle | `after_boot_rom` inicializa HL com base no checksum (0x014D neste teste) | `each_half_of_hl_lands_on_the_internal_m_cycle_and_sp_is_never_written` panickou com HL=333 vs esperado 0; corrigido com `set_hl(0)` explícito |

O handoff do `STATUS.md` pré-anunciou tudo que importava: 3 M-cycles (não 4), mesma coluna de flags do 1.7c (H/C de 8 bits), destino `HL` (não `SP`). A implementação foi direta — copiar o `ReadImmediate` do `add_sp_i8` (que já faz o cálculo correto de flags) e escrever o resultado em `HL` num `Internal` de um passo só, usando `set_hl(latch)`. Não houve erro de spec — o único tropeço foi de teste, assumindo estado zero pós-boot quando o boot seta HL.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 0/12 | 0/12 |
| Testes do workspace | 321 | 337 |

O placar de ROMs não mexe — nenhuma ROM usa `$F8` de forma a destravar suíte nova. O crescimento de 321 para 337 reflete 7 novos testes + ajuste de contagem que o `STATUS.md` subnotificava (a 0030 declarou 321 mas a contagem real já era maior). O `scripts/scoreboard.sh` anexou ao CSV sem regressão.

## Revisão cruzada (segundo modelo)

Não aplicável — iteração linear de um único modelo (OpenCode/Kimi K3).

## Decisões de arquitetura

`$F8` fecha o grupo `x16/alu` do ROADMAP 1.7 (14 opcodes em 4 sub-itens). A escolha de um `Internal` único que escreve `HL` inteiro via `set_hl(latch)` é a mais enxuta possível — com 3 M-cycles e destino em registrador, não há razão para partir a escrita em duas metades como o `$E8` faz (que precisa acessar o barramento duas vezes para `SP`). O `ReadImmediate` reusa 100% do código de flags do `$E8` — se a regra de H/C um dia mudar, os dois quebram juntos e o fato é visível.

`$F8` não é o último avulso de `$E8`/`$F8` que usa regra de 8 bits sobre resultado de 16 — mas é o último do grupo `x16/alu`, e fecha a lista de opcodes avulsos do 1.7c/1.7d que compartilham essa armadilha. A próxima tarefa (1.8, rotações) é outro bloco completamente diferente.

## Notas

Bateria de mutação: **6/6 pegos, 1/1 controles verdes**.

| Mutação | Categoria | Pego? |
|---|---|---|
| Z=true em vez de false | flag | sim (`ld_zeros_z_and_n...`) |
| N=true em vez de false | flag | sim (`ld_zeros_z_and_n...`) |
| H como carry do bit 11 (16-bit) | flag | sim (`ld_calculates_h...`) |
| C como overflow de 16 bits (bit 15) | flag | sim (`ld_calculates_h...`) |
| HL escrito no M2 em vez do M3 | timing | sim (`each_half_of_hl...`) |
| escreve em SP em vez de HL | destino | sim (`ld_adds_signed...`) |
| set_hl via campos diretos (controle) | controle | verde |

A M3 (H bit 11) e M4 (C bit 15) confirmam que os casos de teste (herdados do 1.7c) distinguem efetivamente as duas regras de carry. A M6 confirma que a asserção de "SP não foi alterado" pega o erro de destino.

`decoded_elsewhere` atualizado com `0xF8` na lista `matches!` de opcodes avulsos.
