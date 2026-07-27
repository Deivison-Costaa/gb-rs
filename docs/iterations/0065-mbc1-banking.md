# Iteração 0065 — MBC1 banking: secondary register e modo 0/1

- **Data:** 2026-07-27
- **Item do roadmap:** 4.2

## Objetivo

Completar o banking do MBC1: combinar o registrador secundário (4000-5FFF) como upper bits do endereço de ROM, implementar o efeito do modo 1 na região 0000-3FFF, corrigir o `bank_mask` para ROMs > 512 KiB, e cobrir com testes o que a nota 50 do STATUS.md reportou como "mutação que força banco constante 1 sobreviveu à suíte inteira".

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | MBC1 | `docs/reference/08-cartridges-mbc.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | endereçamento | O `bank_mask` do MBC1 cobre no máximo 5 bits, porque o registrador principal de ROM bank tem 5 bits. A máscara `0x1F` é suficiente para todos os tamanhos de ROM. | Para ROMs > 512 KiB, o endereço efetivo combina o registrador secundário (+2 bits), totalizando até 7 bits. A máscara precisa cobrir o endereço completo (`0x3F` para 1 MiB, `0x7F` para 2 MiB), não só os 5 bits baixos. A máscara fixa em `0x1F` descarta os bits do secondary. | Teste `effective_rom_bank_combines_secondary_register_for_region_4000_7fff` — com 64 bancos e secondary=1, esperava banco 0x21, leu banco 1. |
| 2 | endereçamento | O `effective_bank` só usa o registrador de 5 bits (`rom_bank`). O registrador secundário (`ram_bank`) serve só para RAM. | O endereço efetivo de ROM é `(secondary << 5) \| rom_bank`, com o registrador secundário contribuindo bits 5-6. A única função dual do secondary (RAM bank ou upper ROM bits) é definida por hardware, não por software — o mesmo registrador alimenta os dois caminhos ao mesmo tempo. | Teste `effective_rom_bank_combines_secondary_register_for_region_4000_7fff` — secondary=1 + rom_bank=1 deveria dar banco 33 (0x21), mas sem os upper bits dava banco 1. |
| 3 | endereçamento | A região 0000-3FFF é sempre o banco 0, independentemente do modo. | Em modo 1, as portas AND que forçam os bits dos registradores a 0 em 0000-3FFF são desabilitadas para o registrador secundário. O resultado é que essa região passa a acessar bancos `$20`/`$40`/`$60` conforme o secondary. | Teste `mode_1_applies_secondary_bank_to_region_0000_3fff` — secondary=1 devolveu 0x00 em vez de 0x20. |

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

Placar inalterado (18/121). Testes unitários do workspace: **780** (eram 761 na 0064 — 19 novos em `cart_mbc1.rs`).

## Bateria de mutação

5 mutações aplicadas, 5 pegas. Um buraco de cobertura descoberto durante a bateria e corrigido (ver Notas).

| # | Mutação | Testes que reprovaram | Placar |
|---|---|---|---|
| 1 | Remove upper bits do `effective_bank` (secondary não combinado) | `effective_rom_bank_combines_secondary_register_for_region_4000_7fff`, `secondary_register_with_rom_bank_zero_selects_bank_one_plus_upper_bits`, `switching_secondary_in_mode_1_changes_both_regions` | 3 pegos |
| 2 | Remove modo 1 de `rom_addr` 0000-3FFF | `mode_1_applies_secondary_bank_to_region_0000_3fff`, `switching_secondary_in_mode_1_changes_both_regions` | 2 pegos |
| 3 | Reverte `bank_mask` ao cap `0x1F` | `effective_rom_bank_combines_secondary_register_for_region_4000_7fff`, `secondary_register_with_rom_bank_zero_selects_bank_one_plus_upper_bits`, `mode_1_applies_secondary_bank_to_region_0000_3fff`, `switching_secondary_in_mode_1_changes_both_regions` | 4 pegos |
| 4 | Inverte `banking_mode` (modo 0 age como 1 e vice-versa) | `mode_0_locks_0000_3fff_to_bank_zero`, `mode_1_applies_secondary_bank_to_region_0000_3fff`, `switching_secondary_in_mode_1_changes_both_regions` | 3 pegos |
| 5 | `effective_bank` sempre retorna 1 (quebra 00→01 translation no outro sentido) | `switching_rom_bank_changes_region_4000_7fff`, `bank_mask_caps_effective_bank_to_rom_size` | 2 pegos |

**5/5 pegos, 0 controles necessários** (todas as mutações eram destrutivas e a suíte as capturou).

## Decisões de arquitetura

Nenhuma nova. A estrutura `Mbc1` com campos `rom_bank`, `ram_bank`, `banking_mode` já existia desde a 0046; esta iteração só corrigiu a lógica que os combina.

## Notas

- **Buraco descoberto na bateria:** O teste `mode_0_locks_0000_3fff_to_bank_zero` usava `secondary=2` com ROM de 64 bancos, e o banco 0x40 wrappava para 0 com a máscara 0x3F — indistinguível do comportamento correto. A mutação 4 (inverter `banking_mode`) passou verde na primeira tentativa. Corrigido trocando para `secondary=1` (banco 0x20 = 32 ≤ 63, sem wrap). O buraco estava no operando de teste, não na lógica de produção — variante nova da nota 46.

- O `rom_with_full_bank_mark` armazena o número do banco como byte em cada posição, permitindo distinguir até 256 bancos. O fixture antigo (`rom_with_banks`) usa `(bank << 4) | (offset & 0xF)`, que colapsa para bancos ≥ 16.

- A correção do `bank_mask` afeta só ROMs > 512 KiB (tamanhos `$05` e `$06`). Para ROMs menores, o comportamento é idêntico ao anterior: `(1 << required_bits) - 1` com `required_bits < 5` é igual ao ramo `else` antigo, e `required_bits = 5` dá `0x1F`, igual ao ramo `if` antigo. Nenhum teste existente quebrou.

- A nota 50 do STATUS.md ("mutação que força banco constante 1 sobreviveu à suíte inteira") foi endereçada: a mutação 5 (sempre retornar 1) agora é pega por 2 testes. Mas o aviso da nota 50 era sobre a suíte **inteira** do projeto — uma mutação no MBC1 que sobrevivesse a todos os 740+ testes, não só aos do `cart_mbc1.rs`. Este aspecto não foi verificado: aplicar a mutação e rodar `cargo test --all` exigiria o ciclo completo de rebuild.

- O teste `load_mbc1_ram_battery_creates_ram` revelou que a fixture `rom_mbc1_ram_battery` não setava `0x0148` (ROM size), fazendo o loader interpretar a ROM como 32 KiB e rejeitar ROMs maiores com `RomTooLarge`. Corrigido com a função auxiliar `rom_size_code`.
