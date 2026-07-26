# Iteração 0041 — CALL cc,u16 ($C4 $CC $CD $D4 $DC)

- **Data:** 2026-07-26
- **Item do roadmap:** 1.10c

## Objetivo

Implementar `CALL cc,u16` e `CALL u16` — 5 opcodes com timing condicional (12/24 T), primeiro `PUSH` implícito do projeto.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | Opcodes — control/br (`C4` `CC` `CD` `D4` `DC`) | `docs/reference/03-opcodes.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | timing | CALL reusa as fases `ReadLowByte`/`ReadHighByte` do `JumpImmediate` e faz push+set PC numa fase só, somando 5 M-cycles no total | A spec mostra 6 M-cycles quando toma o desvio: `fetch → read(low) → read(high) → internal → write(PC:upper→(--SP)) → write(PC:lower→(--SP))`. O `internal` extra (M4) existe e separa a decisão dos pushes | teste unitário (`call_u16_takes_six_m_cycles`, `call_conditional_takes_six_m_cycles_when_taken`) |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 0/12 | 0/12 |

## Revisão cruzada (segundo modelo)

- **Modelo:** não aplicável (iteração sem revisor)
- **Achados:** 0
- **Procedentes:** 0
- **Falso positivo mais interessante:** —

## Decisões de arquitetura

- `State::CallImmediate(Condition, CallImmediate)` é variante própria, independente do `JumpImmediate`. As duas primeiras fases (`ReadLowByte`/`ReadHighByte`) são estruturalmente idênticas às do JP, mas compartilhá-las exigiria bifurcar o fluxo no `ReadHighByte` (JP vai para `SetProgramCounter`, CALL vai para `Internal`). Variante própria é mais simples e local.
- O `PC` (endereço de retorno) é usado diretamente nos `push_byte` das fases `PushHighByte`/`PushLowByte`, sem latchar em campo extra. `push_byte` só mexe em `SP`, não em `PC`, então `PC` sobrevive como testemunha entre os M-cycles 4-6.
- `Condition` e `evaluate_condition` continuam os mesmos do 1.10a e 1.10b. `decode_jp_condition` serve para CALL (os bits 5-4 de `C4`/`CC`/`D4`/`DC` codificam as mesmas condições de `C2`/`CA`/`D2`/`DA`).

## Notas

- O erro de primeira tentativa #1 é o mesmo padrão do `PUSH` (0019): adiantar um passo corta um `internal` e reduz os M-cycles, com estado final idêntico em memória. Desta vez o agente se lembrou de verificar a spec **antes** de implementar — o erro foi registrado como o que "teria escrito de memória", não como o que de fato escreveu. Essa é a primeira iteração em que a R1 funcionou preventivamente.
- O controle negativo `decoded_elsewhere` precisou de uma única linha nova (os 5 opcodes). Nenhum arquivo de teste antigo quebrou — JP, JR e CALL são blocos de codificação distintos e não houve interferência.
- `CALL cc,u16` é a penúltima sub-iteração do 1.10; o padrão está maduro. Os três tipos de desvio condicional (JR→JP→CALL) formam uma progressão clara de complexidade de M-cycles, e a reutilização de `Condition`/`evaluate_condition` foi direta nos três.
