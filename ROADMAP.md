# ROADMAP

Cada item é **uma iteração = um PR**. Ordem é obrigatória: cada marco depende
do anterior estar verde. Marque `[x]` só depois do merge em `main`.

---

## M0 — Fundação

- [x] 0.1 Workspace Cargo: `gb-core`, `gb-cli`, `gb-desktop`. `#![forbid(unsafe_code)]` no core.
- [x] 0.2 CI: fmt, clippy `-D warnings`, test. Artefato `scoreboard.csv`.
  - [x] 0.2a Job `check`: fmt, clippy `-D warnings` e test rodando **incondicionalmente** (remover a guarda morta do 0.1) + teste que reprova a regressão do workflow.
  - [x] 0.2b Job `scoreboard`: falhar quando `scripts/scoreboard.sh` morre ou o CSV não cresce (`STATUS.md`, nota 7).
  - [x] 0.2c Persistir a série gerada pela CI: publicar o `scoreboard.csv` acumulado numa branch de dados (`scoreboard-data`) no push para `main` (`STATUS.md`, nota 2). **Não é `main`:** a proteção de `main` exige PR, e o `GITHUB_TOKEN` não tem como contorná-la — ver [doc da 0004](docs/iterations/0004-ci-serie-persistida.md).
- [x] 0.3 Parser do header do cartucho (0x0100–0x014F) + `gb-cli info <rom>`: título, tipo de MBC, tamanho ROM/RAM, checksum.
  - [x] 0.3a `CartridgeHeader::parse(&[u8])` em `gb-core`: título, tipo de cartucho, tamanho de ROM/RAM, checksum do header (armazenado × calculado). Puro, sem I/O.
  - [x] 0.3b `gb-cli info <rom>`: leitura do arquivo, parsing de argumentos, impressão e códigos de saída.
- [ ] 0.4 `Cartridge` trait + `NoMbc` (ROM-only, 32KB).
- [ ] 0.5 `scripts/fetch-test-roms.sh`: baixa blargg, mooneye, dmg-acid2 para `tests/roms/`.

## M1 — CPU (sem gráficos)

- [ ] 1.1 Registradores AF/BC/DE/HL/SP/PC, flags Z/N/H/C, pares de 8/16 bits.
- [ ] 1.2 `Bus` trait + MMU: WRAM, HRAM, echo RAM, região proibida. Estado pós-boot (pular boot ROM).
- [ ] 1.3 Laço M-cycle: `step()` avança 1 M-cycle. Fetch/decode/execute como máquina de estados.
- [ ] 1.4 Opcodes: loads 8-bit.
- [ ] 1.5 Opcodes: loads 16-bit + stack (PUSH/POP).
- [ ] 1.6 Opcodes: ALU 8-bit (ADD/ADC/SUB/SBC/AND/OR/XOR/CP/INC/DEC) — **atenção ao half-carry**.
- [ ] 1.7 Opcodes: ALU 16-bit + `ADD SP,e8` / `LD HL,SP+e8` (flags contraintuitivas).
- [ ] 1.8 Opcodes: rotações e shifts (RLCA/RRCA/RLA/RRA — divergem do prefixo CB no flag Z).
- [ ] 1.9 Opcodes: prefixo CB completo (BIT/RES/SET/rot).
- [ ] 1.10 Opcodes: jumps, calls, rets, RST — com timing condicional correto.
- [ ] 1.11 Opcodes: misc — `DAA`, `CPL`, `SCF`, `CCF`, `DI`, `EI`, `NOP`, `STOP`.
- [ ] 1.12 Stub da porta serial (FF01/FF02) → `gb-cli` imprime em stdout.
- [ ] 1.13 blargg `cpu_instrs/individual/01` a `05`.
- [ ] 1.14 blargg `cpu_instrs/individual/06` a `11` + `cpu_instrs.gb` completo.

**Marco M1: 11/11 cpu_instrs, zero código gráfico escrito.**

## M2 — Timing e interrupções

- [ ] 2.1 Timer: DIV, TIMA, TMA, TAC + comportamento de overflow (delay de 4 ciclos).
- [ ] 2.2 Interrupções: IE/IF/IME, vetores, timing de despacho, `EI` com delay de 1 instrução.
- [ ] 2.3 `HALT` + o bug do HALT.
- [ ] 2.4 blargg `instr_timing`, `mem_timing`, `mem_timing-2`, `halt_bug`.

## M3 — PPU

- [ ] 3.1 Registradores: LCDC, STAT, SCY, SCX, LY, LYC, BGP, OBP0, OBP1, WY, WX. VRAM/OAM.
- [ ] 3.2 Máquina de modos (OAM scan 80 / draw / hblank / vblank) + interrupções STAT e VBlank.
- [ ] 3.3 Background por scanline: tilemap, tiledata, endereçamento signed/unsigned.
- [ ] 3.4 Window (incluindo o contador interno de linha da window).
- [ ] 3.5 Sprites: OAM scan, limite de 10/linha, prioridade, flip X/Y, modo 8x16.
- [ ] 3.6 Bloqueio de acesso a VRAM/OAM por modo.
- [ ] 3.7 `dmg-acid2` passando + comparação de hash do framebuffer na CI.

## M4 — Jogável

- [ ] 4.1 Joypad: P1/JOYP + interrupção.
- [ ] 4.2 MBC1: banking de ROM/RAM, modo 0/1.
- [ ] 4.3 SRAM com bateria: persistir `.sav` ao sair, carregar ao abrir.
- [ ] 4.4 `gb-desktop`: winit + pixels, 60 fps, mapeamento de teclado.

**Marco M4: Tetris e Super Mario Land jogáveis.**

## M5 — Mappers

- [ ] 5.1 MBC2 (RAM embutida de 4 bits).
- [ ] 5.2 MBC3 + RTC.
- [ ] 5.3 MBC5.

**Marco M5: Pokémon Red boota, salva e recarrega o save.**

## M6 — APU

- [ ] 6.1 Frame sequencer 512 Hz (length / envelope / sweep).
- [ ] 6.2 Canal 2: square sem sweep (o mais simples — comece por ele).
- [ ] 6.3 Canal 1: square + sweep de frequência.
- [ ] 6.4 Canal 3: wave RAM.
- [ ] 6.5 Canal 4: noise (LFSR de 15/7 bits).
- [ ] 6.6 Mixer: NR50/NR51/NR52, panning, DAC enable.
- [ ] 6.7 Downsample para 48 kHz + ring buffer + saída via `cpal`.
- [ ] 6.8 blargg `dmg_sound` 01 a 12.

## M7 — Rigor

- [ ] 7.1 Suíte Mooneye acceptance no scoreboard.
- [ ] 7.2 `oam_bug`, `interrupt_time`.
- [ ] 7.3 Savestates (serde) + fast-forward + screenshot.

## M8 — Apresentação

- [ ] 8.1 Consolidar `docs/iterations/*` em relatório único.
- [ ] 8.2 Gráficos a partir de `scoreboard.csv`: aprovações por commit, custo por iteração, taxa de erro de primeira tentativa por categoria.
- [ ] 8.3 Roteiro de demo: dmg-acid2 → Tetris → Pokémon Red com save → Prehistorik Man renderizando errado (trade-off do scanline renderer, explicado).
