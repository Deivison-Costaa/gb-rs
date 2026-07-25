# Interrupções e HALT

> **Fonte de verdade deste projeto (regra R1 do CLAUDE.md).**
> Cópia do [Pan Docs](https://gbdev.io/pandocs/) (domínio público, CC0),
> fixada no commit [`fe246067b695`](https://github.com/gbdev/pandocs/tree/fe246067b695b5404a4a6a47efb4fd6d921ececb).
> Não editar à mão: regenerado por `scripts/fetch-reference-docs.sh`.

Cobre o ROADMAP 2.2 e 2.3, incluindo o bug do HALT e o atraso de uma instrução do EI.

**Nesta página:**

- Interrupts
- Interrupt Sources
- halt

---

<!-- fonte: src/Interrupts.md @ fe246067b695 -->

## Interrupts

### IME: Interrupt master enable flag \[write only\]

`IME` is a flag internal to the CPU that controls whether *any* interrupt handlers are called, regardless of the contents of `IE`.
`IME` cannot be read in any way, and is modified by these instructions/events only:

- **`ei`**: Enables interrupt handling (that is, `IME := 1`)
- **`di`**: Disables interrupt handling (that is, `IME := 0`)
- **`reti`**: Enables interrupts and returns (same as `ei` immediately followed by `ret`)
- **When an [interrupt handler](https://gbdev.io/pandocs/single.html#interrupt-handling) is executed**: Disables interrupts before `call`ing the interrupt handler

`IME` is unset (interrupts are disabled) [when the game starts running](https://gbdev.io/pandocs/single.html#0100-0103--entry-point).

The effect of `ei` is delayed by one instruction. This means that `ei`
followed immediately by `di` does not allow any interrupts between them.
This interacts with the [`halt` bug](https://gbdev.io/pandocs/single.html#halt-bug) in an interesting way.

### FFFF — IE: Interrupt enable

**`IE`** — layout de bits (8 bits):

| Bits | Campo |
|---|---|
| 4 | Joypad |
| 3 | Serial |
| 2 | Timer |
| 1 | LCD |
| 0 | VBlank |


- **VBlank** (*Read/Write*): Controls whether [the VBlank interrupt handler](https://gbdev.io/pandocs/single.html#int-40--vblank-interrupt) may be called (see `IF` below).
- **LCD** (*Read/Write*): Controls whether [the LCD interrupt handler](https://gbdev.io/pandocs/single.html#int-48--stat-interrupt) may be called (see `IF` below).
- **Timer** (*Read/Write*): Controls whether [the Timer interrupt handler](https://gbdev.io/pandocs/single.html#int-50--timer-interrupt) may be called (see `IF` below).
- **Serial** (*Read/Write*): Controls whether [the Serial interrupt handler](https://gbdev.io/pandocs/single.html#int-58--serial-interrupt) may be called (see `IF` below).
- **Joypad** (*Read/Write*): Controls whether [the Joypad interrupt handler](https://gbdev.io/pandocs/single.html#int-60--joypad-interrupt) may be called (see `IF` below).

### FF0F — IF: Interrupt flag

**`IF`** — layout de bits (8 bits):

| Bits | Campo |
|---|---|
| 4 | Joypad |
| 3 | Serial |
| 2 | Timer |
| 1 | LCD |
| 0 | VBlank |


- **VBlank** (*Read/Write*): Controls whether [the VBlank interrupt handler](https://gbdev.io/pandocs/single.html#int-40--vblank-interrupt) is being requested.
- **LCD** (*Read/Write*): Controls whether [the LCD interrupt handler](https://gbdev.io/pandocs/single.html#int-48--stat-interrupt) is being requested.
- **Timer** (*Read/Write*): Controls whether [the Timer interrupt handler](https://gbdev.io/pandocs/single.html#int-50--timer-interrupt) is being requested.
- **Serial** (*Read/Write*): Controls whether [the Serial interrupt handler](https://gbdev.io/pandocs/single.html#int-58--serial-interrupt) is being requested.
- **Joypad** (*Read/Write*): Controls whether [the Joypad interrupt handler](https://gbdev.io/pandocs/single.html#int-60--joypad-interrupt) is being requested.

When an interrupt request signal (some internal wire going from the PPU/APU/... to the CPU) changes from low to high, the corresponding bit in the `IF` register becomes set.
For example, bit 0 becomes set when the PPU enters the [VBlank](https://gbdev.io/pandocs/single.html#ppu-modes) period.

Any set bits in the `IF` register are only **requesting** an interrupt.
The actual **execution** of the interrupt handler happens only if both the `IME` flag and the corresponding bit in the `IE` register are set; otherwise the
interrupt "waits" until **both** `IME` and `IE` allow it to be serviced.

Since the CPU automatically sets and clears the bits in the `IF` register, it
is usually not necessary to write to the `IF` register. However, the user
may still do that in order to manually request (or discard) interrupts.
Just like real interrupts, a manually requested interrupt isn't serviced
unless/until `IME` and `IE` allow it.

### Interrupt handling

1. The `IF` bit corresponding to this interrupt and the `IME` flag are reset by the CPU.
The former "acknowledges" the interrupt, while the latter prevents any further interrupts
from being handled until the program re-enables them, typically by using the `reti` instruction.
2. The corresponding interrupt handler (see the `IE` and `IF` register descriptions [above](https://gbdev.io/pandocs/single.html#ffff--ie-interrupt-enable)) is
called by the CPU. This is a regular call, exactly like what would be performed by a `call <address>` instruction (the current PC is pushed onto the stack
and then set to the address of the interrupt handler).

The following interrupt service routine is executed when control is being transferred to an interrupt handler:

1. Two wait states are executed (2 M-cycles pass while nothing happens; presumably the CPU is executing `nop`s during this time).
2. The current value of the PC register is pushed onto the stack, consuming 2 more M-cycles.
3. The PC register is set to the address of the handler (one of: $40, $48, $50, $58, $60).
This consumes one last M-cycle.

The entire process [lasts 5 M-cycles](https://gist.github.com/SonoSooS/c0055300670d678b5ae8433e20bea595#user-content-isr-and-nmi).

### Interrupt priorities

In the following circumstances it is possible that more than one bit in the IF register is set, requesting more than one interrupt at once:

1. More than one interrupt request signal changed from low to high at the same time.
2. Several interrupts have been requested while IME/IE didn't allow them to be serviced.
3. The user has written a value with several bits set (for example binary 00011111) to the IF register.

If IME and IE allow the servicing of more than one of the
requested interrupts, the interrupt with the highest priority
is serviced first. The priorities follow the order of the bits in the IE
and IF registers: Bit 0 (VBlank) has the highest priority, and Bit 4
(Joypad) has the lowest priority.

### Nested interrupt handling

The CPU automatically disables all the other interrupts by setting IME=0
when it services an interrupt. Usually IME remains zero until the
interrupt handler returns (and sets IME=1 by means of the `reti` instruction).
However, if you want to allow the servicing of other interrupts (of any priority)
during the execution of an interrupt handler, you may do so by using the
`ei` instruction in the handler.

_Fonte desta seção: [`src/Interrupts.md`](https://github.com/gbdev/pandocs/blob/fe246067b695b5404a4a6a47efb4fd6d921ececb/src/Interrupts.md)_


---

<!-- fonte: src/Interrupt_Sources.md @ fe246067b695 -->

## Interrupt Sources

### INT $40 — VBlank interrupt

This interrupt [is requested] every time the Game Boy enters VBlank ([Mode 1](https://gbdev.io/pandocs/single.html#ppu-modes)).

The VBlank interrupt occurs ca. 59.7 times a second on a handheld Game
Boy (DMG or CGB) or Game Boy Player and ca. 61.1 times a second on a
Super Game Boy (SGB). This interrupt occurs at the beginning of the
VBlank period (LY=144). During this period video hardware is not using
VRAM so it may be freely accessed. This period lasts approximately 1.1
milliseconds.

### INT $48 — STAT interrupt

There are various sources which can trigger this interrupt to occur as
described in [STAT register ($FF41)](https://gbdev.io/pandocs/single.html#ff41--stat-lcd-status).

The various STAT interrupt sources (modes 0-2 and LYC=LY) have their 
state (inactive=low and active=high) logically ORed into a shared
"STAT interrupt line" if their respective enable bit is turned on.

A STAT interrupt [will be triggered][is requested] by a rising edge (transition from 
low to high) on the STAT interrupt line.

:::warning STAT blocking

If a STAT interrupt source logically ORs the interrupt line high while 
(or immediately after) it's already set high by another source, then 
there will be no low-to-high transition and so no interrupt will occur. 
This phenomenon is known as "STAT blocking" ([test ROM example](https://github.com/Gekkio/mooneye-gb/blob/2d52008228557f9e713545e702d5b7aa233d09bb/tests/acceptance/ppu/stat_irq_blocking.s#L21-L22)).

As mentioned in the description of the [STAT register](https://gbdev.io/pandocs/single.html#ff41--stat-lcd-status),
the PPU cycles through the different modes in a fixed order. So for 
example, if interrupts are enabled for two consecutive modes such as 
Mode 0 and Mode 1, then no interrupt will trigger for Mode 1 (since 
the STAT interrupt line won't have a chance to go low between them).

:::

#### Using the STAT interrupt

One very popular use is to indicate to the user when the video
hardware is about to redraw a given LCD line. This can be useful for
dynamically controlling the SCX/SCY registers ($FF43/$FF42) to [perform
special video effects](https://github.com/gb-archive/DeadCScroll).

Example application: set LYC to WY, enable LY=LYC interrupt, and have
the handler disable objects. This can be used if you use the window for
a text box (at the bottom of the screen), and you want objects (sprites) to be
hidden by the text box.

### INT $50 — Timer interrupt

The timer interrupt [is requested] every time that the timer overflows (that is, when [TIMA](https://gbdev.io/pandocs/single.html#ff05--tima-timer-counter) exceeds $FF).

### INT $58 — Serial interrupt

The serial interrupt [is requested] upon completion of a serial data transfer.
In other words, eight serial clock cycles after starting a transfer (by setting [SC](https://gbdev.io/pandocs/single.html#ff02--sc-serial-transfer-control) bit 7), the incoming data will be in [SB](https://gbdev.io/pandocs/single.html#ff01--sb-serial-transfer-data) and the interrupt will be requested.

### INT $60 — Joypad interrupt

The Joypad interrupt [is requested] when any of [`P1`](https://gbdev.io/pandocs/single.html#ff00--p1joyp-joypad) bits 0-3 change
from High to Low. This happens when a button is
pressed (provided that the action/direction buttons are enabled by
bit 5/4, respectively), however, due to switch bounce, one or more High to Low
transitions are usually produced when pressing a button.

#### Using the joypad interrupt

This interrupt is useful to identify button presses if we have only selected
either action (bit 5) or direction (bit 4), but not both.
If both are selected and, for example, a bit is already held Low by an action button,
pressing the corresponding direction button would
make no difference. The only meaningful purpose of the Joypad
interrupt would be to terminate the STOP (low power) standby state. GBA SP,
because of the different buttons used, seems to not be affected by
switch bounce.

[is requested]: <#FF0F — IF: Interrupt flag>

_Fonte desta seção: [`src/Interrupt_Sources.md`](https://github.com/gbdev/pandocs/blob/fe246067b695b5404a4a6a47efb4fd6d921ececb/src/Interrupt_Sources.md)_


---

<!-- fonte: src/halt.md @ fe246067b695 -->

## `halt`

`halt` is an instruction that pauses the CPU (during which [less power is
consumed](https://gbdev.io/pandocs/single.html#using-the-halt-instruction)) when executed. The CPU wakes up as soon as an interrupt is pending,
that is, when the bitwise AND of [`IE`](https://gbdev.io/pandocs/single.html#ffff--ie-interrupt-enable)
and [`IF`](https://gbdev.io/pandocs/single.html#ff0f--if-interrupt-flag) is non-zero.

Most commonly, [`IME`](https://gbdev.io/pandocs/single.html#ime-interrupt-master-enable-flag-write-only) is
set. In this case, the CPU simply wakes up, and before executing the instruction
after the `halt`, the [interrupt handler is called](https://gbdev.io/pandocs/single.html#interrupt-handling)
normally.

If `IME` is *not* set, there are two distinct cases, depending on whether an
interrupt is pending as the `halt` instruction is first executed.

- If no interrupt is pending, `halt` executes as normal, and the CPU resumes
  regular execution as soon as an interrupt becomes pending. However, since
  `IME`=0, the interrupt is not handled.
- If an interrupt is pending, `halt` immediately exits, as expected, however
  the "`halt` bug", explained below, is triggered.

### `halt` bug

When a `halt` instruction is executed with `IME = 0` and `[IE] & [IF] != 0`, the `halt` instruction ends immediately, but [`pc` fails to be normally incremented](https://github.com/nitro2k01/little-things-gb/tree/main/double-halt-cancel).

Under most circumstances, this causes the byte after the `halt` to be read a second time (and this behaviour can repeat if said byte executes another `halt` instruction).
But, if the `halt` is immediately followed by a jump to elsewhere, then the behaviour will be slightly different; this is possible in only one of two ways:

- The `halt` comes immediately after a `ei` instruction (whose effect is typically delayed by one instruction, hence `IME` still being zero for the `halt`): the interrupt is serviced and the handler called, but the interrupt returns to the `halt`, which is executed again, and thus
waits for another interrupt.
([Source](https://github.com/LIJI32/SameSuite/blob/master/interrupt/ei_delay_halt.asm))
- The `halt` is immediately followed by a `rst` instruction: the `rst` instruction's return address will point at the `rst` itself, instead of the byte after it.
  Notably, a `ret` would return to the `rst` an execute it again.

If the bugged `halt` is preceded by a `ei` and followed by a `rst`, the former "wins".

_Fonte desta seção: [`src/halt.md`](https://github.com/gbdev/pandocs/blob/fe246067b695b5404a4a6a47efb4fd6d921ececb/src/halt.md)_


---
