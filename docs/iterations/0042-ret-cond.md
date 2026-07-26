# Iteração 0042 — RET cc + RET + RETI

- **Data:** 2026-07-26
- **Item do roadmap:** 1.10d

<!-- PR, custo, turnos e duração NÃO entram aqui: no passo 7 nenhum deles existe
     ainda. Ficam medidos em docs/metricas.csv, casados por head_antes/head_depois. -->

> Preencha só o que você sabe de dentro da iteração. Se um campo exige um número
> que só o orquestrador enxerga, ele não pertence a este documento.

## Objetivo

Implementar os 6 opcodes de retorno — `RET NZ`/`Z`/`NC`/`C` (`$C0 $C8 $D0 $D8`), `RET` (`$C9`) e `RETI` (`$D9`) — com timing condicional correto (2/5 M-cycles) e reuso do `pop_byte` da pilha.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| gbops (Pan Docs) | Tabela de opcodes `C0`–`D9` | `docs/reference/03-opcodes.md` |
| gbops (Pan Docs) | `ret cond` — layout de bits | `docs/reference/02-cpu.md` |
| Pan Docs | Interrupções — `RETI` | `docs/reference/05-interrupts.md` |

## Erros de primeira tentativa

> Campo mais importante do documento. O que o agente escreveu ou ia escrever de
> memória, e que a spec contradisse. Categorias: `flags`, `timing`, `endereçamento`,
> `borrow-checker`, `API-Rust`, `nenhum`.

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | O handoff do `STATUS.md` dizia que os bits de condição do RET eram 5-4. Se eu tivesse escrito uma função nova de decodificação, o NZ/Z/NC/C teriam saído trocados de posição. | Os bits de condição são 4-3, mesma posição de JP/CALL — confirmado verificando cada opcode contra a tabela do `03-opcodes.md` e o `decode_jp_condition` existente. | Ao verificar o `decode_jp_condition` existente, que usa `(opcode >> 3) & 0b11` (bits 4-3), percebi que o handoff estava errado — e reutilizei a função existente em vez de escrever uma nova. |
| 2 | timing | Antes de ler a spec, minha intuição era modelar RET condicional como `State::ReturnImpl(Condition::Always)` e reusar a máquina de CALL — um `Internal` extra que decide depois de ler os operandos. | RET condicional **não lê** operandos se o desvio não for tomado: o M2 é `internal(branch decision?)` sem leitura de barramento. Usar o mesmo estado do CALL incondicional adicionaria 3 M-cycles ao caminho curto (8 T → 20 T). | Verificação da tabela M-cycle: a coluna `fetch → internal` (sem seta de leitura) para o caminho curto. Teste `ret_conditional_takes_two_m_cycles_when_not_taken` (8 T). |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 0/12 | 0/12 |
| Testes do workspace | 511 | 535 (+24) |

## Revisão cruzada (segundo modelo)

- **Modelo:** —
- **Achados:** —
- **Procedentes:** —
- **Falso positivo mais interessante:** —

## Decisões de arquitetura

1. **Dois estados, não um.** `ReturnConditional(Condition)` é o M2 de decisão para os 4 condicionais; `ReturnImpl(ReturnPop, bool)` cobre os passos de pop (`ReadLowByte → ReadHighByte → SetProgramCounter`) compartilhados por todos os 6 opcodes. O `bool` carrega a flag de IME para o `RETI` sem criar uma variante de estado separada. Essa escolha evita duplicar as 3 fases de pop para `C9` e `D9`.

2. **`ime: bool` público no `Cpu`.** `RETI` seta `ime = true` no M4; os outros opcodes deixam o campo intocado. Interrupções (M2) ainda não existem — a flag é stub, mas testável. O campo é `pub` como `registers` para os testes poderem verificá-lo.

3. **Reuso de `pop_byte` e `decode_jp_condition`.** A função `pop_byte` (pós-incremento, 1.5c) é a mesma usada por `POP r16stk`; `decode_jp_condition` extrai bits 4-3 do opcode e serve para JP, CALL e RET cc sem alteração — confirmado para todos os 4 opcodes da máscara `0xE7`/`0xC0`.

## Bateria de mutação

**Placar: 6/6 pegos, 2/2 controles verdes.**

| # | Mutação | Pego por | Resultado |
|---|---|---|---|
| 1 | `return_conditional` sempre toma o desvio (`if true`) | `ret_nz_not_taken_when_z_is_set` | Pego |
| 2 | Ordem dos bytes invertida no latch (low ⇄ high) | `ret_pops_return_address_from_stack_and_jumps`, `ret_pops_low_byte_first_then_high_byte` | Pego |
| 3 | `return_impl` nunca seta IME | `reti_behaves_like_ret_and_sets_ime`, `reti_takes_four_m_cycles` | Pego |
| 4 | `return_impl` sempre seta IME | `ret_does_not_set_ime` | Pego |
| 5 | Leitura da pilha sem pós-incremento de SP | `ret_takes_four_m_cycles` | Pego |
| 6 | RET cc decodificado direto para `ReturnImpl` (sem `ReturnConditional`) | `ret_nz_not_taken_when_z_is_set` | Pego |
| C1 | Extração de variável temporária em `return_conditional` | — | Verde |
| C2 | Temp `let pc = self.latch` em `SetProgramCounter` | — | Verde |

## Notas

- A máscara `0xE7`/`0xC0` pega exatamente `C0 C8 D0 D8` e não colide com `C9` (RET), `D9` (RETI), `C1` (POP BC), `C5` (PUSH BC) nem `C7`/`CF`/`D7` (RST). Verificado para os 256 opcodes — é a mesma posição de campo do JP/CALL mas com bit 5 livre, que `D` seta.
- O `pop_byte` que lê em `sp` e faz `sp += 1` é exatamente o mesmo usado por `POP r16stk`. Nenhuma função nova de acesso à pilha foi necessária.
- `RETI` é o primeiro opcode do projeto a modificar o IME. O campo `ime: bool` entrou no `Cpu` como stub para o M2.
