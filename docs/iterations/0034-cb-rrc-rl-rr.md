# Iteração 0034 — CB RRC + RL + RR

- **Data:** 2026-07-26
- **Item do roadmap:** 1.9b

> PR, custo, turnos e duração: `docs/metricas.csv`.

## Objetivo

Implementar `RRC` (`CB 08`–`CB 0F`), `RL` (`CB 10`–`CB 17`) e `RR` (`CB 18`–`CB 1F`) — 24 opcodes sobre o mecanismo de `CbFetch`/`CbRotHl` da 0033. `RL` e `RR` consomem o `C` antigo como bit de entrada (rotação via carry); `RRC` é auto-contida como `RLC`. Todas calculam `Z` (resultado==0), zeram `N`/`H`, e põem `C` = bit deslocado para fora.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| gbops | Opcodes com prefixo CB (linhas 321–344) | `docs/reference/03-opcodes.md` |
| Pan Docs | $CB prefix instructions (bit layouts) | `docs/reference/02-cpu.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | teste | Eu escrevi `rr_a_clears_z_flag_when_result_is_nonzero` com A=0x01 C=0, esperando resultado != 0 | RR 0x01 C=0 → bit 0 (1) vira C_out, C_in (0) entra no bit 7 → resultado=0x00, Z=1 | Falhou na primeira rodada — o teste passava pelo motivo errado (Z=1 do boot state calhava de bater). Corrigido para A=0x02. |
| 2 | teste | Assumi que todos os bytes 0x08–0x1F estão em `decoded_elsewhere` como opcodes não-CB | Só 22 dos 24 estão: 0x10 (STOP) e 0x18 (JR e8) ainda não foram implementados como não-CB | O teste `cb_second_bytes_08_to_1f_are_decoded_by_cb_fetch_not_by_fetch` falhou para 0x10 e 0x18. Corrigido para listar só os que de fato estão. |

Hardware: nenhum erro. As três operações (`rrc`, `rl`, `rr`) seguem a definição padrão de rotação de 8 bits — `rrc` é simétrico de `rlc`, `rl`/`rr` são simétricos de `rla`/`rra` sem o Z=0 incondicional. O handoff do `STATUS.md` já tinha pré-anunciado a armadilha do Z calculado, e o mecanismo de dois M-cycles do `CbFetch` estava pronto desde a 0033.

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 0/12 | 0/12 |

Nenhum blargg ainda passa.

Testes do workspace: **392** (eram **369** antes da 0034 — +23 do novo arquivo `cpu_cb_rrc_rl_rr`).

## Bateria de mutação

| Mut | Descrição | Pego? | Teste que matou |
|---|---|---|---|
| M1 | `alu::rrc`: carry do bit 7 em vez de bit 0 | Sim | `rrc_b_rotates_right_and_copies_bit_0_to_carry_and_bit_7` (0x01 dá 0x00, não 0x80), `rrc_calculates_z_and_clears_n_h_and_sets_c_from_bit_0` (Z errado para 0x01) |
| M2 | `cb_rl`: carry_in fixo `false` | Sim | `rl_b_with_carry_one_inserts_one_into_bit_0` (0x0B → 0x0A), `rl_calculates_z_and_clears_n_h_and_sets_c_from_bit_7` (Z errado com C_in=true) |
| M3 | `cb_rr`: carry_in fixo `false` | Sim | `rr_b_with_carry_one_inserts_one_into_bit_7` (0xC2 → 0x42), `rr_calculates_z_and_clears_n_h_and_sets_c_from_bit_0` (Z errado com C_in=true) |
| M4 | `cb_rot_hl`: CbRotOp::Rrc → alu::rlc (operação trocada para (HL)) | Sim | `rrc_hl_reads_modifies_and_writes_in_four_m_cycles` (0x0B em vez de 0xC2) |
| M5 | `alu::rl`: `value >> 1` em vez de `value << 1` | Sim | 6/7 testes de RL — `rl_b_rotates_left_and_shifts_old_carry_into_bit_0` (0x42 em vez de 0x0A), `rl_hl_*` etc. |
| M6 | `cb_rot`: N=true, H=true em vez de false | Sim | `rrc_calculates_z_and_clears_n_h_and_sets_c_from_bit_0`, `rl_calculates_z_and_clears_n_h_and_sets_c_from_bit_7`, `rr_calculates_z_and_clears_n_h_and_sets_c_from_bit_0` — 3/3 flag tests falham |
| M7 | `cb_rot`: CbRotOp::Rl ↔ CbRotOp::Rr trocados | Sim | 7 falhas — RL B dá resultado de RR e vice-versa para registradores |

| Ctl | Descrição | Verde? |
|---|---|---|
| C1 | Inverter ordem de `set_flag(N, false)` e `set_flag(H, false)` | Sim (23/23) |
| C2 | `R8::from_bits(opcode & 0b111)` — máscara redundante | Sim (23/23) |

**Placar: 7/7 pegos, 2/2 controles verdes.**

## Decisões de arquitetura

- **`cb_rot`** unifica o dispatch registrador×operação para as quatro rotações. Cada `cb_rlc`/`cb_rrc`/`cb_rl`/`cb_rr` só passa o `CbRotOp` e o `carry_in` (lido de `Flag::C` para RL/RR, `false` para RLC/RRC). A duplicação de quatro métodos idênticos da 0033 fica encapsulada.
- **`cb_rot_hl`** lê `carry_in` do `Flag::C` no momento do `Read` — igual para todas as quatro operações. Para RLC/RRC o parâmetro é ignorado pelo `match` (a função ALU não o usa), mas o custo de ler um booleano é zero e a uniformidade evita surpresa quando as próximas operações (SLA/SRA/SWAP/SRL) entrarem.
- As funções em `alu.rs` (`rrc`, `rl`, `rr`) seguem a mesma interface de `rlc`: devolvem `(resultado, carry)` sem tocar em `Registers`. Quem chama decide as flags.

## Notas

- `rrc` com carry do bit 0 é simétrico de `rlc` com carry do bit 7 — o teste que prova simetria entre as duas é o fato de `rrc(rlc(x)) == x` para todo x. Não foi testado explicitamente, mas os valores cruzados (0x85 → rlc=0x0B → rrc=0x85) o confirmam.
- Os testes de RL e RR com `carry_in=true` são cruciais porque separam as versões CB (Z calculado) das não-prefixadas (`RLA`/`RRA`, Z=0 incondicional) — e também separam uma implementação que ignora o carry de entrada.
- `rl_hl_with_carry_one_uses_it_as_bit_0` e `rr_hl_with_carry_one_uses_it_as_bit_7` não foram pegos pela M3 (que só afetava `cb_rr` de registrador). O caminho (HL) lê `carry_in` em `cb_rot_hl`, que é independente. Uma mutação simétrica em `cb_rot_hl` seria pega por esses testes.
