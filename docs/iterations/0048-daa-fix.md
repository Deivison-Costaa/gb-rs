# Iteração 0048 — Correção do DAA (SM83, $27)

- **Data:** 2026-07-26
- **Item do roadmap:** 1.14 (parcial — ROM 11 e ROM 01 passam; cpu_instrs.gb pendente)

## Objetivo

Corrigir o algoritmo DAA para usar intermediário u16 e threshold `> 0x9F`, fazendo as ROMs blargg `cpu_instrs/individual/11-op a,(hl)` e `cpu_instrs/individual/01-special` passarem.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | DAA ($27) | `docs/reference/03-opcodes.md` (flags: Z C, H=0, N=-) |
| Pan Docs | DAA description | `docs/reference/02-cpu.md` § BCD Flags |
| BGB (emulador) | DAA implementation | fonte de referência (u16 + `a > 0x9F`) |
| SameBoy (emulador) | DAA implementation | fonte de referência (u16 + `(a & 0xFF) > 0x99`) |
| Blargg test ROMs | `11-op a,(hl)` e `01-special` | `tests/roms/blargg/cpu_instrs/individual/` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | diagnóstico | O erro "27" na ROM 11 era BIT 1,(HL) (CB $4E), seguindo o palpite do STATUS. Passei 30 minutos revisando a implementação de BIT (HL) e CB_BIT sem achar nada. | O "27" é o opcode de DAA ($27), não um índice de teste. O blargg ROM imprime o opcode em hex quando um checksum falha. O DAA era a instrução 50 da tabela de `11-op a,(hl)`. | Ao ler o fonte da ROM 11 (`11-op a,(hl).s`) e o `instr_test.s`, vi que `print_a` imprime o byte do opcode, não o índice. A ROM 01 também tinha DAA (teste #6) falhando pelo mesmo bug. |
| 2 | `flags` | Antes de ler a spec, eu implementei DAA com `u8` e `a > 0x99` (comparação após o wrapping do u8). Essa foi a implementação da iteração 0044, copiada de memória seguindo o algoritmo simplificado do Pan Docs. | O SM83 usa um intermediário que não trunca (u16), e o threshold é `> 0x9F`, não `> 0x99`. O `a > 0x99` com `u8` falha quando o ajuste do nibble baixo estoura o byte (ex: A=0xFA + 0x06 = 0x00, perdendo a informação do carry para o nibble alto). | Teste unitário com A=0xFA (produzia 0x00, devia ser 0x60 com C=1). Confirmado contra a ROM 11: passou após a correção. |
| 3 | `flags` | O threshold `> 0x99` vs `> 0x9F` era indistinguível nos casos de teste existentes. Assumi que não importava. | A=0x99 com H=1: o ajuste do nibble baixo leva o intermediário a 0x9F exatamente. Com `> 0x99`, o 0x60 é adicionado (0x9F > 0x99 é true); com `> 0x9F`, não. O resultado correto é A=0x9F, C=0 (sem ajuste do nibble alto). BGB e mGBA usam `> 0x9F`. | Teste unitário com A=0x99, H=1 (produzia 0xFF com `> 0x99`, deve ser 0x9F). |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs individual | 8/12 | 10/12 (+ROM 01, +ROM 11) |
| cpu_instrs/01-special | fail (DAA #6) | **pass** |
| cpu_instrs/11-op a,(hl) | fail (erro "27" = DAA) | **pass** |
| cpu_instrs/02-interrupts | fail | fail (EI — sem interrupções, esperado) |

## Revisão cruzada (segundo modelo)

- **Modelo:** não disponível (nota 5)
- **Achados:** 0
- **Procedentes:** 0
- **Falso positivo mais interessante:** —

## Decisões de arquitetura

- **Intermediário u16**: o DAA agora usa `u16` para o valor intermediário de A, evitando que o ajuste do nibble baixo perca o carry para o nibble alto quando `A + 0x06 > 0xFF`. A conversão final `a as u8` trunca para 8 bits.
- **Threshold `> 0x9F`**: segue BGB e mGBA, não o Pan Docs (`> 0x99`). O valor 0x9F é o ponto a partir do qual o nibble alto precisa de ajuste quando o nibble baixo já foi corrigido.
- **Z = (a & 0xFF) == 0**: o Z é calculado sobre o byte baixo do intermediário, não sobre o valor completo (embora `a as u8` seja equivalente, a máscara é explícita).

## Notas

- **O erro de diagnóstico custou metade da iteração.** O STATUS da 0047 dizia que "27" era "LD (HL+),A ou BIT 1,(HL)". O bug real era DAA ($27), e o diagnóstico errado me fez revisar toda a implementação de CB_BIT antes de perceber. A lição: em blargg ROMs, o número impresso é o opcode em hex, não o índice do teste.
- **O bug existia desde a 0044** (implementação original do DAA), mas só foi detectado agora porque a ROM 11 é a primeira que testa DAA com cobertura combinatória completa (256 A × 16 flags) via checksum CRC.
- **A ROM 01 também foi corrigida**: o teste DAA #6 usava o mesmo checksum CRC $6A9F8D8A. Antes desta iteração, a ROM 01 produzia um checksum diferente e falhava. Agora passa.
- **O blargg `cpu_instrs.gb` agregado ainda não passa.** Ele tem sub-testes que falham (marcados como `:02`) por causa de instruções que dependem de timer/interrupções (M2). Isso fica para a próxima iteração ou para o M2.
- **A comparação com BGB foi essencial**: o Pan Docs descreve DAA com `> 0x99` e não menciona o tamanho do intermediário. O algoritmo do BGB (validado contra hardware real) usa `> 0x9F` e `u16`. Sem essa referência, eu teria implementado errado de novo.
