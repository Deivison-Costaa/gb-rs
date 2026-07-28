# Iteração 0086 — halt_bug e mem_timing-2: verificação por hash do framebuffer

- **Data:** 2026-07-27
- **Item do roadmap:** 2.4b

## Objetivo

Fazer `blargg/halt_bug` (1 ROM) e `blargg/mem_timing-2` (4 ROMs) passarem
no placar. As ROMs usam saída visual (framebuffer), não serial — o `gb-cli`
não as detectava.

## Spec consultada

| Fonte | Seção | Arquivo local |
|---|---|---|
| Blargg ROM (disassembly manual) | halt_bug.gb, mem_timing-2/*.gb | `tests/roms/blargg/` |
| Pan Docs | Interrupt handling (HALT) | `docs/reference/05-interrupts.md` |
| Pan Docs | PPU | `docs/reference/06-ppu.md` |

## Erros de primeira tentativa

| # | Categoria | O que eu assumi | O que a spec diz | Como foi pego |
|---|---|---|---|---|
| 1 | `serial` | As ROMs blargg que não produzem saída serial estão presas em loop ou com bug de CPU. Passei uma hora caçando bug de HALT/timer/PPU. | O halt_bug.gb e os mem_timing-2/*.gb usam o framework visual do blargg (VRAM + PPU), não usam porta serial. Nenhuma instrução `E0 01` ou `E0 02` existe nos dois bancos da ROM. | Disassembly manual dos opcodes da ROM: zero acessos a `$FF01`/`$FF02` em halt_bug.gb. O `--check-fb-hash` do dmg-acid2 já existia e era o mecanismo correto. |
| 2 | `serial` | O halt_bug.gb e os mem_timing-2 individuais usam o mesmo framework serial dos cpu_instrs. Assumi que "Passed" sairia pelo serial como nos outros blargg. | Os ROMs de timing (mem_timing, instr_timing) usam serial; os de comportamento de borda (halt_bug, mem_timing-2) usam tela — o framework visual do blargg renderiza resultados via VRAM com espera de VBlank e escrita em tiles. | `xxd` do halt_bug.gb mostrou zero `E0 01`/`E0 02`; o `mem_timing` (que passa) tem 3. |
| 3 | `medição` | `timeout ... | head -10; echo $?` mede o exit code do gb-cli. O pipe capturava o exit code do `head` (sempre 0), e eu passei 20 min acreditando que o halt_bug saía com 0 em 500M ciclos. | `$?` depois de pipe é o último comando do pipeline. | Saída `cycles=500000000` sem "Passed" mas `EXIT=0` — inconsistente. Rodei de novo sem pipe e vi `EXIT=2`. |

## Placar

| Suíte | Antes | Depois |
|---|---|---|
| cpu_instrs | 12/12 | 12/12 |
| instr_timing | 1/1 | 1/1 |
| mem_timing | 4/4 | 4/4 |
| dmg-acid2 | 1/1 | 1/1 |
| halt_bug | 0/1 | 1/1 |
| mem_timing-2 | 0/4 | 4/4 |
| **Total** | **18/121** | **23/121** |

## Revisão cruzada (segundo modelo)

- **Modelo:** N/A — mudança de infraestrutura (scoreboard), não de comportamento de hardware.
- **Achados:** 0
- **Procedentes:** 0
- **Falso positivo mais interessante:** N/A

## Decisões de arquitetura

1. **Hash do framebuffer como mecanismo de veredito para ROMs sem serial.**
   O `--check-fb-hash` já existia para o dmg-acid2. Estendido para halt_bug e
   mem_timing-2 via array associativo `FB_HASHES` no `scoreboard.sh`. O padrão
   é o mesmo: se o hash bater, a ROM passou.

2. **Hashes estáveis a partir de 2M ciclos.** O framebuffer do halt_bug.gb
   estabiliza entre 1M e 2M ciclos; o scoreboard usa 100M (MAX_CYCLES),
   então o hash é capturado muito depois da tela final — sem risco de frame
   parcial.

## Notas

Os ROMs individuais do mem_timing-2 (`01-read_timing`, `02-write_timing`,
`03-modify_timing`) têm o mesmo entry point (`C3 61 21` = JP $2161) do
halt_bug.gb e usam o mesmo padrão: copiam código de ROM para WRAM, executam
de lá, e renderizam resultado na tela. Diferem dos individuais do mem_timing
(não-2), que usam serial (`C3 13 02` = JP $0213) e produzem "Passed" em stdout.

A bateria de mutação confirmou que hashes errados e entradas ausentes derrubam
o resultado para 0/N (2/2 pegos, 2/2 controles verdes).
