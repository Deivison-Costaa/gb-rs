# Iteração 0035 — CB SLA + SRA + SWAP + SRL (CB 20–CB 3F)

- **Data:** 2026-07-26
- **Item do roadmap:** 1.9c

> PR, custo, turnos e duração: `docs/metricas.csv`.

## Objetivo

Adicionar 32 opcodes do prefixo CB: SLA (0b00100), SRA (0b00101), SWAP (0b00110), SRL (0b00111). As quatro operações compartilham o mecanismo já estabelecido (`CbFetch` → `cb_rot` / `cb_rot_hl`) e diferem apenas na computação da ALU e na flag C.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| tabela gbops | § Opcodes com prefixo CB | `docs/reference/03-opcodes.md` (linhas 345–376) |
| Pan Docs | § CB prefix instructions (Z80 vs GB) | `docs/reference/02-cpu.md` (linha 883) |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | `(value << 4) \| (value >> 4)` para SWAP | clippy sugere `value.rotate_right(4)` — equivalente em u8 | `cargo clippy -- -D warnings` |
| 2 | API-Rust | Closure de tipos diferentes coabitam array | Rust exige tipo homogêneo em arrays; cada closure tem tipo próprio | compilador (`E0308`) |

O erro de hardware previsto — implementar SLL (Z80) em vez de SWAP (GB) nos CB 30–3F — **não ocorreu**: a divergência estava pré-anunciada no handoff do `STATUS.md` e confirmada em `02-cpu.md:883`.

O bug de teste (A sobrescrito no loop multi-registro por `cpu.registers.a = 0xFF`) foi pego pelo próprio teste, que falhou com resultado errado (0xFE em vez de 0x0A para SLA A; 0xFF em vez de 0x21 para SWAP A).

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 0/12 | 0/12 |

Testes do workspace: 414 (eram 392 — +22 do novo arquivo `cpu_cb_sla_sra_swap_srl`).

## Revisão cruzada (segundo modelo)

Não realizada.

## Decisões de arquitetura

As quatro operações foram adicionadas ao enum `CbRotOp` como `Sla`, `Sra`, `Swap`, `Srl`. O `C = false` do SWAP é tratado na ALU (`alu::swap` retorna `carry = false`), não em `cb_rot` — o match genérico `Flag::C = carry` funciona corretamente para todos os casos.

## Notas

- `carry_in` é passado como `false` para as quatro novas operações, mas é ignorado pelo match da ALU (só RL/RR o consomem). Isso não dispara `unused_variable` porque `carry_in` é usado em outros braços do mesmo match.
- O padrão "suja A para isolamento" conflita com testes de A como destino — já tinha aparecido na 0033 com RLC A, mas ali o teste de A era separado do loop. O fix com flag `is_a` é um paliativo; uma refatoração do helper `single_step_reg` para não sujar o destino seria melhor.

## Bateria de mutação

**7/7 pegos, 2/2 controles verdes.**

| # | Mutação | Resultado |
|---|---|---|
| 1 | SLA carry do bit 0 em vez do bit 7 | 2 falhas |
| 2 | SLA bit 0 = 1 em vez de 0 | 6 falhas |
| 3 | SRA sem preservação de bit 7 (`value >> 1` puro) | 2 falhas |
| 4 | SWAP com C = true em vez de false | 4 falhas |
| 5 | SRL bit 7 = 1 em vez de 0 (`(value >> 1) \| 0x80`) | 3 falhas |
| 6 | cb_fetch: 0b00100 → cb_srl em vez de cb_sla | 6 falhas |
| 7 | cb_fetch: 0b00110 → cb_srl em vez de cb_swap | 6 falhas |
| C1 | cb_sla com `carry_in = true` (ignorado pela ALU) | 22/22 verdes |
| C2 | comentário alterado (`shift left arithmetic` → `shift left`) | 22/22 verdes |
