# docs/reference — a fonte de verdade

> **Regra R1 do `CLAUDE.md`:** antes de implementar qualquer opcode, registrador
> ou comportamento de periférico, leia o arquivo correspondente **aqui**.
> Se a informação não estiver aqui, pare, traga a seção que falta, commite,
> e só então implemente.

Estes arquivos existem porque a intuição de Z80 não vale para o SM83, e porque
"eu li a spec" só significa alguma coisa se a spec for a mesma entre iterações.
Tudo aqui é **fixado por commit** e **commitado no repositório**: é offline,
auditável e imune a mudança upstream silenciosa.

## Procedência

| Fonte | Commit | Licença |
|---|---|---|
| [Pan Docs](https://gbdev.io/pandocs/) | [`fe246067b695`](https://github.com/gbdev/pandocs/tree/fe246067b695b5404a4a6a47efb4fd6d921ececb) | CC0 1.0 (domínio público) |
| [gbops](https://izik1.github.io/gbops/) | [`90b9bf296aed`](https://github.com/izik1/gbops/tree/90b9bf296aed373335a0abbef0a0794a919b9f2c) | MIT |

Regenerar: `./scripts/fetch-reference-docs.sh`.
Os arquivos `01-`…`09-` são **gerados** — não edite à mão, a próxima execução
descarta. Para atualizar as specs, mude o SHA no script, rode, e commite o diff
de `docs/reference/` **separado** de qualquer mudança de código, para que dê
para ver o que a spec mudou.

## Qual arquivo ler para cada item do ROADMAP

| Item do ROADMAP | Leia antes |
|---|---|
| 0.3 header do cartucho, 0.4 `NoMbc` | [`08-cartridges-mbc.md`](08-cartridges-mbc.md) |
| 1.1 registradores e flags | [`02-cpu.md`](02-cpu.md) |
| 1.2 MMU, estado pós-boot | [`01-memory-map.md`](01-memory-map.md) |
| 1.3 laço M-cycle | [`03-opcodes.md`](03-opcodes.md) (coluna *M-cycles passo a passo*) |
| 1.4–1.11 opcodes | [`03-opcodes.md`](03-opcodes.md) + [`02-cpu.md`](02-cpu.md) |
| 1.12 stub serial | [`09-joypad-serial.md`](09-joypad-serial.md) |
| 2.1 timer | [`04-timers.md`](04-timers.md) |
| 2.2 interrupções, 2.3 `HALT` e o bug | [`05-interrupts.md`](05-interrupts.md) |
| 3.1–3.7 PPU | [`06-ppu.md`](06-ppu.md) |
| 4.1 joypad | [`09-joypad-serial.md`](09-joypad-serial.md) |
| 4.2 MBC1, 5.1–5.3 MBC2/3/5 | [`08-cartridges-mbc.md`](08-cartridges-mbc.md) |
| 6.1–6.8 APU | [`07-apu.md`](07-apu.md) |

## Índice

| Arquivo | Conteúdo |
|---|---|
| [`01-memory-map.md`](01-memory-map.md) | Mapa de memória, lista de registradores de I/O, estado pós-boot |
| [`02-cpu.md`](02-cpu.md) | Registradores, flags, semântica das instruções, **comparação com o Z80** |
| [`03-opcodes.md`](03-opcodes.md) | 512 opcodes: flags, tamanho, timing e o que o barramento faz em cada M-cycle |
| [`04-timers.md`](04-timers.md) | DIV, TIMA, TMA, TAC e o comportamento obscuro do overflow |
| [`05-interrupts.md`](05-interrupts.md) | IME/IE/IF, vetores, despacho, `HALT` e o bug do `HALT` |
| [`06-ppu.md`](06-ppu.md) | LCDC/STAT, tiles, tilemaps, OAM, window, scroll, paletas, modos, pixel FIFO |
| [`07-apu.md`](07-apu.md) | 4 canais, frame sequencer, mixer, registradores NR |
| [`08-cartridges-mbc.md`](08-cartridges-mbc.md) | Cabeçalho do cartucho, sem MBC, MBC1, MBC2, MBC3, MBC5 |
| [`09-joypad-serial.md`](09-joypad-serial.md) | P1/JOYP e a porta serial (por onde as ROMs blargg reportam) |

## As armadilhas, em um lugar só

Atalho para o que a R1 existe para evitar. **Não implemente a partir desta
lista** — ela é um índice de onde olhar, não a spec.

- **Rotações.** `RLCA`/`RRCA`/`RLA`/`RRA` zeram `Z` sempre. Os equivalentes com
  prefixo `CB` calculam `Z`. → `03-opcodes.md`
- **`ADD SP,e8` e `LD HL,SP+e8`.** Zeram `Z` e `N`; `H` e `C` saem do **byte
  baixo** (bit 3 e bit 7), não de bit 11/15. → `03-opcodes.md`
- **`DAA`.** Depende de `N` e `H` da operação anterior. → `03-opcodes.md`
- **Bug do `HALT`.** Com `IME=0` e `IE & IF != 0`, o `PC` não incrementa e o
  byte seguinte é lido duas vezes. → `05-interrupts.md`
- **`EI`.** O efeito atrasa uma instrução. → `05-interrupts.md`
- **Overflow de `TIMA`.** Tem atraso; `TIMA` fica em zero por um ciclo antes de
  receber `TMA`. → `04-timers.md`
- **Timing condicional.** `JR/JP/CALL/RET` condicionais duram tempos diferentes
  conforme tomem ou não o desvio. → `03-opcodes.md`
- **Opcodes inexistentes.** `D3 DB DD E3 E4 EB EC ED F4 FC FD` travam a CPU;
  não são `NOP`. → `03-opcodes.md`
- **Endereçamento de tile data.** O "método `$8800`" tem base em **`$9000`** e
  índice **com sinal** (−128..127). O método `$8000` é base `$8000` e índice sem
  sinal. O nome do primeiro não é o endereço-base dele. → `06-ppu.md`
