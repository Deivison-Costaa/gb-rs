# Iteração 0039 — Quebra do 1.10 + JR cc,i8 (1.10a)

- **Data:** 2026-07-26
- **Item do roadmap:** 1.10 → 1.10a

## Objetivo

Quebrar o item 1.10 (jumps, calls, rets, RST) em cinco sub-itens por tipo de desvio
e implementar o primeiro: `JR cc,i8` (`$18` `$20` `$28` `$30` `$38`) com timing
condicional correto (8/12 T-cycles).

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| gbops (dmgops.json) | control/br — `$18` `$20` `$28` `$30` `$38` | `docs/reference/03-opcodes.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | teste | O teste de flags condicionais comparava `cpu.registers.f` contra o `DIRTY_F` original, esquecendo que o setup do teste modifica propositalmente Z ou C antes da instrução. A asserção falhava para os casos em que o branch é tomado — Z=0 para `JR NZ` ou Z=1 para `JR Z` — porque o `expected_f` não reproduzia o setup. | O teste deve capturar `expected_f` depois do setup de flag e antes da instrução, e comparar o `f` pós-instrução contra esse valor. | `cargo test --test cpu_jr -- jr_conditional_does_not_affect_flags` |
| 2 | nenhum | — | — | — |

> O JR é uma das poucas instruções em que SM83 e Z80 concordam: mesma condição,
> mesma contagem de M-cycles (3 incondicional, 2/3 condicional), mesmas flags
> intocadas. O trabalho foi de encoding (bits 5-3 mapeiam a condição) e de
> máquina de estados (separar M2 de M3), não de comportamento de hardware.

> O erro do teste #1 é da classe "o setup modifica o valor mas a asserção compara
> contra o original" — um padrão que já apareceu antes (nota 8, "passou de
> primeira é suspeita") e foi pego pela própria suíte nova, não pela bateria de
> mutação (que muta o fonte, não o teste).

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 0/11 | 0/11 |

A suíte `cpu_instrs` continua 0/11 porque JR sozinho não faz um blargg passar —
os testes precisam de loops, calls e rets, que são os próximos sub-itens.

## Revisão cruzada (segundo modelo)

- **Modelo:** não disponível (nota 5 — `REVIEWER_CMD` não configurado)
- **Achados:** —
- **Procedentes:** —
- **Falso positivo mais interessante:** —

## Decisões de arquitetura

1. **`Condition` como enum explícito (Always, NotZero, Zero, NotCarry, Carry).**
   Cada condição é um valor, não uma combinação de flag+polaridade em runtime. A
   função `decode_jr_condition` faz a tradução de `(opcode >> 3) & 0b111` para
   o enum. Alternativa considerada: guardar o índice de flag e polaridade num par
   `(Flag, bool)` — rejeitada porque a forma escolhida é exaustiva no `match`,
   mais fácil de auditar e sem indireção.

2. **`$18` tratado explicitamente, condicionais por máscara.** O `JR_U8` é uma
   constante separada (`$18`), em vez de entrar no `JR_COND_MASK`. Motivo: o
   encoding do `$18` em bits 5-3 é `0b011`, que não segue a sequência dos
   condicionais (`0b100` a `0b111`). Forçar o `$18` na mesma máscara exigiria um
   `match` interno para distinguir o incondicional, e a separação deixa claro
   que o `$18` sempre toma 3 M-cycles (sem fase de decisão no M2).

3. **Offset armazenado como `u8` no `latch`, sign-extendido no M3.** O `latch`
   é `u16`. A leitura do offset em M2 guarda `u16::from(offset)` (0..255). Em M3,
   `(self.latch as u8) as i8` faz sign-extension correta para `wrapping_add_signed`.
   Alternativa: guardar `(offset as i8) as i16 as u16` no M2 — rejeitada porque
   a operação de cast é menos óbvia (bit patterns iguais, caminhos de
   reinterpretação diferentes) e o `as i8` implícito no M3 é mais fácil de
   auditar em inspeção visual.

4. **Quebra do 1.10 em cinco sub-itens por tipo de desvio.** A ordem é da forma
   mais simples para a mais complexa: JR (2-3 M-cycles, sem pilha), JP condicional
   (3-4 M-cycles, sem pilha), CALL (3/5 M-cycles, com PUSH implícito), RET
   (2/4-5 M-cycles, com POP implícito), RST (CALL para endereço fixo, 4 M-cycles).
   A quebra foi commitada antes da implementação de JR, como manda o protocolo
   para itens grandes.

## Notas

- O `grep` por `JR` em `docs/reference/02-cpu.md` devolveu **zero** resultados
  além de uma menção de passagem — a nota 38 acertou de novo. A tabela de
  M-cycles (`03-opcodes.md`) é a única fonte local que descreve o passo a passo.
- A varredura negativa (`jr_opcodes_the_rest_of_block_is_still_undecoded_or_illegal`)
  demora ~3 segundos no total porque cria uma ROM de 32KB para cada um dos
  ~250 opcodes. É o preço do controle negativo: a suíte completa de 19 testes
  roda em 0.00s, e a varredura é o único teste que aparece no perfil.
- O `Condition` implementado aqui será reutilizado por JP condicional, CALL
  condicional e RET condicional — a quebra do 1.10 por tipo de desvio não
  significa reimplementar a condição. O enum é um ativo comum.
