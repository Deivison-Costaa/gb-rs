# Timer e divisor

> **Fonte de verdade deste projeto (regra R1 do CLAUDE.md).**
> Cópia do [Pan Docs](https://gbdev.io/pandocs/) (domínio público, CC0),
> fixada no commit [`fe246067b695`](https://github.com/gbdev/pandocs/tree/fe246067b695b5404a4a6a47efb4fd6d921ececb).
> Não editar à mão: regenerado por `scripts/fetch-reference-docs.sh`.

Cobre o ROADMAP 2.1. A seção de comportamento obscuro documenta o atraso de 4 ciclos no overflow de TIMA, que é o que a blargg instr_timing cobra.

**Nesta página:**

- Timer and Divider Registers
- Timer Obscure Behaviour

---

<!-- fonte: src/Timer_and_Divider_Registers.md @ fe246067b695 -->

## Timer and Divider Registers

:::tip NOTE

The Timer described below is the built-in timer in the Game Boy. It has
nothing to do with the MBC3s battery buffered Real Time Clock - that\'s
a completely different thing, described in
[Memory Bank Controllers](https://gbdev.io/pandocs/single.html#mbcs).

:::

### FF04 — DIV: Divider register

This register is incremented at a rate of 16384Hz (\~16779Hz on SGB).
Writing any value to this register resets it to $00.
Additionally, this register is reset when executing the `stop` instruction, and
only begins ticking again once `stop` mode ends. This also occurs during a
[speed switch](https://gbdev.io/pandocs/single.html#ff4d--key1spd-cgb-mode-only-prepare-speed-switch).
(TODO: how is it affected by the wait after a speed switch?)

Note: The divider is affected by CGB double speed mode, and will
increment at 32768Hz in double speed.

### FF05 — TIMA: Timer counter

This timer is incremented at the clock frequency specified by the TAC
register ($FF07). When the value overflows (exceeds $FF)
it is reset to the value specified in TMA (FF06) and [an interrupt](https://gbdev.io/pandocs/single.html#int-50--timer-interrupt)
is requested, as described below.

### FF06 — TMA: Timer modulo

When TIMA overflows, it is reset to the value in this register and [an interrupt](https://gbdev.io/pandocs/single.html#int-50--timer-interrupt) is requested.
Example of use: if TMA is set to $FF, an interrupt is requested at the clock frequency selected in
TAC (because every increment is an overflow). However, if TMA is set to $FE, an interrupt is
only requested every two increments, which effectively divides the selected clock by two. Setting
TMA to $FD would divide the clock by three, and so on.

If a TMA write is executed on the same M-cycle as the content of TMA is transferred to TIMA
due to a timer overflow, the old value is transferred to TIMA.

### FF07 — TAC: Timer control

**`TAC`** — layout de bits (8 bits):

| Bits | Campo |
|---|---|
| 2 | Enable |
| 1-0 | Clock select |


- **Enable**: Controls whether `TIMA` is incremented.
  Note that `DIV` is **always** counting, regardless of this bit.
- **Clock select**: Controls the frequency at which `TIMA` is incremented, as follows:
  
  <div class="table-wrapper"><table>
    <thead>
      <tr><th rowspan=2>Clock select</th><th rowspan=2>Increment every</th><th colspan=3>Frequency (Hz)</th></tr>
      <tr><th>DMG, SGB2, CGB in normal-speed mode</th><th>SGB1</th><th>CGB in double-speed mode</th></tr>
    </thead><tbody>
      <tr><td>00</td><td>256 M-cycles </td><td>  4096</td><td>  ~4194</td><td>  8192</td></tr>
      <tr><td>01</td><td>4 M-cycles   </td><td>262144</td><td>~268400</td><td>524288</td></tr>
      <tr><td>10</td><td>16 M-cycles  </td><td> 65536</td><td> ~67110</td><td>131072</td></tr>
      <tr><td>11</td><td>64 M-cycles  </td><td> 16384</td><td> ~16780</td><td> 32768</td></tr>
    </tbody>
  </table></div>

Note that writing to this register [may increase `TIMA` once](https://gbdev.io/pandocs/single.html#relation-between-timer-and-divider-register)!

_Fonte desta seção: [`src/Timer_and_Divider_Registers.md`](https://github.com/gbdev/pandocs/blob/fe246067b695b5404a4a6a47efb4fd6d921ececb/src/Timer_and_Divider_Registers.md)_


---

<!-- fonte: src/Timer_Obscure_Behaviour.md @ fe246067b695 -->

## Timer obscure behaviour

:::tip System counter

DIV is just the visible part of the **system counter**.

The **system counter** is constantly incrementing every M-cycle, unless the CPU is in [STOP mode](https://gbdev.io/pandocs/single.html#using-the-stop-instruction).

:::

### Timer Global Circuit

> _Diagrama `imgs/src/timer_simplified.svg` omitido nesta cópia offline (era um SVG do Pan Docs). Ver https://gbdev.io/pandocs/single.html_

### Relation between Timer and Divider register

This is a schematic of the circuit involving TAC and DIV:

<figure><figcaption>

On **DMG**:

</figcaption>
> _Diagrama `imgs/src/timer_tac_bug_dmg.svg` omitido nesta cópia offline (era um SVG do Pan Docs). Ver https://gbdev.io/pandocs/single.html_
</figure>

<figure><figcaption>

On **CGB**:

</figcaption>
> _Diagrama `imgs/src/timer_tac_bug_gbc.svg` omitido nesta cópia offline (era um SVG do Pan Docs). Ver https://gbdev.io/pandocs/single.html_
</figure>

Notice how the bits themselves are connected to the multiplexer and then to the falling-edge detector; this causes a few odd behaviors:

- Resetting the entire system counter (by writing to `DIV`) can reset the bit currently selected by the multiplexer, thus sending a "Timer tick" and/or "[DIV-APU event](https://gbdev.io/pandocs/single.html#div-apu)" pulse early.
- Changing which bit of the system counter is selected (by changing the "Clock select" bits of [`TAC`]) from a bit currently set to another that is currently unset, will send a "Timer tick" pulse.
  (For example: if the system counter is equal to $3FF0 and `TAC` to $FC, writing $05 or $06 to `TAC` will instantly send a "Timer tick", but $04 or $07 won't.)
- On monochrome consoles, disabling the timer if the currently selected bit is set, will send a "Timer tick" once.
  This does not happen on Color models.
- On Color models, a write to `TAC` that fulfills the previous bullet's conditions *and* turns the timer on (it was disabled before) may or may not send a "Timer tick".
  The exact behaviour varies between individual consoles.

### Timer overflow behavior

When `TIMA` overflows, the value from `TMA` is copied, and the timer flag is set in [`IF`], but **one M-cycle later**.
This means that `TIMA` is equal to $00 for the M-cycle after it overflows.

This only happens when `TIMA` overflows from incrementing, it cannot be made to happen by manually writing to `TIMA`.

Here is an example; `SYS` represents the lower 8 bits of the system counter, and `TAC` is $FD (timer enabled, bit 1 of `SYS` selected as source):

<figure><figcaption>

`TIMA` overflows on cycle <var>A</var>, but the interrupt is only requested on cycle <var>B</var>:

</figcaption>

M-cycle |    |    ||<var>A</var>|<var>B</var>||&#8203;
--------|----|----|----|--------|----|----|---
`SYS`   | 2B | 2C | 2D |   2E   | 2F | 30 | 31
`TIMA`  | FE | FF | FF | **00** | 23 | 24 | 24
`TMA`   | 23 | 23 | 23 |   23   | 23 | 23 | 23
`IF`    | E0 | E0 | E0 | **E0** | E4 | E4 | E4

</figure>

Here are some unexpected behaviors:

1. Writing to `TIMA` during cycle <var>A</var> acts as if the overflow **didn't happen**!
   `TMA` will not be copied to `TIMA` (the value written will therefore stay), and bit 2 of `IF` will not be set.
   Writing to `DIV`, `TAC`, or other registers won't prevent the `IF` flag from being set or `TIMA` from being reloaded.
2. Writing to `TIMA` during cycle <var>B</var> will be ignored; `TIMA` will be equal to `TMA` at the end of the cycle anyway.
3. Writing to `TMA` during cycle <var>B</var> will have the same value copied to `TIMA` as well, on the same cycle.

Here is how `TIMA` and `TMA` interact:

> _Diagrama `imgs/src/timer_tima_tma_detailed.svg` omitido nesta cópia offline (era um SVG do Pan Docs). Ver https://gbdev.io/pandocs/single.html_

<details><summary>Explanation of the above behaviors:</summary>

1. Writing to `TIMA` blocks the falling edge from the increment from being detected (see the `AND` gate)[^Timer_Obscure_Behaviour_write_edge].
2. The "Load" signal stays enabled for the entirety of cycle <var>B</var>, and since `TIMA` is made of <abbr title="T-flip-flop with Asynchronous Load">TAL</abbr> cells, it's constantly copying its input.
   However, the "Write to TIMA" signal gets reset in the middle of the cycle, thus the multiplexer emits `TMA`'s value again; in essence, the CPU's write to `TIMA` *does* go through, but it's overwritten right after.
3. As mentioned in the previous bullet point, `TIMA` constantly copies its input, so it updates together with `TMA`.
   This and the previous bullet point can be emulated as if `TMA` was copied to `TIMA` at the very end of the cycle, though this is not quite what's happening in hardware.

[^Timer_Obscure_Behaviour_write_edge]: This is necessary, because otherwise writing a number with bit 7 reset (either from the CPU or from `TMA`) when `TIMA`'s bit 7 is set, would trigger the bit 7 falling edge detector and thus schedule a spurious interrupt.

</details>

[`TAC`]: <#FF07 — TAC: Timer control>
[`IF`]: <#FF0F — IF: Interrupt flag>

_Fonte desta seção: [`src/Timer_Obscure_Behaviour.md`](https://github.com/gbdev/pandocs/blob/fe246067b695b5404a4a6a47efb4fd6d921ececb/src/Timer_Obscure_Behaviour.md)_


---
