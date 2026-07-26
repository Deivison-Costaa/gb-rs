# Iteração 0047 — Máscara do nibble baixo de F (POP AF)

- **Data:** 2026-07-26
- **Item do roadmap:** 1.14 (parcial — ROM 08 passa; ROM 11 adiada)

## Objetivo

Adicionar máscara `& 0xF0` no nibble baixo do registrador F após POP AF e em `set_af`, fazendo ROM 08 (misc instrs) passar.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | § POP AF (`F1`) | `docs/reference/03-opcodes.md` |
| Pan Docs | § Flags Register | `docs/reference/02-cpu.md` |
| Pan Docs | § CPU Comparison with Z80 | `docs/reference/02-cpu.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | flags | Antes de ler a spec (iteração 0009), eu teria mascaro o nibble baixo de F por hábito de Z80. A spec **não** descreve os bits 3–0 em lugar nenhum — a decisão de não mascarar (documentada na invariante) foi correta *baseada na spec*, mas a realidade do hardware (confirmada pela blargg) é que esses bits são sempre 0 | A spec local (`02-cpu.md` § Flags Register) lista só os bits 7–4 e é omissa sobre 3–0. Nenhum dos 75 arquivos do Pan Docs no commit fixado menciona os bits baixos de F | blargg `cpu_instrs/08-misc` (e `01-special`) — a ROM escreve um valor arbitrário na pilha, faz POP AF, e espera F com nibble baixo = 0 |
| 2 | `timing` | *(nenhum de conta)* — a implementação do POP AF já estava correta em M-cycles; o único erro era a ausência da máscara | idem | teste unitário + ROM |
| 3 | `API-Rust` | Que atualizar `set_af` e `write_r16_stk_low` bastaria e os testes existentes se ajustariam com poucas mudanças | 7 testes em 3 arquivos precisaram ser atualizados: `set_af` quebrou todos os testes que usam `pair.set(Pair::Af, ...)` nos arquivos de PUSH/POP e no teste `each_pair_is_independent_of_the_others` | `cargo test` — 4 testes falharam por causa do `set_af`, 3 por causa do `write_r16_stk_low` |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs individual | 7/12 | 8/12 (ROM 08 passou) |
| cpu_instrs/08-misc instrs | fail | **pass** |
| cpu_instrs/11-op a,(hl) | fail | fail (pré-existente, ver notas) |
| cpu_instrs/01-special | fail | fail (ainda falha em DAA #6, não POP AF) |

## Revisão cruzada (segundo modelo)

- **Modelo:** não disponível (nota 5)
- **Achados:** 0
- **Procedentes:** 0
- **Falso positivo mais interessante:** —

## Decisões de arquitetura

- **Máscara aplicada em dois lugares**: `write_r16_stk_low` (POP AF) e `set_af` (consistência). Só `write_r16_stk_low` era necessário para passar a ROM, mas `set_af` foi incluído para que futuros usos (fora de teste) herdem o comportamento correto.
- **PUSH AF não foi alterado**: continua escrevendo os 8 bits de F na pilha — como F agora tem nibble baixo = 0 em operação normal, o byte escrito é naturalmente correto. Testes que forçam `f = 0x57` diretamente (via `registers.f = ...`) continuam funcionando porque a máscara é só na *escrita opcode* em F, não na *leitura* via `af()`.

## Notas

- **ROM 11 (`op a,(hl)`)** continua falhando com erro "27". Este é um problema pré-existente, não causado pela máscara de F (os valores C usados no teste — $00, $10, $E0, $F0 — já têm nibble baixo 0 e não são afetados pela máscara). O erro "27" corresponde ao checksum da instrução 6 (`LD (HL+),A`) ou da instrução 27 (`BIT 1,(HL)`), conforme a interpretação do formato de erro do blargg — não foi possível determinar com certeza nesta iteração. A investigação fica para a próxima iteração.
- **O cache de build (nota 14) atacou na bateria de mutação**: após reverter M5 (máscara `0x00` → `0xF0`) com `sed` + `touch`, o binário antigo sobreviveu. Foi preciso `cargo clean -p gb-core` para ver o resultado correto. Confirmado: `sed -i` + `touch` **não** basta — o `mtime` do artefato pode ficar mais novo que o fonte revertido.
- **A invariante `F carrega os 8 bits` foi atualizada** para refletir a nova realidade: o registrador F armazena 8 bits no `struct`, mas as operações de escrita mascaram o nibble baixo.
