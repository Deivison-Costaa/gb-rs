# Tabela de opcodes do SM83

> **Fonte de verdade deste projeto (regra R1 do CLAUDE.md).**
> Gerada de [gbops](https://izik1.github.io/gbops/) (`dmgops.json`), fixado no
> commit [`90b9bf296aed`](https://github.com/izik1/gbops/tree/90b9bf296aed373335a0abbef0a0794a919b9f2c).
> Não editar à mão: regenerado por `scripts/fetch-reference-docs.sh`.

## Como ler esta tabela

**Flags (colunas Z, N, H, C):**

| Símbolo | Significado |
|---|---|
| `-` | flag não é afetada |
| `0` | flag é sempre zerada |
| `1` | flag é sempre setada |
| `Z` / `N` / `H` / `C` | flag é calculada a partir do resultado |

**T-cycles:** 1 M-cycle = 4 T-cycles. Onde aparecem dois valores (`8 / 12`), o
primeiro é sem tomar o desvio e o segundo é tomando — timing condicional.

**M-cycles (passo a passo):** o que o barramento faz em cada M-cycle da
instrução. É esta coluna que a regra R2 exige: `Cpu::step()` avança **um**
M-cycle, então cada passo aqui é uma parada da máquina de estados, não um
somatório aplicado no fim.

## Armadilhas que esta tabela resolve (R1)

O SM83 não é um Z80. Os pontos onde a intuição de Z80 erra e a tabela desmente:

- **`RLCA`/`RRCA`/`RLA`/`RRA` (opcodes `07`, `0F`, `17`, `1F`)** zeram `Z`
  incondicionalmente. Os equivalentes prefixados por CB (`CB 07` = `RLC A` etc.)
  **calculam** `Z`. Mesmo nome, flag diferente.
- **`ADD SP,i8` (`E8`) e `LD HL,SP+i8` (`F8`)** zeram `Z` e `N`, e calculam
  `H`/`C` sobre o **byte baixo** — carry de bit 3 e bit 7, não de bit 11/15
  como um `ADD HL,rr` faria.
- **`DAA` (`27`)** depende de `N` e `H` deixados pela operação anterior.
- **`LD (a16),SP` (`08`)** tem 5 M-cycles e escreve dois bytes.
- Opcodes `D3 DB DD E3 E4 EB EC ED F4 FC FD` **não existem** no SM83 e travam a
  CPU. Não são NOPs.

---

## Opcodes sem prefixo

| Opcode | Instrução | Grupo | Bytes | T-cycles | Z | N | H | C | M-cycles (passo a passo) |
|---|---|---|---|---|---|---|---|---|---|
| `00` | `NOP` | control/misc | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `01` | `LD BC,u16` | x16/lsm | 3 | 12 | `-` | `-` | `-` | `-` | fetch → read(u16:lower->C) → read(u16:upper->B) |
| `02` | `LD (BC),A` | x8/lsm | 1 | 8 | `-` | `-` | `-` | `-` | fetch → write(A->(BC)) |
| `03` | `INC BC` | x16/alu | 1 | 8 | `-` | `-` | `-` | `-` | fetch(Probably writes to C here) → internal(Probably writes to B here) |
| `04` | `INC B` | x8/alu | 1 | 4 | `Z` | `0` | `H` | `-` | fetch |
| `05` | `DEC B` | x8/alu | 1 | 4 | `Z` | `1` | `H` | `-` | fetch |
| `06` | `LD B,u8` | x8/lsm | 2 | 8 | `-` | `-` | `-` | `-` | fetch → read(u8->B) |
| `07` | `RLCA` | x8/rsb | 1 | 4 | `0` | `0` | `0` | `C` | fetch |
| `08` | `LD (u16),SP` | x16/lsm | 3 | 20 | `-` | `-` | `-` | `-` | fetch → read(u16:lower) → read(u16:upper) → write(SP:lower->(u16)) → write(SP:upper->(u16+1)) |
| `09` | `ADD HL,BC` | x16/alu | 1 | 8 | `-` | `0` | `H` | `C` | fetch(Probably writes to L here) → internal(Probably writes to H here) |
| `0A` | `LD A,(BC)` | x8/lsm | 1 | 8 | `-` | `-` | `-` | `-` | fetch → read((BC)->A) |
| `0B` | `DEC BC` | x16/alu | 1 | 8 | `-` | `-` | `-` | `-` | fetch(Probably writes to C here) → internal(Probably writes to B here) |
| `0C` | `INC C` | x8/alu | 1 | 4 | `Z` | `0` | `H` | `-` | fetch |
| `0D` | `DEC C` | x8/alu | 1 | 4 | `Z` | `1` | `H` | `-` | fetch |
| `0E` | `LD C,u8` | x8/lsm | 2 | 8 | `-` | `-` | `-` | `-` | fetch → read(u8->C) |
| `0F` | `RRCA` | x8/rsb | 1 | 4 | `0` | `0` | `0` | `C` | fetch |
| `10` | `STOP` | control/misc | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `11` | `LD DE,u16` | x16/lsm | 3 | 12 | `-` | `-` | `-` | `-` | fetch → read(u16:lower->E) → read(u16:upper->D) |
| `12` | `LD (DE),A` | x8/lsm | 1 | 8 | `-` | `-` | `-` | `-` | fetch → write(A->(DE)) |
| `13` | `INC DE` | x16/alu | 1 | 8 | `-` | `-` | `-` | `-` | fetch(Probably writes to E here) → internal(Probably writes to D here) |
| `14` | `INC D` | x8/alu | 1 | 4 | `Z` | `0` | `H` | `-` | fetch |
| `15` | `DEC D` | x8/alu | 1 | 4 | `Z` | `1` | `H` | `-` | fetch |
| `16` | `LD D,u8` | x8/lsm | 2 | 8 | `-` | `-` | `-` | `-` | fetch → read(u8->D) |
| `17` | `RLA` | x8/rsb | 1 | 4 | `0` | `0` | `0` | `C` | fetch |
| `18` | `JR i8` | control/br | 2 | 12 | `-` | `-` | `-` | `-` | — &nbsp;/&nbsp; **com desvio:** fetch → read(i8) → internal(modify PC) |
| `19` | `ADD HL,DE` | x16/alu | 1 | 8 | `-` | `0` | `H` | `C` | fetch(Probably writes to L here) → internal(Probably writes to H here) |
| `1A` | `LD A,(DE)` | x8/lsm | 1 | 8 | `-` | `-` | `-` | `-` | fetch → read((DE)->A) |
| `1B` | `DEC DE` | x16/alu | 1 | 8 | `-` | `-` | `-` | `-` | fetch(Probably writes to E here) → internal(Probably writes to D here) |
| `1C` | `INC E` | x8/alu | 1 | 4 | `Z` | `0` | `H` | `-` | fetch |
| `1D` | `DEC E` | x8/alu | 1 | 4 | `Z` | `1` | `H` | `-` | fetch |
| `1E` | `LD E,u8` | x8/lsm | 2 | 8 | `-` | `-` | `-` | `-` | fetch → read(u8->E) |
| `1F` | `RRA` | x8/rsb | 1 | 4 | `0` | `0` | `0` | `C` | fetch |
| `20` | `JR NZ,i8` | control/br | 2 | 8 / 12 | `-` | `-` | `-` | `-` | fetch → read(i8) &nbsp;/&nbsp; **com desvio:** fetch → read(i8) → internal(modify PC) |
| `21` | `LD HL,u16` | x16/lsm | 3 | 12 | `-` | `-` | `-` | `-` | fetch → read(u16:lower->L) → read(u16:upper->H) |
| `22` | `LD (HL+),A` | x8/lsm | 1 | 8 | `-` | `-` | `-` | `-` | fetch → write(A->(HL++)) |
| `23` | `INC HL` | x16/alu | 1 | 8 | `-` | `-` | `-` | `-` | fetch(Probably writes to L here) → internal(Probably writes to H here) |
| `24` | `INC H` | x8/alu | 1 | 4 | `Z` | `0` | `H` | `-` | fetch |
| `25` | `DEC H` | x8/alu | 1 | 4 | `Z` | `1` | `H` | `-` | fetch |
| `26` | `LD H,u8` | x8/lsm | 2 | 8 | `-` | `-` | `-` | `-` | fetch → read(u8->H) |
| `27` | `DAA` | x8/alu | 1 | 4 | `Z` | `-` | `0` | `C` | fetch |
| `28` | `JR Z,i8` | control/br | 2 | 8 / 12 | `-` | `-` | `-` | `-` | fetch → read(i8) &nbsp;/&nbsp; **com desvio:** fetch → read(i8) → internal(modify PC) |
| `29` | `ADD HL,HL` | x16/alu | 1 | 8 | `-` | `0` | `H` | `C` | fetch(Probably writes to L here) → internal(Probably writes to H here) |
| `2A` | `LD A,(HL+)` | x8/lsm | 1 | 8 | `-` | `-` | `-` | `-` | fetch → read((HL++)->A) |
| `2B` | `DEC HL` | x16/alu | 1 | 8 | `-` | `-` | `-` | `-` | fetch(Probably writes to L here) → internal(Probably writes to H here) |
| `2C` | `INC L` | x8/alu | 1 | 4 | `Z` | `0` | `H` | `-` | fetch |
| `2D` | `DEC L` | x8/alu | 1 | 4 | `Z` | `1` | `H` | `-` | fetch |
| `2E` | `LD L,u8` | x8/lsm | 2 | 8 | `-` | `-` | `-` | `-` | fetch → read(u8->L) |
| `2F` | `CPL` | x8/alu | 1 | 4 | `-` | `1` | `1` | `-` | fetch |
| `30` | `JR NC,i8` | control/br | 2 | 8 / 12 | `-` | `-` | `-` | `-` | fetch → read(i8) &nbsp;/&nbsp; **com desvio:** fetch → read(i8) → internal(modify PC) |
| `31` | `LD SP,u16` | x16/lsm | 3 | 12 | `-` | `-` | `-` | `-` | fetch → read(u16:lower->SP:lower) → read(u16:upper->SP:upper) |
| `32` | `LD (HL-),A` | x8/lsm | 1 | 8 | `-` | `-` | `-` | `-` | fetch → write(A->(HL--)) |
| `33` | `INC SP` | x16/alu | 1 | 8 | `-` | `-` | `-` | `-` | fetch(Probably writes to SP:lower here) → internal(Probably writes to SP:upper here) |
| `34` | `INC (HL)` | x8/alu | 1 | 12 | `Z` | `0` | `H` | `-` | fetch → read((HL)) → write((HL)) |
| `35` | `DEC (HL)` | x8/alu | 1 | 12 | `Z` | `1` | `H` | `-` | fetch → read((HL)) → write((HL)) |
| `36` | `LD (HL),u8` | x8/lsm | 2 | 12 | `-` | `-` | `-` | `-` | fetch → read(u8) → write((HL)) |
| `37` | `SCF` | x8/alu | 1 | 4 | `-` | `0` | `0` | `1` | fetch |
| `38` | `JR C,i8` | control/br | 2 | 8 / 12 | `-` | `-` | `-` | `-` | fetch → read(i8) &nbsp;/&nbsp; **com desvio:** fetch → read(i8) → internal(modify PC) |
| `39` | `ADD HL,SP` | x16/alu | 1 | 8 | `-` | `0` | `H` | `C` | fetch(Probably writes to L here) → internal(Probably writes to H here) |
| `3A` | `LD A,(HL-)` | x8/lsm | 1 | 8 | `-` | `-` | `-` | `-` | fetch → read((HL--)->A) |
| `3B` | `DEC SP` | x16/alu | 1 | 8 | `-` | `-` | `-` | `-` | fetch(Probably writes to SP:lower here) → internal(Probably writes to SP:upper here) |
| `3C` | `INC A` | x8/alu | 1 | 4 | `Z` | `0` | `H` | `-` | fetch |
| `3D` | `DEC A` | x8/alu | 1 | 4 | `Z` | `1` | `H` | `-` | fetch |
| `3E` | `LD A,u8` | x8/lsm | 2 | 8 | `-` | `-` | `-` | `-` | fetch → read(u8->A) |
| `3F` | `CCF` | x8/alu | 1 | 4 | `-` | `0` | `0` | `C` | fetch |
| `40` | `LD B,B` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `41` | `LD B,C` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `42` | `LD B,D` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `43` | `LD B,E` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `44` | `LD B,H` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `45` | `LD B,L` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `46` | `LD B,(HL)` | x8/lsm | 1 | 8 | `-` | `-` | `-` | `-` | fetch → read((HL)->B) |
| `47` | `LD B,A` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `48` | `LD C,B` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `49` | `LD C,C` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `4A` | `LD C,D` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `4B` | `LD C,E` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `4C` | `LD C,H` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `4D` | `LD C,L` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `4E` | `LD C,(HL)` | x8/lsm | 1 | 8 | `-` | `-` | `-` | `-` | fetch → read((HL)->C) |
| `4F` | `LD C,A` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `50` | `LD D,B` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `51` | `LD D,C` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `52` | `LD D,D` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `53` | `LD D,E` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `54` | `LD D,H` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `55` | `LD D,L` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `56` | `LD D,(HL)` | x8/lsm | 1 | 8 | `-` | `-` | `-` | `-` | fetch → read((HL)->D) |
| `57` | `LD D,A` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `58` | `LD E,B` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `59` | `LD E,C` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `5A` | `LD E,D` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `5B` | `LD E,E` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `5C` | `LD E,H` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `5D` | `LD E,L` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `5E` | `LD E,(HL)` | x8/lsm | 1 | 8 | `-` | `-` | `-` | `-` | fetch → read((HL)->E) |
| `5F` | `LD E,A` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `60` | `LD H,B` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `61` | `LD H,C` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `62` | `LD H,D` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `63` | `LD H,E` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `64` | `LD H,H` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `65` | `LD H,L` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `66` | `LD H,(HL)` | x8/lsm | 1 | 8 | `-` | `-` | `-` | `-` | fetch → read((HL)->H) |
| `67` | `LD H,A` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `68` | `LD L,B` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `69` | `LD L,C` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `6A` | `LD L,D` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `6B` | `LD L,E` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `6C` | `LD L,H` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `6D` | `LD L,L` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `6E` | `LD L,(HL)` | x8/lsm | 1 | 8 | `-` | `-` | `-` | `-` | fetch → read((HL)->L) |
| `6F` | `LD L,A` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `70` | `LD (HL),B` | x8/lsm | 1 | 8 | `-` | `-` | `-` | `-` | fetch → write(B->(HL)) |
| `71` | `LD (HL),C` | x8/lsm | 1 | 8 | `-` | `-` | `-` | `-` | fetch → write(C->(HL)) |
| `72` | `LD (HL),D` | x8/lsm | 1 | 8 | `-` | `-` | `-` | `-` | fetch → write(D->(HL)) |
| `73` | `LD (HL),E` | x8/lsm | 1 | 8 | `-` | `-` | `-` | `-` | fetch → write(E->(HL)) |
| `74` | `LD (HL),H` | x8/lsm | 1 | 8 | `-` | `-` | `-` | `-` | fetch → write(H->(HL)) |
| `75` | `LD (HL),L` | x8/lsm | 1 | 8 | `-` | `-` | `-` | `-` | fetch → write(L->(HL)) |
| `76` | `HALT` | control/misc | 1 | 4 | `-` | `-` | `-` | `-` | fetch(This can actually last forever) |
| `77` | `LD (HL),A` | x8/lsm | 1 | 8 | `-` | `-` | `-` | `-` | fetch → write(A->(HL)) |
| `78` | `LD A,B` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `79` | `LD A,C` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `7A` | `LD A,D` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `7B` | `LD A,E` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `7C` | `LD A,H` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `7D` | `LD A,L` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `7E` | `LD A,(HL)` | x8/lsm | 1 | 8 | `-` | `-` | `-` | `-` | fetch → read((HL)->A) |
| `7F` | `LD A,A` | x8/lsm | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `80` | `ADD A,B` | x8/alu | 1 | 4 | `Z` | `0` | `H` | `C` | fetch |
| `81` | `ADD A,C` | x8/alu | 1 | 4 | `Z` | `0` | `H` | `C` | fetch |
| `82` | `ADD A,D` | x8/alu | 1 | 4 | `Z` | `0` | `H` | `C` | fetch |
| `83` | `ADD A,E` | x8/alu | 1 | 4 | `Z` | `0` | `H` | `C` | fetch |
| `84` | `ADD A,H` | x8/alu | 1 | 4 | `Z` | `0` | `H` | `C` | fetch |
| `85` | `ADD A,L` | x8/alu | 1 | 4 | `Z` | `0` | `H` | `C` | fetch |
| `86` | `ADD A,(HL)` | x8/alu | 1 | 8 | `Z` | `0` | `H` | `C` | fetch → read((HL)) |
| `87` | `ADD A,A` | x8/alu | 1 | 4 | `Z` | `0` | `H` | `C` | fetch |
| `88` | `ADC A,B` | x8/alu | 1 | 4 | `Z` | `0` | `H` | `C` | fetch |
| `89` | `ADC A,C` | x8/alu | 1 | 4 | `Z` | `0` | `H` | `C` | fetch |
| `8A` | `ADC A,D` | x8/alu | 1 | 4 | `Z` | `0` | `H` | `C` | fetch |
| `8B` | `ADC A,E` | x8/alu | 1 | 4 | `Z` | `0` | `H` | `C` | fetch |
| `8C` | `ADC A,H` | x8/alu | 1 | 4 | `Z` | `0` | `H` | `C` | fetch |
| `8D` | `ADC A,L` | x8/alu | 1 | 4 | `Z` | `0` | `H` | `C` | fetch |
| `8E` | `ADC A,(HL)` | x8/alu | 1 | 8 | `Z` | `0` | `H` | `C` | fetch → read((HL)) |
| `8F` | `ADC A,A` | x8/alu | 1 | 4 | `Z` | `0` | `H` | `C` | fetch |
| `90` | `SUB A,B` | x8/alu | 1 | 4 | `Z` | `1` | `H` | `C` | fetch |
| `91` | `SUB A,C` | x8/alu | 1 | 4 | `Z` | `1` | `H` | `C` | fetch |
| `92` | `SUB A,D` | x8/alu | 1 | 4 | `Z` | `1` | `H` | `C` | fetch |
| `93` | `SUB A,E` | x8/alu | 1 | 4 | `Z` | `1` | `H` | `C` | fetch |
| `94` | `SUB A,H` | x8/alu | 1 | 4 | `Z` | `1` | `H` | `C` | fetch |
| `95` | `SUB A,L` | x8/alu | 1 | 4 | `Z` | `1` | `H` | `C` | fetch |
| `96` | `SUB A,(HL)` | x8/alu | 1 | 8 | `Z` | `1` | `H` | `C` | fetch → read((HL)) |
| `97` | `SUB A,A` | x8/alu | 1 | 4 | `Z` | `1` | `H` | `C` | fetch |
| `98` | `SBC A,B` | x8/alu | 1 | 4 | `Z` | `1` | `H` | `C` | fetch |
| `99` | `SBC A,C` | x8/alu | 1 | 4 | `Z` | `1` | `H` | `C` | fetch |
| `9A` | `SBC A,D` | x8/alu | 1 | 4 | `Z` | `1` | `H` | `C` | fetch |
| `9B` | `SBC A,E` | x8/alu | 1 | 4 | `Z` | `1` | `H` | `C` | fetch |
| `9C` | `SBC A,H` | x8/alu | 1 | 4 | `Z` | `1` | `H` | `C` | fetch |
| `9D` | `SBC A,L` | x8/alu | 1 | 4 | `Z` | `1` | `H` | `C` | fetch |
| `9E` | `SBC A,(HL)` | x8/alu | 1 | 8 | `Z` | `1` | `H` | `C` | fetch → read((HL)) |
| `9F` | `SBC A,A` | x8/alu | 1 | 4 | `Z` | `1` | `H` | `C` | fetch |
| `A0` | `AND A,B` | x8/alu | 1 | 4 | `Z` | `0` | `1` | `0` | fetch |
| `A1` | `AND A,C` | x8/alu | 1 | 4 | `Z` | `0` | `1` | `0` | fetch |
| `A2` | `AND A,D` | x8/alu | 1 | 4 | `Z` | `0` | `1` | `0` | fetch |
| `A3` | `AND A,E` | x8/alu | 1 | 4 | `Z` | `0` | `1` | `0` | fetch |
| `A4` | `AND A,H` | x8/alu | 1 | 4 | `Z` | `0` | `1` | `0` | fetch |
| `A5` | `AND A,L` | x8/alu | 1 | 4 | `Z` | `0` | `1` | `0` | fetch |
| `A6` | `AND A,(HL)` | x8/alu | 1 | 8 | `Z` | `0` | `1` | `0` | fetch → read((HL)) |
| `A7` | `AND A,A` | x8/alu | 1 | 4 | `Z` | `0` | `1` | `0` | fetch |
| `A8` | `XOR A,B` | x8/alu | 1 | 4 | `Z` | `0` | `0` | `0` | fetch |
| `A9` | `XOR A,C` | x8/alu | 1 | 4 | `Z` | `0` | `0` | `0` | fetch |
| `AA` | `XOR A,D` | x8/alu | 1 | 4 | `Z` | `0` | `0` | `0` | fetch |
| `AB` | `XOR A,E` | x8/alu | 1 | 4 | `Z` | `0` | `0` | `0` | fetch |
| `AC` | `XOR A,H` | x8/alu | 1 | 4 | `Z` | `0` | `0` | `0` | fetch |
| `AD` | `XOR A,L` | x8/alu | 1 | 4 | `Z` | `0` | `0` | `0` | fetch |
| `AE` | `XOR A,(HL)` | x8/alu | 1 | 8 | `Z` | `0` | `0` | `0` | fetch → read((HL)) |
| `AF` | `XOR A,A` | x8/alu | 1 | 4 | `Z` | `0` | `0` | `0` | fetch |
| `B0` | `OR A,B` | x8/alu | 1 | 4 | `Z` | `0` | `0` | `0` | fetch |
| `B1` | `OR A,C` | x8/alu | 1 | 4 | `Z` | `0` | `0` | `0` | fetch |
| `B2` | `OR A,D` | x8/alu | 1 | 4 | `Z` | `0` | `0` | `0` | fetch |
| `B3` | `OR A,E` | x8/alu | 1 | 4 | `Z` | `0` | `0` | `0` | fetch |
| `B4` | `OR A,H` | x8/alu | 1 | 4 | `Z` | `0` | `0` | `0` | fetch |
| `B5` | `OR A,L` | x8/alu | 1 | 4 | `Z` | `0` | `0` | `0` | fetch |
| `B6` | `OR A,(HL)` | x8/alu | 1 | 8 | `Z` | `0` | `0` | `0` | fetch → read((HL)) |
| `B7` | `OR A,A` | x8/alu | 1 | 4 | `Z` | `0` | `0` | `0` | fetch |
| `B8` | `CP A,B` | x8/alu | 1 | 4 | `Z` | `1` | `H` | `C` | fetch |
| `B9` | `CP A,C` | x8/alu | 1 | 4 | `Z` | `1` | `H` | `C` | fetch |
| `BA` | `CP A,D` | x8/alu | 1 | 4 | `Z` | `1` | `H` | `C` | fetch |
| `BB` | `CP A,E` | x8/alu | 1 | 4 | `Z` | `1` | `H` | `C` | fetch |
| `BC` | `CP A,H` | x8/alu | 1 | 4 | `Z` | `1` | `H` | `C` | fetch |
| `BD` | `CP A,L` | x8/alu | 1 | 4 | `Z` | `1` | `H` | `C` | fetch |
| `BE` | `CP A,(HL)` | x8/alu | 1 | 8 | `Z` | `1` | `H` | `C` | fetch → read((HL)) |
| `BF` | `CP A,A` | x8/alu | 1 | 4 | `Z` | `1` | `H` | `C` | fetch |
| `C0` | `RET NZ` | control/br | 1 | 8 / 20 | `-` | `-` | `-` | `-` | fetch → internal(branch decision?) &nbsp;/&nbsp; **com desvio:** fetch → internal(branch decision?) → read((SP++)->lower) → read((SP++)->upper) → internal(set PC?) |
| `C1` | `POP BC` | x16/lsm | 1 | 12 | `-` | `-` | `-` | `-` | fetch → read((SP++)->C) → read((SP++)->B) |
| `C2` | `JP NZ,u16` | control/br | 3 | 12 / 16 | `-` | `-` | `-` | `-` | fetch → read(u16:lower) → read(u16:upper) &nbsp;/&nbsp; **com desvio:** fetch → read(u16:lower) → read(u16:upper) → internal(branch decision?) |
| `C3` | `JP u16` | control/br | 3 | 16 | `-` | `-` | `-` | `-` | — &nbsp;/&nbsp; **com desvio:** fetch → read(u16:lower) → read(u16:upper) → internal(branch decision?) |
| `C4` | `CALL NZ,u16` | control/br | 3 | 12 / 24 | `-` | `-` | `-` | `-` | fetch → read(u16:lower) → read(u16:upper) &nbsp;/&nbsp; **com desvio:** fetch → read(u16:lower) → read(u16:upper) → internal(branch decision?) → write(PC:upper->(--SP)) → write(PC:lower->(--SP)) |
| `C5` | `PUSH BC` | x16/lsm | 1 | 16 | `-` | `-` | `-` | `-` | fetch → internal → write(B->(--SP)) → write(C->(--SP)) |
| `C6` | `ADD A,u8` | x8/alu | 2 | 8 | `Z` | `0` | `H` | `C` | fetch → read(u8) |
| `C7` | `RST 00h` | control/br | 1 | 16 | `-` | `-` | `-` | `-` | fetch → internal → write(PC:upper->(--SP)) → write(PC:lower->(--SP)) |
| `C8` | `RET Z` | control/br | 1 | 8 / 20 | `-` | `-` | `-` | `-` | fetch → internal(branch decision?) &nbsp;/&nbsp; **com desvio:** fetch → internal(branch decision?) → read((SP++)->lower) → read((SP++)->upper) → internal(set PC?) |
| `C9` | `RET` | control/br | 1 | 16 | `-` | `-` | `-` | `-` | — &nbsp;/&nbsp; **com desvio:** fetch → read((SP++)->lower) → read((SP++)->upper) → internal(set PC?) |
| `CA` | `JP Z,u16` | control/br | 3 | 12 / 16 | `-` | `-` | `-` | `-` | fetch → read(u16:lower) → read(u16:upper) &nbsp;/&nbsp; **com desvio:** fetch → read(u16:lower) → read(u16:upper) → internal(branch decision?) |
| `CB` | `PREFIX CB` | control/misc | 1 | 4 | `-` | `-` | `-` | `-` | fetch(probably fetches twice?) |
| `CC` | `CALL Z,u16` | control/br | 3 | 12 / 24 | `-` | `-` | `-` | `-` | fetch → read(u16:lower) → read(u16:upper) &nbsp;/&nbsp; **com desvio:** fetch → read(u16:lower) → read(u16:upper) → internal(branch decision?) → write(PC:upper->(--SP)) → write(PC:lower->(--SP)) |
| `CD` | `CALL u16` | control/br | 3 | 24 | `-` | `-` | `-` | `-` | — &nbsp;/&nbsp; **com desvio:** fetch → read(u16:lower) → read(u16:upper) → internal(branch decision?) → write(PC:upper->(--SP)) → write(PC:lower->(--SP)) |
| `CE` | `ADC A,u8` | x8/alu | 2 | 8 | `Z` | `0` | `H` | `C` | fetch → read(u8) |
| `CF` | `RST 08h` | control/br | 1 | 16 | `-` | `-` | `-` | `-` | fetch → internal → write(PC:upper->(--SP)) → write(PC:lower->(--SP)) |
| `D0` | `RET NC` | control/br | 1 | 8 / 20 | `-` | `-` | `-` | `-` | fetch → internal(branch decision?) &nbsp;/&nbsp; **com desvio:** fetch → internal(branch decision?) → read((SP++)->lower) → read((SP++)->upper) → internal(set PC?) |
| `D1` | `POP DE` | x16/lsm | 1 | 12 | `-` | `-` | `-` | `-` | fetch → read((SP++)->E) → read((SP++)->D) |
| `D2` | `JP NC,u16` | control/br | 3 | 12 / 16 | `-` | `-` | `-` | `-` | fetch → read(u16:lower) → read(u16:upper) &nbsp;/&nbsp; **com desvio:** fetch → read(u16:lower) → read(u16:upper) → internal(branch decision?) |
| `D3` | `*(inválido)*` | unused | 1 | 0 | `-` | `-` | `-` | `-` | — |
| `D4` | `CALL NC,u16` | control/br | 3 | 12 / 24 | `-` | `-` | `-` | `-` | fetch → read(u16:lower) → read(u16:upper) &nbsp;/&nbsp; **com desvio:** fetch → read(u16:lower) → read(u16:upper) → internal(branch decision?) → write(PC:upper->(--SP)) → write(PC:lower->(--SP)) |
| `D5` | `PUSH DE` | x16/lsm | 1 | 16 | `-` | `-` | `-` | `-` | fetch → internal → write(D->(--SP)) → write(E->(--SP)) |
| `D6` | `SUB A,u8` | x8/alu | 2 | 8 | `Z` | `1` | `H` | `C` | fetch → read(u8) |
| `D7` | `RST 10h` | control/br | 1 | 16 | `-` | `-` | `-` | `-` | fetch → internal → write(PC:upper->(--SP)) → write(PC:lower->(--SP)) |
| `D8` | `RET C` | control/br | 1 | 8 / 20 | `-` | `-` | `-` | `-` | fetch → internal(branch decision?) &nbsp;/&nbsp; **com desvio:** fetch → internal(branch decision?) → read((SP++)->lower) → read((SP++)->upper) → internal(set PC?) |
| `D9` | `RETI` | control/br | 1 | 16 | `-` | `-` | `-` | `-` | — &nbsp;/&nbsp; **com desvio:** fetch → read((SP++)->lower) → read((SP++)->upper) → internal(set PC?) |
| `DA` | `JP C,u16` | control/br | 3 | 12 / 16 | `-` | `-` | `-` | `-` | fetch → read(u16:lower) → read(u16:upper) &nbsp;/&nbsp; **com desvio:** fetch → read(u16:lower) → read(u16:upper) → internal(branch decision?) |
| `DB` | `*(inválido)*` | unused | 1 | 0 | `-` | `-` | `-` | `-` | — |
| `DC` | `CALL C,u16` | control/br | 3 | 12 / 24 | `-` | `-` | `-` | `-` | fetch → read(u16:lower) → read(u16:upper) &nbsp;/&nbsp; **com desvio:** fetch → read(u16:lower) → read(u16:upper) → internal(branch decision?) → write(PC:upper->(--SP)) → write(PC:lower->(--SP)) |
| `DD` | `*(inválido)*` | unused | 1 | 0 | `-` | `-` | `-` | `-` | — |
| `DE` | `SBC A,u8` | x8/alu | 2 | 8 | `Z` | `1` | `H` | `C` | fetch → read(u8) |
| `DF` | `RST 18h` | control/br | 1 | 16 | `-` | `-` | `-` | `-` | fetch → internal → write(PC:upper->(--SP)) → write(PC:lower->(--SP)) |
| `E0` | `LD (FF00+u8),A` | x8/lsm | 2 | 12 | `-` | `-` | `-` | `-` | fetch → read(u8) → write(A->(FF00+u8)) |
| `E1` | `POP HL` | x16/lsm | 1 | 12 | `-` | `-` | `-` | `-` | fetch → read((SP++)->L) → read((SP++)->H) |
| `E2` | `LD (FF00+C),A` | x8/lsm | 1 | 8 | `-` | `-` | `-` | `-` | fetch → write(A->(FF00+C)) |
| `E3` | `*(inválido)*` | unused | 1 | 0 | `-` | `-` | `-` | `-` | — |
| `E4` | `*(inválido)*` | unused | 1 | 0 | `-` | `-` | `-` | `-` | — |
| `E5` | `PUSH HL` | x16/lsm | 1 | 16 | `-` | `-` | `-` | `-` | fetch → internal → write(H->(--SP)) → write(L->(--SP)) |
| `E6` | `AND A,u8` | x8/alu | 2 | 8 | `Z` | `0` | `1` | `0` | fetch → read(u8) |
| `E7` | `RST 20h` | control/br | 1 | 16 | `-` | `-` | `-` | `-` | fetch → internal → write(PC:upper->(--SP)) → write(PC:lower->(--SP)) |
| `E8` | `ADD SP,i8` | x16/alu | 2 | 16 | `0` | `0` | `H` | `C` | fetch → read(i8) → internal(Probably writes to SP:lower here) → write(Probably writes to SP:upper here) |
| `E9` | `JP HL` | control/br | 1 | 4 | `-` | `-` | `-` | `-` | — &nbsp;/&nbsp; **com desvio:** fetch |
| `EA` | `LD (u16),A` | x8/lsm | 3 | 16 | `-` | `-` | `-` | `-` | fetch → read(u16:lower) → read(u16:upper) → write(A->(u16)) |
| `EB` | `*(inválido)*` | unused | 1 | 0 | `-` | `-` | `-` | `-` | — |
| `EC` | `*(inválido)*` | unused | 1 | 0 | `-` | `-` | `-` | `-` | — |
| `ED` | `*(inválido)*` | unused | 1 | 0 | `-` | `-` | `-` | `-` | — |
| `EE` | `XOR A,u8` | x8/alu | 2 | 8 | `Z` | `0` | `0` | `0` | fetch → read(u8) |
| `EF` | `RST 28h` | control/br | 1 | 16 | `-` | `-` | `-` | `-` | fetch → internal → write(PC:upper->(--SP)) → write(PC:lower->(--SP)) |
| `F0` | `LD A,(FF00+u8)` | x8/lsm | 2 | 12 | `-` | `-` | `-` | `-` | fetch → read(u8) → read((FF00+u8)->A) |
| `F1` | `POP AF` | x16/lsm | 1 | 12 | `Z` | `N` | `H` | `C` | fetch → read((SP++)->F) → read((SP++)->A) |
| `F2` | `LD A,(FF00+C)` | x8/lsm | 1 | 8 | `-` | `-` | `-` | `-` | fetch → read((FF00+C)->A) |
| `F3` | `DI` | control/misc | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `F4` | `*(inválido)*` | unused | 1 | 0 | `-` | `-` | `-` | `-` | — |
| `F5` | `PUSH AF` | x16/lsm | 1 | 16 | `-` | `-` | `-` | `-` | fetch → internal → write(A->(--SP)) → write(F->(--SP)) |
| `F6` | `OR A,u8` | x8/alu | 2 | 8 | `Z` | `0` | `0` | `0` | fetch → read(u8) |
| `F7` | `RST 30h` | control/br | 1 | 16 | `-` | `-` | `-` | `-` | fetch → internal → write(PC:upper->(--SP)) → write(PC:lower->(--SP)) |
| `F8` | `LD HL,SP+i8` | x16/alu | 2 | 12 | `0` | `0` | `H` | `C` | fetch → read(i8) → internal |
| `F9` | `LD SP,HL` | x16/lsm | 1 | 8 | `-` | `-` | `-` | `-` | fetch → internal |
| `FA` | `LD A,(u16)` | x8/lsm | 3 | 16 | `-` | `-` | `-` | `-` | fetch → read(u16:lower) → read(u16:upper) → read((u16)->A) |
| `FB` | `EI` | control/misc | 1 | 4 | `-` | `-` | `-` | `-` | fetch |
| `FC` | `*(inválido)*` | unused | 1 | 0 | `-` | `-` | `-` | `-` | — |
| `FD` | `*(inválido)*` | unused | 1 | 0 | `-` | `-` | `-` | `-` | — |
| `FE` | `CP A,u8` | x8/alu | 2 | 8 | `Z` | `1` | `H` | `C` | fetch → read(u8) |
| `FF` | `RST 38h` | control/br | 1 | 16 | `-` | `-` | `-` | `-` | fetch → internal → write(PC:upper->(--SP)) → write(PC:lower->(--SP)) |

---

## Opcodes com prefixo `CB`

Todos têm 2 bytes (o `CB` mais o opcode) e operam sobre registrador ou `(HL)`.

| Opcode | Instrução | Grupo | Bytes | T-cycles | Z | N | H | C | M-cycles (passo a passo) |
|---|---|---|---|---|---|---|---|---|---|
| `CB 00` | `RLC B` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 01` | `RLC C` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 02` | `RLC D` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 03` | `RLC E` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 04` | `RLC H` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 05` | `RLC L` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 06` | `RLC (HL)` | x8/rsb | 2 | 16 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch → read((HL)) → write((HL)) |
| `CB 07` | `RLC A` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 08` | `RRC B` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 09` | `RRC C` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 0A` | `RRC D` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 0B` | `RRC E` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 0C` | `RRC H` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 0D` | `RRC L` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 0E` | `RRC (HL)` | x8/rsb | 2 | 16 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch → read((HL)) → write((HL)) |
| `CB 0F` | `RRC A` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 10` | `RL B` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 11` | `RL C` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 12` | `RL D` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 13` | `RL E` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 14` | `RL H` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 15` | `RL L` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 16` | `RL (HL)` | x8/rsb | 2 | 16 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch → read((HL)) → write((HL)) |
| `CB 17` | `RL A` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 18` | `RR B` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 19` | `RR C` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 1A` | `RR D` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 1B` | `RR E` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 1C` | `RR H` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 1D` | `RR L` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 1E` | `RR (HL)` | x8/rsb | 2 | 16 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch → read((HL)) → write((HL)) |
| `CB 1F` | `RR A` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 20` | `SLA B` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 21` | `SLA C` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 22` | `SLA D` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 23` | `SLA E` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 24` | `SLA H` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 25` | `SLA L` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 26` | `SLA (HL)` | x8/rsb | 2 | 16 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch → read((HL)) → write((HL)) |
| `CB 27` | `SLA A` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 28` | `SRA B` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 29` | `SRA C` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 2A` | `SRA D` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 2B` | `SRA E` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 2C` | `SRA H` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 2D` | `SRA L` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 2E` | `SRA (HL)` | x8/rsb | 2 | 16 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch → read((HL)) → write((HL)) |
| `CB 2F` | `SRA A` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 30` | `SWAP B` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `0` | fetch((0xCB)) → fetch |
| `CB 31` | `SWAP C` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `0` | fetch((0xCB)) → fetch |
| `CB 32` | `SWAP D` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `0` | fetch((0xCB)) → fetch |
| `CB 33` | `SWAP E` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `0` | fetch((0xCB)) → fetch |
| `CB 34` | `SWAP H` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `0` | fetch((0xCB)) → fetch |
| `CB 35` | `SWAP L` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `0` | fetch((0xCB)) → fetch |
| `CB 36` | `SWAP (HL)` | x8/rsb | 2 | 16 | `Z` | `0` | `0` | `0` | fetch((0xCB)) → fetch → read((HL)) → write((HL)) |
| `CB 37` | `SWAP A` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `0` | fetch((0xCB)) → fetch |
| `CB 38` | `SRL B` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 39` | `SRL C` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 3A` | `SRL D` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 3B` | `SRL E` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 3C` | `SRL H` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 3D` | `SRL L` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 3E` | `SRL (HL)` | x8/rsb | 2 | 16 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch → read((HL)) → write((HL)) |
| `CB 3F` | `SRL A` | x8/rsb | 2 | 8 | `Z` | `0` | `0` | `C` | fetch((0xCB)) → fetch |
| `CB 40` | `BIT 0,B` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 41` | `BIT 0,C` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 42` | `BIT 0,D` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 43` | `BIT 0,E` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 44` | `BIT 0,H` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 45` | `BIT 0,L` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 46` | `BIT 0,(HL)` | x8/rsb | 2 | 12 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch → read((HL)) |
| `CB 47` | `BIT 0,A` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 48` | `BIT 1,B` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 49` | `BIT 1,C` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 4A` | `BIT 1,D` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 4B` | `BIT 1,E` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 4C` | `BIT 1,H` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 4D` | `BIT 1,L` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 4E` | `BIT 1,(HL)` | x8/rsb | 2 | 12 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch → read((HL)) |
| `CB 4F` | `BIT 1,A` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 50` | `BIT 2,B` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 51` | `BIT 2,C` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 52` | `BIT 2,D` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 53` | `BIT 2,E` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 54` | `BIT 2,H` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 55` | `BIT 2,L` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 56` | `BIT 2,(HL)` | x8/rsb | 2 | 12 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch → read((HL)) |
| `CB 57` | `BIT 2,A` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 58` | `BIT 3,B` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 59` | `BIT 3,C` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 5A` | `BIT 3,D` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 5B` | `BIT 3,E` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 5C` | `BIT 3,H` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 5D` | `BIT 3,L` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 5E` | `BIT 3,(HL)` | x8/rsb | 2 | 12 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch → read((HL)) |
| `CB 5F` | `BIT 3,A` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 60` | `BIT 4,B` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 61` | `BIT 4,C` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 62` | `BIT 4,D` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 63` | `BIT 4,E` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 64` | `BIT 4,H` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 65` | `BIT 4,L` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 66` | `BIT 4,(HL)` | x8/rsb | 2 | 12 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch → read((HL)) |
| `CB 67` | `BIT 4,A` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 68` | `BIT 5,B` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 69` | `BIT 5,C` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 6A` | `BIT 5,D` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 6B` | `BIT 5,E` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 6C` | `BIT 5,H` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 6D` | `BIT 5,L` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 6E` | `BIT 5,(HL)` | x8/rsb | 2 | 12 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch → read((HL)) |
| `CB 6F` | `BIT 5,A` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 70` | `BIT 6,B` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 71` | `BIT 6,C` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 72` | `BIT 6,D` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 73` | `BIT 6,E` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 74` | `BIT 6,H` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 75` | `BIT 6,L` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 76` | `BIT 6,(HL)` | x8/rsb | 2 | 12 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch → read((HL)) |
| `CB 77` | `BIT 6,A` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 78` | `BIT 7,B` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 79` | `BIT 7,C` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 7A` | `BIT 7,D` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 7B` | `BIT 7,E` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 7C` | `BIT 7,H` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 7D` | `BIT 7,L` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 7E` | `BIT 7,(HL)` | x8/rsb | 2 | 12 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch → read((HL)) |
| `CB 7F` | `BIT 7,A` | x8/rsb | 2 | 8 | `Z` | `0` | `1` | `-` | fetch((0xCB)) → fetch |
| `CB 80` | `RES 0,B` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB 81` | `RES 0,C` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB 82` | `RES 0,D` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB 83` | `RES 0,E` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB 84` | `RES 0,H` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB 85` | `RES 0,L` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB 86` | `RES 0,(HL)` | x8/rsb | 2 | 16 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch → read((HL)) → write((HL)) |
| `CB 87` | `RES 0,A` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB 88` | `RES 1,B` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB 89` | `RES 1,C` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB 8A` | `RES 1,D` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB 8B` | `RES 1,E` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB 8C` | `RES 1,H` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB 8D` | `RES 1,L` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB 8E` | `RES 1,(HL)` | x8/rsb | 2 | 16 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch → read((HL)) → write((HL)) |
| `CB 8F` | `RES 1,A` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB 90` | `RES 2,B` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB 91` | `RES 2,C` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB 92` | `RES 2,D` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB 93` | `RES 2,E` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB 94` | `RES 2,H` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB 95` | `RES 2,L` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB 96` | `RES 2,(HL)` | x8/rsb | 2 | 16 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch → read((HL)) → write((HL)) |
| `CB 97` | `RES 2,A` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB 98` | `RES 3,B` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB 99` | `RES 3,C` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB 9A` | `RES 3,D` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB 9B` | `RES 3,E` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB 9C` | `RES 3,H` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB 9D` | `RES 3,L` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB 9E` | `RES 3,(HL)` | x8/rsb | 2 | 16 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch → read((HL)) → write((HL)) |
| `CB 9F` | `RES 3,A` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB A0` | `RES 4,B` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB A1` | `RES 4,C` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB A2` | `RES 4,D` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB A3` | `RES 4,E` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB A4` | `RES 4,H` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB A5` | `RES 4,L` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB A6` | `RES 4,(HL)` | x8/rsb | 2 | 16 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch → read((HL)) → write((HL)) |
| `CB A7` | `RES 4,A` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB A8` | `RES 5,B` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB A9` | `RES 5,C` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB AA` | `RES 5,D` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB AB` | `RES 5,E` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB AC` | `RES 5,H` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB AD` | `RES 5,L` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB AE` | `RES 5,(HL)` | x8/rsb | 2 | 16 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch → read((HL)) → write((HL)) |
| `CB AF` | `RES 5,A` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB B0` | `RES 6,B` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB B1` | `RES 6,C` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB B2` | `RES 6,D` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB B3` | `RES 6,E` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB B4` | `RES 6,H` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB B5` | `RES 6,L` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB B6` | `RES 6,(HL)` | x8/rsb | 2 | 16 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch → read((HL)) → write((HL)) |
| `CB B7` | `RES 6,A` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB B8` | `RES 7,B` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB B9` | `RES 7,C` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB BA` | `RES 7,D` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB BB` | `RES 7,E` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB BC` | `RES 7,H` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB BD` | `RES 7,L` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB BE` | `RES 7,(HL)` | x8/rsb | 2 | 16 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch → read((HL)) → write((HL)) |
| `CB BF` | `RES 7,A` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB C0` | `SET 0,B` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB C1` | `SET 0,C` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB C2` | `SET 0,D` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB C3` | `SET 0,E` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB C4` | `SET 0,H` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB C5` | `SET 0,L` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB C6` | `SET 0,(HL)` | x8/rsb | 2 | 16 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch → read((HL)) → write((HL)) |
| `CB C7` | `SET 0,A` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB C8` | `SET 1,B` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB C9` | `SET 1,C` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB CA` | `SET 1,D` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB CB` | `SET 1,E` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB CC` | `SET 1,H` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB CD` | `SET 1,L` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB CE` | `SET 1,(HL)` | x8/rsb | 2 | 16 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch → read((HL)) → write((HL)) |
| `CB CF` | `SET 1,A` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB D0` | `SET 2,B` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB D1` | `SET 2,C` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB D2` | `SET 2,D` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB D3` | `SET 2,E` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB D4` | `SET 2,H` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB D5` | `SET 2,L` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB D6` | `SET 2,(HL)` | x8/rsb | 2 | 16 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch → read((HL)) → write((HL)) |
| `CB D7` | `SET 2,A` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB D8` | `SET 3,B` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB D9` | `SET 3,C` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB DA` | `SET 3,D` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB DB` | `SET 3,E` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB DC` | `SET 3,H` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB DD` | `SET 3,L` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB DE` | `SET 3,(HL)` | x8/rsb | 2 | 16 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch → read((HL)) → write((HL)) |
| `CB DF` | `SET 3,A` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB E0` | `SET 4,B` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB E1` | `SET 4,C` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB E2` | `SET 4,D` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB E3` | `SET 4,E` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB E4` | `SET 4,H` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB E5` | `SET 4,L` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB E6` | `SET 4,(HL)` | x8/rsb | 2 | 16 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch → read((HL)) → write((HL)) |
| `CB E7` | `SET 4,A` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB E8` | `SET 5,B` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB E9` | `SET 5,C` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB EA` | `SET 5,D` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB EB` | `SET 5,E` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB EC` | `SET 5,H` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB ED` | `SET 5,L` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB EE` | `SET 5,(HL)` | x8/rsb | 2 | 16 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch → read((HL)) → write((HL)) |
| `CB EF` | `SET 5,A` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB F0` | `SET 6,B` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB F1` | `SET 6,C` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB F2` | `SET 6,D` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB F3` | `SET 6,E` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB F4` | `SET 6,H` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB F5` | `SET 6,L` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB F6` | `SET 6,(HL)` | x8/rsb | 2 | 16 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch → read((HL)) → write((HL)) |
| `CB F7` | `SET 6,A` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB F8` | `SET 7,B` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB F9` | `SET 7,C` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB FA` | `SET 7,D` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB FB` | `SET 7,E` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB FC` | `SET 7,H` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB FD` | `SET 7,L` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
| `CB FE` | `SET 7,(HL)` | x8/rsb | 2 | 16 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch → read((HL)) → write((HL)) |
| `CB FF` | `SET 7,A` | x8/rsb | 2 | 8 | `-` | `-` | `-` | `-` | fetch((0xCB)) → fetch |
