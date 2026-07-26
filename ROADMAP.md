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
- [x] 0.4 `Cartridge` trait + `NoMbc` (ROM-only, 32KB). A RAM opcional da § No MBC **não** entrou: os tipos que a declaram (`$08`/`$09`) são os que o Pan Docs marca como comportamento desconhecido — ver [doc da 0007](docs/iterations/0007-cart-nombc.md).
- [x] 0.5 `scripts/fetch-test-roms.sh`: baixa blargg, mooneye, dmg-acid2 para `tests/roms/`. Entregue pelo scaffold; a [0008](docs/iterations/0008-fetch-test-roms-guard.md) verificou (121 ROMs, três suítes) e cobriu com teste hermético. O fallback ainda entrega menos do que promete — `STATUS.md`, nota 17.

## M1 — CPU (sem gráficos)

- [x] 1.1 Registradores AF/BC/DE/HL/SP/PC, flags Z/N/H/C, pares de 8/16 bits. **Sem máscara no nibble baixo de `F`**: o Pan Docs no commit fixado não descreve os bits 3–0 nem menciona `POP AF` — ver [doc da 0009](docs/iterations/0009-cpu-registers.md). Se a máscara for necessária, quem cobra é o 1.13.
- [x] 1.2 `Bus` + MMU: WRAM, HRAM, echo RAM, região proibida. Estado pós-boot (pular boot ROM).
  - [x] 1.2a Decodificação de endereço e RAM interna: o mapa de memória inteiro em regiões, WRAM, echo RAM, HRAM, região proibida `$FEA0`–`$FEFF`, e o roteamento das duas janelas do cartucho. Sem valores iniciais — ver [doc da 0010](docs/iterations/0010-bus-memory-map.md). **`Bus` é `struct`, não `trait`** — o item dizia "trait", mas `CLAUDE.md` § Arquitetura diz que o `Bus` é o dono de tudo e que os componentes recebem `&mut Bus`; um trait com um único implementador poria vtable no caminho mais quente do emulador sem comprar nada. Extrair depois é mudança local.
  - [x] 1.2b Estado pós-boot: registradores da CPU e registradores de hardware (`$FF00`–`$FF7F`, `IE`) no hand-off da boot ROM, que este emulador pula. **Quebrado em dois na 0011:** são duas tabelas distintas da § Console state after boot ROM hand-off, e a segunda exige ligar uma região nova ao `Bus` — o que derruba `the_regions_without_an_owner_are_open_bus_and_swallow_writes` e não cabe no mesmo PR pequeno que a primeira.
    - [x] 1.2b-i Registradores da CPU no hand-off: a coluna **DMG** da tabela § CPU registers. `F` não é constante — `H` e `C` saem do checksum do cabeçalho, e é o **gravado em `$014D`**, não o calculado; ver [doc da 0011](docs/iterations/0011-cpu-boot-state.md). Puro, sem tocar no `Bus`.
    - [x] 1.2b-ii Registradores de hardware no hand-off: `$FF00`–`$FF7F` e `IE`, a partir da coluna **DMG / MGB** da tabela § Hardware registers. A tabela dá valor a **41** dos 128 endereços, marca **15** como `---` (só CGB) e **não menciona 72** — os 87 últimos continuam sem dono, e a wave RAM sai dessa lista com a APU (6.4). `OBP0`/`OBP1` são `??` na spec e `$00` por escolha; ver [doc da 0012](docs/iterations/0012-bus-io-boot-state.md). Valor inicial, **não** semântica: sem máscara, sem read-only, sem efeito colateral — isso vem com o componente dono.
- [x] 1.3 Laço M-cycle: `step()` avança 1 M-cycle. Fetch/decode/execute como máquina de estados. **`JP u16` (`C3`) entrou junto, e o item não pedia:** com só `NOP` decodificado a máquina não tem estado — instruction-stepped e cycle-stepped dão o mesmo resultado, e a R2 fica sem teste que a separe do que ela proíbe. Medido, não suposto: contra o esqueleto instruction-stepped, o teste de `NOP` **passou**. Ver [doc da 0013](docs/iterations/0013-cpu-mcycle-loop.md). Os onze opcodes inexistentes travam a CPU (`02-cpu.md` § Moved, Removed, and Added Opcodes); os 243 ainda não decodificados param com `Lockup::UndecodedOpcode`, que é rótulo diferente de propósito.
- [ ] 1.4 Opcodes: loads 8-bit. **Quebrado em quatro na 0014:** o grupo `x8/lsm`
  da tabela de gbops tem **85** opcodes e cinco modos de endereçamento
  distintos — passa de longe do "um PR pequeno, um conceito só" do protocolo de
  iteração. A quebra é por **regra de decodificação**, não por quantidade: cada
  sub-item é um bloco contíguo da tabela com uma forma de M-cycle própria.
  85 = 63 + 8 + 8 + 6.
  - [x] 1.4a O bloco `LD r8,r8` — `$40`–`$7F` **sem** `$76`: 63 opcodes, uma
    regra só (`01 ddd sss`, § Block 1 do `02-cpu.md`) e três formas de M-cycle:
    `LD r,r'` em 1, `LD r,(HL)` e `LD (HL),r` em 2. `$76` é `HALT` — a spec o
    chama de **exceção** à codificação, e ele é o 2.3, não este item.
    `LD r,(HL)` faz a leitura no barramento e a escrita no registrador **no
    mesmo** M2: não há terceiro M-cycle, e supor que houvesse foi a lição do
    `JP u16` aplicada onde ela não vale — ver
    [doc da 0014](docs/iterations/0014-cpu-ld-r8-block.md).
  - [ ] 1.4b Imediatos de 8 bits: `LD r8,u8` (`$06 $0E $16 $1E $26 $2E $3E`) e
    `LD (HL),u8` (`$36`) — o bloco `00 ddd 110`, em 2 e 3 M-cycles.
  - [ ] 1.4c Indireto por par de registradores: `LD (BC),A`, `LD A,(BC)`,
    `LD (DE),A`, `LD A,(DE)` (`$02 $0A $12 $1A`) e as quatro formas com `HL+`/
    `HL-` (`$22 $2A $32 $3A`) — 8 opcodes, e o efeito colateral sobre `HL`.
  - [ ] 1.4d Endereço absoluto e a página `$FF00`: `LD (u16),A` / `LD A,(u16)`
    (`$EA $FA`), `LD (FF00+u8),A` / `LD A,(FF00+u8)` (`$E0 $F0`) e
    `LD (FF00+C),A` / `LD A,(FF00+C)` (`$E2 $F2`) — 6 opcodes.
    **É aqui que a tabela de micro-operações se decide**, e não no 1.4a: a
    0013 previu que o 1.4 traria casos suficientes para generalizar, e traz —
    mas só com os quatro sub-itens no lugar. Generalizar a partir das três
    formas do 1.4a seria a nota 8 outra vez, com um terço dos dados.
- [ ] 1.5 Opcodes: loads 16-bit + stack (PUSH/POP).
- [ ] 1.6 Opcodes: ALU 8-bit (ADD/ADC/SUB/SBC/AND/OR/XOR/CP/INC/DEC) — **atenção ao half-carry**.
- [ ] 1.7 Opcodes: ALU 16-bit + `ADD SP,e8` / `LD HL,SP+e8` (flags contraintuitivas).
- [ ] 1.8 Opcodes: rotações e shifts (RLCA/RRCA/RLA/RRA — divergem do prefixo CB no flag Z).
- [ ] 1.9 Opcodes: prefixo CB completo (BIT/RES/SET/rot).
- [ ] 1.10 Opcodes: jumps, calls, rets, RST — com timing condicional correto. `JP u16` (`C3`) já saiu no 1.3; o que sobra aqui é o difícil — os desvios condicionais duram tempos diferentes conforme tomem ou não o desvio (`8 / 12`, `12 / 24`), e essa é a coluna que a tabela dá em dois valores.
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
