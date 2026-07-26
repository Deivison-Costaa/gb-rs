# Iteração 0033 — CB prefix decode + RLC

- **Data:** 2026-07-26
- **Item do roadmap:** 1.9a
- **PR:** #
- **Duração:** ~30min
- **Custo reportado:** N/D
- **Turnos:** 1

## Objetivo

Decodificar o prefixo `$CB` como transição de estado (`State::CbFetch`) e implementar `RLC` (`CB 00`–`CB 07`, 8 opcodes) com flags calculadas — `Z` a partir do resultado, `N=0`, `H=0`, `C` = bit 7 antigo. `(HL)` em 4 M-cycles (read-modify-write).

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| gbops | Opcodes com prefixo CB | `docs/reference/03-opcodes.md` |
| Pan Docs | $CB prefix instructions | `docs/reference/02-cpu.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | mecanismo | Eu teria lido os dois bytes do CB num M-cycle só (fetch lê `$CB` + próximo byte juntos), como uma instrução de 2 bytes comum | A spec diz `fetch((0xCB)) → fetch`: o `$CB` é o opcode do M1 e o segundo byte é lido num M2 **separado**, com decode próprio | O handoff do `STATUS.md` pré-anunciou esse mecanismo; o teste `rlc_b_takes_two_m_cycles` verifica PC e `is_between_instructions` entre os steps |
| 2 | flags | Eu teria reusado `alu::rlca` para o CB RLC A — Z=0 incondicional, como o `RLCA` não-prefixado | A tabela `03-opcodes.md` mostra `Z` (calculado) na coluna do `CB 07`, contra `0` (literal) na coluna do `07`. A nota da armadilha está no próprio arquivo | O handoff do `STATUS.md` e a nota da 0032 já tinham registrado essa divergência; o teste `rlc_a_sets_z_flag_when_result_is_zero_unlike_rlca` a pega |
| 3 | timing | Eu teria assumido que `(HL)` leva 2 M-cycles (fetch + fetch), como os registradores | A spec põe 16 T-cycles = 4 M-cycles: `fetch((0xCB)) → fetch → read((HL)) → write((HL))` — é read-modify-write como `INC (HL)` | Teste `rlc_hl_reads_modifies_and_writes_in_four_m_cycles` conta os 4 steps e verifica `is_between_instructions` no M4 |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 0/12 | 0/12 |

Nenhum blargg ainda passa — o emulador não tem PPU, timer, interrupções.

Testes do workspace: **369** (eram **352** antes da 0033 — +17 do novo arquivo `cpu_cb_rlc`).

## Bateria de mutação

| Mut | Descrição | Pego? | Teste que matou |
|---|---|---|---|
| M1 | Remover `CB_PREFIX => State::CbFetch` | Sim | `rlc_b_rotates_*` (13/17 falham) |
| M2 | `Z = false` incondicional em vez de `result == 0` | Sim | `rlc_a_sets_z_flag_when_result_is_zero_unlike_rlca`, `rlc_calculates_z_from_result_...` |
| M3 | Não avançar PC em `cb_fetch` | Sim | `rlc_b_takes_two_m_cycles`, `rlc_hl_reads_modifies_...` |
| M4 | Não escrever resultado no registrador | Sim | `rlc_b_rotates_*`, `rlc_c_rotates_*`, etc. (8/17) |
| M5 | Não escrever resultado em `(HL)` | Sim | `rlc_hl_reads_modifies_*`, `rlc_hl_preserves_hl_value` |
| M6 | `N = true` em vez de `false` | Sim | `rlc_calculates_z_from_result_...` |
| M7 | `R8::from_bits(opcode \| 1)` — operando trocado | Sim | `rlc_b_rotates_*`, `rlc_d_rotates_*`, `rlc_h_rotates_*`, `rlc_hl_*` (7/17) |

| Ctl | Descrição | Verde? |
|---|---|---|
| C1 | Inverter ordem de `set_flag(N, false)` e `set_flag(H, false)` | Sim (17/17) |
| C2 | `from_bits(opcode & 0b111)` — máscara redundante | Sim (17/17) |

**Placar: 7/7 pegos, 2/2 controles verdes.**

## Decisões de arquitetura

- **`State::CbFetch`** como novo estado da máquina, não como campo booleano `is_cb` na CPU. Segue o padrão do `State::Fetch` — cada M-cycle novo é uma nova variante de `State`. O `$CB` no `fetch` transita para `CbFetch`; `CbFetch` lê o segundo byte e transita para `Fetch` (ou `CbRotHl`).
- **`CbRotHl`** separa `Read` e `Write` como fases, espelhando `IncDecHl` do 1.6e. O latch carrega o resultado entre as fases.
- **`CbRotOp`** é enum com variante `Rlc` — preparado para `Rrc`, `Rl`, `Rr` nas próximas iterações.
- **`alu::rlc(value) -> (u8, bool)`** retorna `(resultado, carry)` sem tocar em `Registers` — quem chama decide as flags. É diferente de `alu::rlca(&mut Registers)` que fixa Z=0.
- O decode usa `(opcode >> 3) & 0b11111` para isolar os 5 bits da operação, consistente com a tabela do `02-cpu.md` § `$CB prefix instructions`.

## Notas

- O handoff do `STATUS.md` pré-anunciou a armadilha do `Z` e do mecanismo de dois fetches, e nenhum dos dois virou erro — o handoff funcionou. A nota 41 ("o handoff que descreve o erro seguinte funciona") se confirma.
- Os dois testes que passaram com `$CB` não decodificado (`rlc_a_sets_z_flag_when_result_is_zero_unlike_rlca` e `rlc_hl_does_not_change_hl_itself`) passavam por coincidência de boot state — o Z do pós-boot calhava de ser 1 para a ROM de teste. Ajustei o valor de `A` no teste de Z (troquei 0x80 por 0x00) para que a asserção ficasse correta.
- O helper `single_step_reg` sujava `A = 0xFF` como controle de isolamento, mas isso contaminava os testes de `RLC A`. Corrigido: os testes de `RLC A` usam setup direto, sem o helper.
