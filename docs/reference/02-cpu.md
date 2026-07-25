# CPU SM83 — registradores, flags e instruções

> **Fonte de verdade deste projeto (regra R1 do CLAUDE.md).**
> Cópia do [Pan Docs](https://gbdev.io/pandocs/) (domínio público, CC0),
> fixada no commit [`fe246067b695`](https://github.com/gbdev/pandocs/tree/fe246067b695b5404a4a6a47efb4fd6d921ececb).
> Não editar à mão: regenerado por `scripts/fetch-reference-docs.sh`.

Cobre o ROADMAP 1.1 e 1.4–1.11. A seção de comparação com o Z80 é leitura obrigatória antes de qualquer opcode: é o catálogo das divergências que a regra R1 existe para evitar.

**Nesta página:**

- CPU Registers and Flags
- CPU Instruction Set
- CPU Comparison with Z80

---

<!-- fonte: src/CPU_Registers_and_Flags.md @ fe246067b695 -->

## CPU registers and flags

### Registers

16-bit |Hi |Lo | Name/Function
-------|---|---|--------------
   AF  | A | - | Accumulator & Flags
   BC  | B | C | BC
   DE  | D | E | DE
   HL  | H | L | HL
   SP  | - | - | Stack Pointer
   PC  | - | - | Program Counter/Pointer

As shown above, most registers can be accessed either as one 16-bit
register, or as two separate 8-bit registers.

### The Flags Register (lower 8 bits of AF register)

Bit | Name | Explanation
----|------|-------
  7 |   z  | Zero flag
  6 |   n  | Subtraction flag (BCD)
  5 |   h  | Half Carry flag (BCD)
  4 |   c  | Carry flag

Contains information about the result of the most recent instruction that has affected
flags.

### The Zero Flag (Z)

This bit is set if and only if the result of an operation is zero. Used by conditional jumps.

### The Carry Flag (C, or Cy)

Is set in these cases:
- When the result of an 8-bit addition is higher than $FF.
- When the result of a 16-bit addition is higher than $FFFF.
- When the result of a subtraction or comparison
is lower than zero (like in Z80 and x86 CPUs, but unlike in
65XX and ARM CPUs).
- When a rotate/shift operation shifts out a "1" bit.

Used by conditional jumps and
instructions such as ADC, SBC, RL, RLA, etc.

### The BCD Flags (N, H)

These flags are used by the DAA instruction only. N indicates
whether the previous instruction has been a subtraction,
and H indicates carry for the lower 4 bits of the result. DAA also uses the C flag,
which must indicate carry for the upper 4 bits. After adding/subtracting two
BCD numbers, DAA is used to convert the result to BCD format. BCD
numbers range from $00 to $99 rather than $00 to $FF. Because only two flags
(C and H) exist to indicate carry-outs of BCD digits, DAA is ineffective for
16-bit operations (which have 4 digits), and use for INC/DEC operations
(which do not affect C-flag) has limits.

_Fonte desta seção: [`src/CPU_Registers_and_Flags.md`](https://github.com/gbdev/pandocs/blob/fe246067b695b5404a4a6a47efb4fd6d921ececb/src/CPU_Registers_and_Flags.md)_


---

<!-- fonte: src/CPU_Instruction_Set.md @ fe246067b695 -->

## CPU Instruction Set

:::tip

If you are looking for textual explanations of what each each instruction does, please read [gbz80(7)](https://rgbds.gbdev.io/docs/gbz80.7); if you want a compact reference card/cheat sheet of each opcode and its flag effects, please consult [the optables](https://gbdev.io/gb-opcodes/optables) (whose [octal view](https://gbdev.io/gb-opcodes/optables/octal) makes most encoding patterns more apparent).

:::

<style>table td { padding: 3px 10px; overflow-wrap: break-word; }</style>

The Game Boy's SM83 processor possesses a <abbr title="Complex Instruction Set Computer">CISC</abbr>, variable-length instruction set.
This page attempts to shed some light on how the CPU decodes the raw bytes fed into it into instructions.

The first byte of each instruction is typically called the "opcode" (for "operation code").
By noticing that some instructions perform identical operations but with different parameters, they can be grouped together; for example, `inc bc`, `inc de`, `inc hl`, and `inc sp` differ only in what 16-bit register they modify.

In each table, one line represents one such grouping.
Since many groupings have some variation, the variation has to be encoded in the instruction; for example, the above four instructions will be collectively referred to as `inc r16`.
Here are the possible placeholders and their values:

**`r8`** — layout de bits (8 bits):

| Bits | Campo |
|---|---|
| 0 | <code>b</code> |
| 1 | <code>c</code> |
| 2 | <code>d</code> |
| 3 | <code>e</code> |
| 4 | <code>h</code> |
| 5 | <code>l</code> |
| 6 | <code>[hl]</code> |
| 7 | <code>a</code> |
| 0 | <code>bc</code> |
| 1 | <code>de</code> |
| 2 | <code>hl</code> |
| 3 | <code>sp</code> |
| 0 | <code>bc</code> |
| 1 | <code>de</code> |
| 2 | <code>hl</code> |
| 3 | <code>af</code> |
| 0 | <code>bc</code> |
| 1 | <code>de</code> |
| 2 | <code>hl+</code> |
| 3 | <code>hl-</code> |
| 0 | <code>nz</code> |
| 1 | <code>z</code> |
| 2 | <code>nc</code> |
| 3 | <code>c</code> |
| 0-7 | A 3-bit bit index |
| 0-7 | <code>rst</code>'s target address, divided by 8 |
| 0-7 | The following byte |
| 0-7 | The following two bytes, in little-endian order |


These last two are a little special: if they are present in the instruction's mnemonic, it means that the instruction is 1 (`imm8`) / 2 (`imm16`) extra bytes long.

:::tip

`[hl+]` and `[hl-]` can also be notated `[hli]` and `[hld]` respectively (as in **i**ncrement and **d**ecrement).

:::

Groupings have been loosely associated based on what they do into separate tables; those have no particular ordering, and are purely for readability and convenience.
Finally, the instruction "families" have been further grouped into four "blocks", differentiated by the first two bits of the opcode.

### Block 0

**`<code>nop</code>`** — layout de bits (8 bits):

| Bits | Campo |
|---|---|
| 7 | 0 |
| 6 | 0 |
| 5 | 0 |
| 4 | 0 |
| 3 | 0 |
| 2 | 0 |
| 1 | 0 |
| 0 | 0 |


**`<code>ld r16, imm16</code>`** — layout de bits (8 bits):

| Bits | Campo |
|---|---|
| 7 | 0 |
| 6 | 0 |
| 5-4 | Dest (r16) |
| 3 | 0 |
| 2 | 0 |
| 1 | 0 |
| 0 | 1 |
| 7 | 0 |
| 6 | 0 |
| 5-4 | Dest (r16mem) |
| 3 | 0 |
| 2 | 0 |
| 1 | 1 |
| 0 | 0 |
| 7 | 0 |
| 6 | 0 |
| 5-4 | Source (r16mem) |
| 3 | 1 |
| 2 | 0 |
| 1 | 1 |
| 0 | 0 |
| 7 | 0 |
| 6 | 0 |
| 5 | 0 |
| 4 | 0 |
| 3 | 1 |
| 2 | 0 |
| 1 | 0 |
| 0 | 0 |


**`<code>inc r16</code>`** — layout de bits (8 bits):

| Bits | Campo |
|---|---|
| 7 | 0 |
| 6 | 0 |
| 5-4 | Operand (r16) |
| 3 | 0 |
| 2 | 0 |
| 1 | 1 |
| 0 | 1 |
| 7 | 0 |
| 6 | 0 |
| 5-4 | Operand (r16) |
| 3 | 1 |
| 2 | 0 |
| 1 | 1 |
| 0 | 1 |
| 7 | 0 |
| 6 | 0 |
| 5-4 | Operand (r16) |
| 3 | 1 |
| 2 | 0 |
| 1 | 0 |
| 0 | 1 |


**`<code>inc r8</code>`** — layout de bits (8 bits):

| Bits | Campo |
|---|---|
| 7 | 0 |
| 6 | 0 |
| 5-3 | Operand (r8) |
| 2 | 1 |
| 1 | 0 |
| 0 | 0 |
| 7 | 0 |
| 6 | 0 |
| 5-3 | Operand (r8) |
| 2 | 1 |
| 1 | 0 |
| 0 | 1 |


**`<code>ld r8, imm8</code>`** — layout de bits (8 bits):

| Bits | Campo |
|---|---|
| 7 | 0 |
| 6 | 0 |
| 5-3 | Dest (r8) |
| 2 | 1 |
| 1 | 1 |
| 0 | 0 |


**`<code>rlca</code>`** — layout de bits (8 bits):

| Bits | Campo |
|---|---|
| 7 | 0 |
| 6 | 0 |
| 5 | 0 |
| 4 | 0 |
| 3 | 0 |
| 2 | 1 |
| 1 | 1 |
| 0 | 1 |
| 7 | 0 |
| 6 | 0 |
| 5 | 0 |
| 4 | 0 |
| 3 | 1 |
| 2 | 1 |
| 1 | 1 |
| 0 | 1 |
| 7 | 0 |
| 6 | 0 |
| 5 | 0 |
| 4 | 1 |
| 3 | 0 |
| 2 | 1 |
| 1 | 1 |
| 0 | 1 |
| 7 | 0 |
| 6 | 0 |
| 5 | 0 |
| 4 | 1 |
| 3 | 1 |
| 2 | 1 |
| 1 | 1 |
| 0 | 1 |
| 7 | 0 |
| 6 | 0 |
| 5 | 1 |
| 4 | 0 |
| 3 | 0 |
| 2 | 1 |
| 1 | 1 |
| 0 | 1 |
| 7 | 0 |
| 6 | 0 |
| 5 | 1 |
| 4 | 0 |
| 3 | 1 |
| 2 | 1 |
| 1 | 1 |
| 0 | 1 |
| 7 | 0 |
| 6 | 0 |
| 5 | 1 |
| 4 | 1 |
| 3 | 0 |
| 2 | 1 |
| 1 | 1 |
| 0 | 1 |
| 7 | 0 |
| 6 | 0 |
| 5 | 1 |
| 4 | 1 |
| 3 | 1 |
| 2 | 1 |
| 1 | 1 |
| 0 | 1 |


**`<code>jr imm8</code>`** — layout de bits (8 bits):

| Bits | Campo |
|---|---|
| 7 | 0 |
| 6 | 0 |
| 5 | 0 |
| 4 | 1 |
| 3 | 1 |
| 2 | 0 |
| 1 | 0 |
| 0 | 0 |
| 7 | 0 |
| 6 | 0 |
| 5 | 1 |
| 4-3 | Condition (cond) |
| 2 | 0 |
| 1 | 0 |
| 0 | 0 |


**`<code>stop</code>`** — layout de bits (8 bits):

| Bits | Campo |
|---|---|
| 7 | 0 |
| 6 | 0 |
| 5 | 0 |
| 4 | 1 |
| 3 | 0 |
| 2 | 0 |
| 1 | 0 |
| 0 | 0 |


[`stop`](https://gbdev.io/pandocs/single.html#using-the-stop-instruction) is often considered a **two-byte** instruction, though [the second byte is not always ignored](https://gist.github.com/SonoSooS/c0055300670d678b5ae8433e20bea595#nop-and-stop).

### Block 1: 8-bit register-to-register loads

**`<code>ld r8, r8</code>`** — layout de bits (8 bits):

| Bits | Campo |
|---|---|
| 7 | 0 |
| 6 | 1 |
| 5-3 | Dest (r8) |
| 2-0 | Source (r8) |


**Exception**: trying to encode `ld [hl], [hl]` instead yields [the `halt` instruction](https://gbdev.io/pandocs/single.html#halt):

**`<code>halt</code>`** — layout de bits (8 bits):

| Bits | Campo |
|---|---|
| 7 | 0 |
| 6 | 1 |
| 5 | 1 |
| 4 | 1 |
| 3 | 0 |
| 2 | 1 |
| 1 | 1 |
| 0 | 0 |


### Block 2: 8-bit arithmetic

**`<code>add a, r8</code>`** — layout de bits (8 bits):

| Bits | Campo |
|---|---|
| 7 | 1 |
| 6 | 0 |
| 5 | 0 |
| 4 | 0 |
| 3 | 0 |
| 2-0 | Operand (r8) |
| 7 | 1 |
| 6 | 0 |
| 5 | 0 |
| 4 | 0 |
| 3 | 1 |
| 2-0 | Operand (r8) |
| 7 | 1 |
| 6 | 0 |
| 5 | 0 |
| 4 | 1 |
| 3 | 0 |
| 2-0 | Operand (r8) |
| 7 | 1 |
| 6 | 0 |
| 5 | 0 |
| 4 | 1 |
| 3 | 1 |
| 2-0 | Operand (r8) |
| 7 | 1 |
| 6 | 0 |
| 5 | 1 |
| 4 | 0 |
| 3 | 0 |
| 2-0 | Operand (r8) |
| 7 | 1 |
| 6 | 0 |
| 5 | 1 |
| 4 | 0 |
| 3 | 1 |
| 2-0 | Operand (r8) |
| 7 | 1 |
| 6 | 0 |
| 5 | 1 |
| 4 | 1 |
| 3 | 0 |
| 2-0 | Operand (r8) |
| 7 | 1 |
| 6 | 0 |
| 5 | 1 |
| 4 | 1 |
| 3 | 1 |
| 2-0 | Operand (r8) |


### Block 3

**`<code>add a, imm8</code>`** — layout de bits (8 bits):

| Bits | Campo |
|---|---|
| 7 | 1 |
| 6 | 1 |
| 5 | 0 |
| 4 | 0 |
| 3 | 0 |
| 2 | 1 |
| 1 | 1 |
| 0 | 0 |
| 7 | 1 |
| 6 | 1 |
| 5 | 0 |
| 4 | 0 |
| 3 | 1 |
| 2 | 1 |
| 1 | 1 |
| 0 | 0 |
| 7 | 1 |
| 6 | 1 |
| 5 | 0 |
| 4 | 1 |
| 3 | 0 |
| 2 | 1 |
| 1 | 1 |
| 0 | 0 |
| 7 | 1 |
| 6 | 1 |
| 5 | 0 |
| 4 | 1 |
| 3 | 1 |
| 2 | 1 |
| 1 | 1 |
| 0 | 0 |
| 7 | 1 |
| 6 | 1 |
| 5 | 1 |
| 4 | 0 |
| 3 | 0 |
| 2 | 1 |
| 1 | 1 |
| 0 | 0 |
| 7 | 1 |
| 6 | 1 |
| 5 | 1 |
| 4 | 0 |
| 3 | 1 |
| 2 | 1 |
| 1 | 1 |
| 0 | 0 |
| 7 | 1 |
| 6 | 1 |
| 5 | 1 |
| 4 | 1 |
| 3 | 0 |
| 2 | 1 |
| 1 | 1 |
| 0 | 0 |
| 7 | 1 |
| 6 | 1 |
| 5 | 1 |
| 4 | 1 |
| 3 | 1 |
| 2 | 1 |
| 1 | 1 |
| 0 | 0 |


**`<code>ret cond</code>`** — layout de bits (8 bits):

| Bits | Campo |
|---|---|
| 7 | 1 |
| 6 | 1 |
| 5 | 0 |
| 4-3 | Condition (cond) |
| 2 | 0 |
| 1 | 0 |
| 0 | 0 |
| 7 | 1 |
| 6 | 1 |
| 5 | 0 |
| 4 | 0 |
| 3 | 1 |
| 2 | 0 |
| 1 | 0 |
| 0 | 1 |
| 7 | 1 |
| 6 | 1 |
| 5 | 0 |
| 4 | 1 |
| 3 | 1 |
| 2 | 0 |
| 1 | 0 |
| 0 | 1 |
| 7 | 1 |
| 6 | 1 |
| 5 | 0 |
| 4-3 | Condition (cond) |
| 2 | 0 |
| 1 | 1 |
| 0 | 0 |
| 7 | 1 |
| 6 | 1 |
| 5 | 0 |
| 4 | 0 |
| 3 | 0 |
| 2 | 0 |
| 1 | 1 |
| 0 | 1 |
| 7 | 1 |
| 6 | 1 |
| 5 | 1 |
| 4 | 0 |
| 3 | 1 |
| 2 | 0 |
| 1 | 0 |
| 0 | 1 |
| 7 | 1 |
| 6 | 1 |
| 5 | 0 |
| 4-3 | Condition (cond) |
| 2 | 1 |
| 1 | 0 |
| 0 | 0 |
| 7 | 1 |
| 6 | 1 |
| 5 | 0 |
| 4 | 0 |
| 3 | 1 |
| 2 | 1 |
| 1 | 0 |
| 0 | 1 |
| 7 | 1 |
| 6 | 1 |
| 5-3 | Target (tgt3) |
| 2 | 1 |
| 1 | 1 |
| 0 | 1 |


**`<code>pop r16stk</code>`** — layout de bits (8 bits):

| Bits | Campo |
|---|---|
| 7 | 1 |
| 6 | 1 |
| 5-4 | Register (r16stk) |
| 3 | 0 |
| 2 | 0 |
| 1 | 0 |
| 0 | 1 |
| 7 | 1 |
| 6 | 1 |
| 5-4 | Register (r16stk) |
| 3 | 0 |
| 2 | 1 |
| 1 | 0 |
| 0 | 1 |


**`Prefix (see block below)`** — layout de bits (8 bits):

| Bits | Campo |
|---|---|
| 7 | 1 |
| 6 | 1 |
| 5 | 0 |
| 4 | 0 |
| 3 | 1 |
| 2 | 0 |
| 1 | 1 |
| 0 | 1 |


**`<code>ldh [c], a</code>`** — layout de bits (8 bits):

| Bits | Campo |
|---|---|
| 7 | 1 |
| 6 | 1 |
| 5 | 1 |
| 4 | 0 |
| 3 | 0 |
| 2 | 0 |
| 1 | 1 |
| 0 | 0 |
| 7 | 1 |
| 6 | 1 |
| 5 | 1 |
| 4 | 0 |
| 3 | 0 |
| 2 | 0 |
| 1 | 0 |
| 0 | 0 |
| 7 | 1 |
| 6 | 1 |
| 5 | 1 |
| 4 | 0 |
| 3 | 1 |
| 2 | 0 |
| 1 | 1 |
| 0 | 0 |
| 7 | 1 |
| 6 | 1 |
| 5 | 1 |
| 4 | 1 |
| 3 | 0 |
| 2 | 0 |
| 1 | 1 |
| 0 | 0 |
| 7 | 1 |
| 6 | 1 |
| 5 | 1 |
| 4 | 1 |
| 3 | 0 |
| 2 | 0 |
| 1 | 0 |
| 0 | 0 |
| 7 | 1 |
| 6 | 1 |
| 5 | 1 |
| 4 | 1 |
| 3 | 1 |
| 2 | 0 |
| 1 | 1 |
| 0 | 0 |


**`<code>add sp, imm8</code>`** — layout de bits (8 bits):

| Bits | Campo |
|---|---|
| 7 | 1 |
| 6 | 1 |
| 5 | 1 |
| 4 | 0 |
| 3 | 1 |
| 2 | 0 |
| 1 | 0 |
| 0 | 0 |
| 7 | 1 |
| 6 | 1 |
| 5 | 1 |
| 4 | 1 |
| 3 | 1 |
| 2 | 0 |
| 1 | 0 |
| 0 | 0 |
| 7 | 1 |
| 6 | 1 |
| 5 | 1 |
| 4 | 1 |
| 3 | 1 |
| 2 | 0 |
| 1 | 0 |
| 0 | 1 |


**`<code>di</code>`** — layout de bits (8 bits):

| Bits | Campo |
|---|---|
| 7 | 1 |
| 6 | 1 |
| 5 | 1 |
| 4 | 1 |
| 3 | 0 |
| 2 | 0 |
| 1 | 1 |
| 0 | 1 |
| 7 | 1 |
| 6 | 1 |
| 5 | 1 |
| 4 | 1 |
| 3 | 1 |
| 2 | 0 |
| 1 | 1 |
| 0 | 1 |


The following opcodes are **invalid**, and [hard-lock the CPU](https://gist.github.com/SonoSooS/c0055300670d678b5ae8433e20bea595#opcode-holes-not-implemented-opcodes) until the console is powered off: $D3, $DB, $DD, $E3, $E4, $EB, $EC, $ED, $F4, $FC, and $FD.

### $CB prefix instructions

**`<code>rlc r8</code>`** — layout de bits (8 bits):

| Bits | Campo |
|---|---|
| 7 | 0 |
| 6 | 0 |
| 5 | 0 |
| 4 | 0 |
| 3 | 0 |
| 2-0 | Operand (r8) |
| 7 | 0 |
| 6 | 0 |
| 5 | 0 |
| 4 | 0 |
| 3 | 1 |
| 2-0 | Operand (r8) |
| 7 | 0 |
| 6 | 0 |
| 5 | 0 |
| 4 | 1 |
| 3 | 0 |
| 2-0 | Operand (r8) |
| 7 | 0 |
| 6 | 0 |
| 5 | 0 |
| 4 | 1 |
| 3 | 1 |
| 2-0 | Operand (r8) |
| 7 | 0 |
| 6 | 0 |
| 5 | 1 |
| 4 | 0 |
| 3 | 0 |
| 2-0 | Operand (r8) |
| 7 | 0 |
| 6 | 0 |
| 5 | 1 |
| 4 | 0 |
| 3 | 1 |
| 2-0 | Operand (r8) |
| 7 | 0 |
| 6 | 0 |
| 5 | 1 |
| 4 | 1 |
| 3 | 0 |
| 2-0 | Operand (r8) |
| 7 | 0 |
| 6 | 0 |
| 5 | 1 |
| 4 | 1 |
| 3 | 1 |
| 2-0 | Operand (r8) |


**`<code>bit b3, r8</code>`** — layout de bits (8 bits):

| Bits | Campo |
|---|---|
| 7 | 0 |
| 6 | 1 |
| 5-3 | Bit index (b3) |
| 2-0 | Operand (r8) |
| 7 | 1 |
| 6 | 0 |
| 5-3 | Bit index (b3) |
| 2-0 | Operand (r8) |
| 7 | 1 |
| 6 | 1 |
| 5-3 | Bit index (b3) |
| 2-0 | Operand (r8) |

_Fonte desta seção: [`src/CPU_Instruction_Set.md`](https://github.com/gbdev/pandocs/blob/fe246067b695b5404a4a6a47efb4fd6d921ececb/src/CPU_Instruction_Set.md)_


---

<!-- fonte: src/CPU_Comparison_with_Z80.md @ fe246067b695 -->

## CPU Comparison with Z80

### Comparison with 8080

The Game Boy CPU has a bit more in common with an older Intel 8080 CPU
than the more powerful Zilog Z80 CPU. It is missing a handful of 8080
instructions but does support JR and almost all CB-prefixed
instructions. Also, all known Game Boy assemblers use the more obvious
Z80-style syntax, rather than the chaotic 8080-style syntax.

Unlike the 8080 and Z80, the Game Boy has no dedicated I/O bus and no
IN/OUT opcodes. Instead, I/O ports are accessed directly by normal LD
instructions, or by new LD (FF00+n) opcodes.

The sign and parity/overflow flags have been removed, as have the 12
RET, CALL, and JP instructions conditioned on them. So have EX (SP),HL
(XTHL) and EX DE,HL (XCHG).

### Comparison with Z80

In addition to the removed 8080 instructions, the other exchange
instructions have been removed (including total absence of second
register set).

All DD- and FD-prefixed instructions are missing. That means no IX- or
IY-registers.

All ED-prefixed instructions are missing. That means 16-bit memory
accesses are mostly missing, 16-bit arithmetic functions are heavily
cut-down, and some other missing instructions. IN/OUT (C) are replaced with
new LD ($FF00+C) opcodes. Block instructions are gone, but autoincrementing
HL accesses are added.

The Game Boy operates approximately as fast as a 4 MHz Z80 (8 MHz in CGB
double speed mode), with execution time of all instructions having been
rounded up to a multiple of 4 cycles.

### Moved, Removed, and Added Opcodes


 Opcode | Z80            | GB CPU
--------|----------------|-------------
 08     | EX   AF,AF     | LD   (nn),SP
 10     | DJNZ PC+dd     | STOP
 22     | LD   (nn),HL   | LDI  (HL),A
 2A     | LD   HL,(nn)   | LDI  A,(HL)
 32     | LD   (nn),A    | LDD  (HL),A
 3A     | LD   A,(nn)    | LDD  A,(HL)
 D3     | OUT  (n),A     | -
 D9     | EXX            | RETI
 DB     | IN   A,(n)     | -
 DD     | \<IX\> prefix  | -
 E0     | RET  PO        | LD   (FF00+n),A
 E2     | JP   PO,nn     | LD   (FF00+C),A
 E3     | EX   (SP),HL   | -
 E4     | CALL P0,nn     | -
 E8     | RET  PE        | ADD  SP,dd
 EA     | JP   PE,nn     | LD   (nn),A
 EB     | EX   DE,HL     | -
 EC     | CALL PE,nn     | -
 ED     | \<prefix\>     | -
 F0     | RET  P         | LD   A,(FF00+n)
 F2     | JP   P,nn      | LD   A,(FF00+C)
 F4     | CALL P,nn      | -
 F8     | RET  M         | LD   HL,SP+dd
 FA     | JP   M,nn      | LD   A,(nn)
 FC     | CALL M,nn      | -
 FD     | \<IY\> prefix  | -
 CB 3X  | SLL  r/(HL)    | SWAP r/(HL)

Note: The unused (-) opcodes will lock up the Game Boy CPU when used.

_Fonte desta seção: [`src/CPU_Comparison_with_Z80.md`](https://github.com/gbdev/pandocs/blob/fe246067b695b5404a4a6a47efb4fd6d921ececb/src/CPU_Comparison_with_Z80.md)_


---
