# Iteração 0046 — `gb-cli run` + MBC1 mínimo

- **Data:** 2026-07-26
- **Item do roadmap:** 1.13 (blargg `cpu_instrs/individual/01` a `05`)

## Objetivo

Implementar `gb-cli run <rom> --headless --max-cycles <n>` e adiantar MBC1 mínimo para tornar as ROMs blargg carregáveis.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Pan Docs | Serial Data Transfer (Link Cable) | `docs/reference/09-joypad-serial.md` |
| Pan Docs | MBC1 | `docs/reference/08-cartridges-mbc.md` |
| scoreboard.sh | Contrato do `gb-cli run` | `scripts/scoreboard.sh` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | API-Rust | `for arg in args` permitiria chamar `args.next()` dentro do corpo do loop | `for` consome o iterador; é preciso `while let Some(arg) = args.next()` | compilação (E0382) |
| 2 | endereçamento | `rom_bank_count` podia usar `rom_size.code()` diretamente com `2 << code` para qualquer código | ROMs de teste com `patterned_rom` têm byte `$0148` = `$49` (padrão), que estoura o shift | teste `cart_nombc` (panic no shift) |
| 3 | endereçamento | As ROMs blargg eram ROM ONLY ($00) e carregariam direto | São MBC1 ($01) com 32 KiB — precisam de mapeador mínimo para banco 1 em `$4000-$7FFF` | `cart::load` recusando tipo $01 |
| 4 | flags | O `run` existente sempre saía 2; bastava trocar o corpo | O teste `run_is_still_unimplemented_and_exits_two` esperava exatamente esse comportamento e quebrou | suíte de integração |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 0/12 | 5/12 |

ROMs individuais:
- 01-special: Failed #5 (POP AF — F sem máscara no nibble baixo, já previsto na 0020)
- 02-interrupts: Failed #2 (EI — sem interrupções implementadas)
- 03-op sp,hl: **Passed**
- 04-op r,imm: **Passed**
- 05-op rp: **Passed**
- 06-ld r,r: **Passed**
- 07-jr,jp,call,ret,rst: **Passed**
- 08-misc instrs: Failed (DAA/SCF/CCF — investigar)
- 09 a 11: sem resultado dentro de 5M ciclos

## Revisão cruzada (segundo modelo)

- **Modelo:** não disponível
- **Achados:** N/A
- **Procedentes:** N/A
- **Falso positivo mais interessante:** N/A

## Decisões de arquitetura

1. **MBC1 entrou no 1.13, não no 4.2.** Sem mapeador mínimo as ROMs blargg (todas MBC1) não carregam, e o `gb-cli run` ficaria sem efeito mensurável no placar. O MBC1 implementado é o mínimo para 32 KiB (2 bancos): banco 0 fixo em `$0000-$3FFF`, banco 1 em `$4000-$7FFF` (power-on), registro de banco em `$2000-$3FFF` com máscara e tradução 0→1. Sem RAM, sem registro secundário, sem modo avançado. Os testes completos de MBC1 pertencem ao 4.2.

2. **`Cpu::is_stopped()` como método público.** O laço de `step()` precisa detectar o estado `Stopped` (instrução `STOP`) para não queimar `max_cycles` à toa. `lockup()` não cobre porque `Stopped` não é lockup. O método é `const fn` com `matches!`, sem custo.

## Notas

- O `rom_bank_count` caiu de `const fn` para `fn` porque o fallback (`rom_len / 0x4000`) não é `const` (divisão de `usize` por literal em `const fn` ainda não estabilizado na MSRV 1.85? Na verdade `rom_len` é parâmetro de função `fn`, não `const fn`, então não podia ser `const fn` de qualquer jeito).
- A mensagem de `CartridgeError::RomTooLarge` foi encurtada: a referência explícita a `NoMbc::MAX_ROM_LEN` não fazia sentido quando o erro também pode vir do MBC1.
- O teste `load_refuses_types_it_cannot_map` perdeu `$01` e `$03` da lista de tipos recusados.
- O teste `run_is_still_unimplemented_and_exits_two` virou `run_with_relative_path_that_does_not_point_to_an_existing_rom_exits_with_no_input`.
- A saída serial dos ROMs blargg aparece corretamente: títulos, nomes de teste e resultados chegam como texto ASCII pelo stub da porta serial.
