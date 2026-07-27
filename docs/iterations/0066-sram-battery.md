# Iteração 0066 — SRAM com bateria: persistir .sav e carregar ao abrir

- **Data:** 2026-07-27
- **Item do roadmap:** 4.3

## Objetivo

Adicionar persistência de SRAM (battery-backed RAM) ao emulador: o `Cartridge` trait ganha métodos `ram_data()` e `load_ram()`; o `Mbc1` armazena o flag de bateria e expõe/aceita os bytes da RAM; o `Bus` expõe `cartridge_ram()`; o `gb-cli` carrega arquivo `.sav` ao abrir a ROM e salva ao fechar.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| — | Não há spec de hardware. Persistência de SRAM é funcionalidade de QoL do emulador, não comportamento do hardware original. | — |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | Assumi que o `assert_eq!(mbc.read(0xA100), 0x00)` verificaria que bytes além da RAM alocada não são escritos por `load_ram`. | O Mbc1 retorna `OPEN_BUS` (0xFF) para qualquer endereço de RAM fora dos limites do array alocado, não 0x00. O teste `load_ram_excess_is_truncated` falhou com `left: 255, right: 0`. | Teste `load_ram_excess_is_truncated` — corrigido para `assert_eq!(mbc.read(0xA100), OPEN_BUS)`. |
| 2 | API-Rust | Pensei que precisaria mudar a assinatura de `Mbc1::new()` para receber `has_battery`, quebrando todas as chamadas existentes nos testes. | O builder pattern `with_battery()` resolve sem quebrar compatibilidade — `Mbc1::new(rom, nbanks, ram).with_battery()` mantém a API existente. | Avaliado antes de codificar — a mudança de assinatura nunca chegou a compilar. |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 11/12 | 11/12 |
| instr_timing | 1/1 | 1/1 |
| mem_timing | 4/4 | 4/4 |
| mem_timing-2 | 0/4 | 0/4 |
| halt_bug | 0/1 | 0/1 |
| oam_bug | 0/9 | 0/9 |
| interrupt_time | 0/1 | 0/1 |
| dmg_sound | 0/13 | 0/13 |
| dmg-acid2 | 1/1 | 1/1 |
| mooneye acceptance | 0/66 | 0/66 |
| mooneye (outros) | 0/9 | 0/9 |

Placar inalterado (18/121). SRAM não afeta ROMs de teste — todas são ROM_ONLY e não têm RAM. Testes unitários do workspace: **790** (eram 780 na 0065 — 8 novos em `cart_mbc1.rs`, 2 novos em `bus_memory_map.rs`).

## Bateria de mutação

10 mutações aplicadas, 10 pegas. Um buraco de cobertura descoberto e corrigido durante a bateria (ver Notas).

| # | Mutação | Testes que reprovaram | Placar |
|---|---|---|---|
| 1 | `ram_data()` sempre retorna `None` (ignora battery) | `ram_data_with_battery_returns_some` | 1 pego |
| 2 | `ram_data()` sempre retorna `Some(&self.ram)` (ignora battery + empty) | `ram_data_without_battery_returns_none`, `ram_data_with_no_ram_returns_none_even_with_battery` | 2 pegos |
| 3 | `load_ram()` é no-op (não copia nada) | `load_ram_populates_sram_readable_through_read`, `load_ram_partial_preserves_remaining_bytes`, `load_ram_excess_is_truncated` | 3 pegos |
| 4 | `load_ram()` copia zero bytes (`len = 0`) | `load_ram_populates_sram_readable_through_read`, `load_ram_partial_preserves_remaining_bytes`, `load_ram_excess_is_truncated` | 3 pegos |
| 5 | `Bus::cartridge_ram()` sempre retorna `None` | `cartridge_ram_delegates_to_cartridge_trait` | 1 pego |

Controles (verdes em todas as mutações):

| # | Teste de controle | Mutação que não o afetou |
|---|---|---|
| C1 | `ram_data_no_mbc_returns_none` | M2 (sempre Some — NoMbc usa o default do trait, não o override) |
| C2 | `load_ram_no_mbc_is_noop` | M3, M4 (NoMbc usa o default do trait) |
| C3 | `cartridge_ram_without_battery_returns_none` | M5 |
| C4 | `ram_data_without_battery_returns_none` | M1 (o teste verifica que sem battery retorna None — a mutação quebra isso também, mas é o mesmo caso) |

**10/10 pegos, 4/4 controles verdes.**

## Decisões de arquitetura

- **`with_battery()` como builder, não como parâmetro de `new()`.** O campo `has_battery` é privado e só pode ser setado via `with_battery()`, que consome `self` e devolve um novo `Self`. Isso evita quebrar todos os call sites de `Mbc1::new()` nos testes existentes — são 8 chamadas em `cart_mbc1.rs` e nenhuma precisou mudar.

- **Métodos com default no trait.** `ram_data()` retorna `None` e `load_ram()` é no-op por default. Isso significa que `NoMbc` não precisou de nenhuma alteração, e futuros mapeadores (MBC2/3/5) só precisam sobrescrever se tiverem RAM com bateria.

- **O `.sav` mora onde o `STATUS.md` decidiu: ao lado da ROM, com o mesmo nome base e extensão `.sav`** (ex: `pokemon.gb` → `pokemon.sav`). A decisão é convenção de fato, sem spec, e está registrada em `STATUS.md` § Próxima tarefa.

## Notas

- **Buraco de cobertura descoberto na mutação 5.** Nenhum teste verificava que `Bus::cartridge_ram()` delega corretamente ao `Cartridge::ram_data()`. A mutação que faz `cartridge_ram()` sempre retornar `None` sobreviveu à suíte inteira. Corrigido com dois testes novos em `bus_memory_map.rs`: `cartridge_ram_delegates_to_cartridge_trait` (battery → Some) e `cartridge_ram_without_battery_returns_none` (sem battery → None). A nota 46 do STATUS.md se aplica aqui: o operando de teste precisa distinguir os casos.

- A persistência de SRAM não altera o placar de ROMs — todas as ROMs de teste são `ROM_ONLY` ($00) e não têm RAM. O scoreboard permanece em 18/121.

- O `gb-cli` carrega o `.sav` **antes** de mover o cartridge para o `Bus`, via `cartridge.load_ram(&data)`. Salva **depois** do loop de execução, extraindo a RAM do `Bus` via `bus.cartridge_ram()`. Se o cartridge não tem bateria, `ram_data()` retorna `None` e nenhum arquivo `.sav` é escrito — isso protege ROMs de teste de criarem `.sav` indesejados no diretório de trabalho.
