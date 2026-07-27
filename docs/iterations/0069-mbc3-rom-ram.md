# Iteração 0069 — MBC3: ROM banking (2MB, 7 bits) + RAM banking (32KB, 4 bancos)

- **Data:** 2026-07-27
- **Item do roadmap:** 5.2a

## Objetivo

Implementar o banking de ROM e RAM do MBC3: até 2MB de ROM com registrador de 7 bits, até 32KB de RAM externa com 4 bancos, seleção RAM/RTC no registrador `$4000–$5FFF`, e dispatch para os 5 tipos de cartucho (`$0F`, `$10`, `$11`, `$12`, `$13`). O RTC (registradores de clock e latch) fica para o 5.2b.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | MBC3 | `docs/reference/08-cartridges-mbc.md` § MBC3 |
| Pan Docs | Cartridge Header (tabela de tipos) | `docs/reference/08-cartridges-mbc.md` § The Cartridge Header |
| Pan Docs | MBC Unmapped RAM Bank Access | `docs/reference/08-cartridges-mbc.md` § MBC Unmapped RAM Bank Access |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | cobertura | O teste `disabling_ram_makes_writes_ignored_and_reads_return_open_bus` cobria supressão de escrita. Ele só verificava que o read retorna `OPEN_BUS`, mas a escrita ao RAM desabilitado passava silenciosamente — o valor era escrito e o read depois não revelava porque o lado do read também estava suprimido. | — | Mutante que removeu `ram_enabled` do lado da escrita sobreviveu à suíte de 27 testes. |
| 2 | cobertura | O enable de RAM do MBC3 exigiria `$0A` exato. A spec diz "a value of $0A" mas o MBC1/MBC5 confirmam que qualquer valor com nibble baixo `$A` ativa. | Qualquer valor com nibble baixo `$A` ativa RAM, como no MBC1. | Mutante que comparava `value == 0x0A` em vez de `(value & 0x0F) == 0x0A` sobreviveu à suíte. |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 12/12 | 12/12 |
| instr_timing | 1/1 | 1/1 |
| mem_timing | 4/4 | 4/4 |
| dmg-acid2 | 1/1 | 1/1 |
| mooneye acceptance | 0/66 | 0/66 |

## Revisão cruzada (segundo modelo)

- **Modelo:** —
- **Achados:** —
- **Procedentes:** —
- **Falso positivo mais interessante:** —

`scripts/review.sh` ainda não configurado (nota 5).

## Decisões de arquitetura

- **Quebra do 5.2 em 5.2a e 5.2b**: MBC3 + RTC são dois conceitos distintos (banking de memória vs. relógio de tempo real). O banking de ROM/RAM é análogo ao MBC1 e fecha em ~110 linhas + 29 testes. O RTC (registradores de clock, latch, halt flag) fica para iteração própria, e até lá os slots RTC (`$08`–`$0C`) não têm armazenamento — escritas são ignoradas e leituras retornam `OPEN_BUS` (se RAM estiver desabilitada) ou o que quer que esteja no banco RAM corrente.

- **Wrap de banco RAM**: bancos não mapeados ($04–$07 para cartuchos com ≤ 4 bancos) dão wrap com módulo, seguindo a seção "MBC Unmapped RAM Bank Access" da spec.

- **MBC3_TIMER_BATTERY ($0F)**: tem bateria (para o RTC) mas não tem RAM externa. O flag `has_battery` é setado, mas `ram_data()` continua retornando `None` porque `ram.is_empty()`. É o primeiro tipo de cartucho com bateria sem RAM — MBC1 e MBC2 não tinham esse caso.

- **O `gb-cli info` não foi alterado**. O reconhecimento textual dos novos tipos de cartucho depende do `CartridgeType` já existente, que aceita qualquer código da tabela e nomeia os conhecidos. Os tipos `$0F`, `$10`, `$11`, `$12`, `$13` já são nomeados corretamente porque a tabela do `header.rs` cobre todos os códigos do Pan Docs.

## Notas

- O 5.2a é a primeira vez que um mapper quebra um teste existente (`load_refuses_types_it_cannot_map` e `error_messages_carry_the_offending_value` em `cart_nombc.rs`) simplesmente por existir — os testes usavam códigos MBC3 como "tipo não suportado". Atualizados para usar MBC5 (`$19`).

- A bateria de mutação revelou dois buracos de cobertura que a leitura isolada dos testes não revelava. Ambos estavam no teste, não na implementação — a implementação estava correta de primeira, mas a suíte não provava isso.

- Placar: **6/6 pegos, 2/2 controles verdes**.
