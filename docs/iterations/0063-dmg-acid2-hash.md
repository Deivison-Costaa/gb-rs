# Iteração 0063 — dmg-acid2 + comparação de hash do framebuffer na CI

- **Data:** 2026-07-27
- **Item do roadmap:** 3.7

> O dmg-acid2 é o teste de renderização de precisão que fecha o M3. A ROM
> não usa porta serial — renderiza um frame e entra em loop em `LD B,B`.
> O mecanismo de validação é hash SHA-256 do framebuffer, comparado com
> o valor extraído do PNG de referência oficial.

## Objetivo

Fazer o `dmg-acid2.gb` passar (hash do framebuffer bate com a referência) e
adicionar a comparação de hash ao scoreboard.sh para que a CI monitore o
resultado sem intervenção manual.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| dmg-acid2 README | Reference image, guide | `tests/roms/dmg-acid2/README.md` |
| dmg-acid2 PNG | Referência DMG (160×144, L) | `tests/roms/dmg-acid2/dmg-acid2-dmg.png` |

O PNG de referência tem exatamente 160×144 pixels em modo L (grayscale), com
os quatro tons DMG: $00 (preto), $55 (cinza escuro), $AA (cinza claro),
$FF (branco). Mapeamento para índices 0–3 do framebuffer: $FF→0, $AA→1,
$55→2, $00→3.

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | saída | `print!("fb-hash={hash}"); println!("cycles=...");` — assumi que o `print!` com a macro seguinte `println!` produziriam linhas separadas | `print!` não emite newline; a saída saiu `fb-hash=...cycles=...` grudado numa linha só. O teste `check_fb_hash_prints_hash_before_cycles` verificava a ordem mas não o newline, e passou mesmo assim — foi a inspeção visual da saída do scoreboard que mostrou o problema | Teste `check_fb_hash_prints_hash_before_cycles` (passou, mas não media newline); corrigido trocando `print!` por `println!` |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 11/12 | 11/12 |
| dmg-acid2 | 0/1 | 1/1 |
| blargg total | 17/121 | 17/121 |

dmg-acid2 passou de primeira — o PPU já estava correto o bastante. Nenhuma
correção de renderização foi necessária.

## Bateria de mutação

| # | Mutação | Pego? | Testes que falharam |
|---|---|---|---|
| M1 | Hash esperado trocado no teste por `000...0001` | Sim | `check_fb_hash_with_correct_hash_exits_zero` |
| M2 | Exit codes 0 e 1 trocados em `execute_inner` | Sim | `check_fb_hash_with_correct_hash_exits_zero`, `check_fb_hash_with_wrong_hash_exits_one`, `run_exits_zero_when_serial_output_contains_passed`, `run_exits_one_when_serial_output_contains_failed` |
| M3 | Hash calculado trocado por string constante `"deadbeef"` | Sim (compilação) | Código não compila — `Sha256` fica sem uso e o Rust emite warning; com `-D warnings` vira erro |
| C1 | Comentário alterado no scoreboard.sh ("de referência" em vez de "esperado") | Não (verde) | — |

**Placar: 3/3 pegos, 1/1 controle verde.**

## Decisões de arquitetura

- **Hash em `gb-cli`, não em `gb-core`.** SHA-256 é computação pura, mas
  `gb-core` tem política de zero dependências (`purity.rs`). Adicionar `sha2`
  seria a primeira quebra dessa invariante sem ganho arquitetural — o hash é
  consumido apenas pelo CLI e pelo scoreboard.
- **Flag `--check-fb-hash` vs subcomando separado.** Optei por uma flag em
  `run` porque o contrato com o scoreboard.sh é simples: mesmo comando base
  (`run <rom> --headless --max-cycles`), com uma flag extra que troca o
  critério de sucesso. Um subcomando novo forçaria o scoreboard a ter dois
  caminhos completamente diferentes.
- **Referência do hash extraída do PNG oficial.** O hash
  `f844ea760a6f1fe137f7f992c7ab1c72d34c7fcd3a807b4174a78eb04a32a458` foi
  calculado com um script Python que lê o PNG, converte cada pixel L para o
  índice de shade 0–3, e aplica SHA-256 sobre os 23040 bytes. Esse valor vai
  direto para `scoreboard.sh` como constante `DMG_ACID2_HASH`. Se o PNG de
  referência for atualizado, o hash também precisa ser recalculado.
- **dmg-acid2 não causa lockup.** A ROM termina com `LD B,B` ($40), que é um
  opcode válido (load de registrador para ele mesmo). Não dispara
  `Lockup::IllegalOpcode`. O scoreboard depende de `max_cycles` para terminar
  a execução — 1M de M-cycles é mais que suficiente (um frame completo de PPU
  são ~17.5K M-cycles, e a ROM cabe em algumas centenas de milhares).

## Notas

- O PPU passou no dmg-acid2 **de primeira** — o hash do framebuffer bateu
  com a referência sem nenhuma correção de renderização. Isso significa que
  o M3 (PPU completo: BG, window, sprites, paletas, scrolling, bloqueio de
  VRAM/OAM) está funcionalmente correto para o que o acid2 testa.
- `2.4b` (`halt_bug` e `mem_timing-2`) continua `crash` — não foi reavaliado
  nesta iteração. Com o M3 fechado, a próxima iteração pode reavaliá-los.
