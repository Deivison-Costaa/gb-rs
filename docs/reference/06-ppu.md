# PPU — registradores, modos e renderização

> **Fonte de verdade deste projeto (regra R1 do CLAUDE.md).**
> Cópia do [Pan Docs](https://gbdev.io/pandocs/) (domínio público, CC0),
> fixada no commit [`fe246067b695`](https://github.com/gbdev/pandocs/tree/fe246067b695b5404a4a6a47efb4fd6d921ececb).
> Não editar à mão: regenerado por `scripts/fetch-reference-docs.sh`.

Cobre todo o marco M3. Inclui o pixel FIFO por completude: este projeto usa renderizador por scanline (invariante do STATUS.md), mas o FIFO explica os efeitos que o scanline não reproduz.

**Nesta página:**

- Graphics
- Tile Data
- Tile Maps
- OAM
- OAM DMA Transfer
- Window
- LCDC
- STAT
- Scrolling
- Palettes
- Rendering
- pixel fifo
- Accessing VRAM and OAM
- OAM Corruption Bug

---

<!-- fonte: src/Graphics.md @ fe246067b695 -->

## Graphics Overview

The Game Boy outputs graphics to a 160×144 pixel LCD, using a quite complex
mechanism to facilitate rendering.

:::warning Terminology

Sprites/graphics terminology can vary a lot among different platforms, consoles,
users and communities. You may be familiar with slightly different definitions.
Keep also in mind that some definitions refer to lower (hardware) tools
and some others to higher abstractions concepts.

:::

### Tiles

Similarly to other retro systems, pixels are not manipulated
individually, as this would be expensive CPU-wise. Instead, pixels are grouped
in 8×8 squares, called _tiles_ (or sometimes "patterns" or "characters"), often considered as
the base unit in Game Boy graphics.

A tile does not encode color information. Instead, a tile assigns a
[_color indices_](https://gbdev.io/pandocs/single.html#data-format) to each of its pixels, ranging from 0 to 3. For this reason,
Game Boy graphics are also called _2bpp_ (2 bits per pixel). When a tile is used
in the Background or Window, these [color indices](https://gbdev.io/pandocs/single.html#data-format) are associated with a _palette_. When
a tile is used in an object, the indices 1 to 3 are associated with a palette, but
ID 0 means transparent.

### Palettes

A palette consists of an array of colors, 4 in the Game Boy's case.
Palettes are stored differently in monochrome and color versions of the console.

Modifying palettes enables graphical effects such as quickly flashing some graphics (damage,
invulnerability, thunderstorm, etc.), fading the screen, "palette swaps", and more.

### Layers

The Game Boy has three "layers", from back to front: the Background, the Window,
and the Objects. Some features and behaviors break this abstraction,
but it works for the most part.

#### Background

The background is composed of a _tilemap_. A tilemap is a
large grid of tiles. However, tiles aren't directly written to tilemaps,
they merely contain references to the tiles.
This makes reusing tiles very cheap, both in CPU time and in
required memory space, and it is the main mechanism that helps work around the
paltry 8 KiB of video RAM.

The background can be made to scroll as a whole, writing to two
hardware registers. This makes scrolling very cheap.

#### Window

The window is sort of a second background layer on top of the background.
It is fairly limited: it has no transparency, it's always a
rectangle and only the position of the top-left pixel can be controlled.

Possible usage include a fixed status bar in an otherwise scrolling game (e.g.
_Super Mario Land 2_).

#### Objects

The background layer is useful for elements scrolling as a whole, but
it's impractical for objects that need to move separately, such as the player.

The _objects_ layer is designed to fill this gap: _objects_ are made of 1 or 2 stacked tiles (8×8 or 8×16 pixels)
and can be displayed anywhere on the screen.

:::tip NOTE

Several objects can be combined (they can be called _metasprites_) to draw
a larger graphical element, usually called "sprite". Originally, the term "sprites"
referred to fixed-sized objects composited together, by hardware, with a background.
Use of the term has since become more general.

:::

To summarise:

- **Tile**, an 8×8-pixel chunk of graphics.
- **Object**, an entry in object attribute memory, composed of 1 or 2
  tiles. Can be moved independently of the background.

_Fonte desta seção: [`src/Graphics.md`](https://github.com/gbdev/pandocs/blob/fe246067b695b5404a4a6a47efb4fd6d921ececb/src/Graphics.md)_


---

<!-- fonte: src/Tile_Data.md @ fe246067b695 -->

## VRAM Tile Data

Tile data is stored in VRAM in the memory area at $8000-$97FF; with each tile
taking 16 bytes, this area defines data for 384 tiles. In CGB Mode,
this is doubled (768 tiles) because of the two VRAM banks.

Each tile (or character) has 8×8 pixels and has a color depth of
2 bits per pixel, allowing each pixel to use one of 4 colors or gray
shades. Tiles can be displayed as part of the Background/Window maps,
and/or as objects (movable sprites).  Color 0 has a special meaning
in objects - it's transparent, allowing the background or other
objects behind it to show through.

There are three "blocks" of 128 tiles each:

<div class="table-wrapper" style="text-align: center;"><table><thead>
  <tr>
    <th rowspan=2>Tile IDs for...</th>
    <th>Block 0</th>
    <th>Block 1</th>
    <th>Block 2</th>
  </tr>
  <tr>
    <th>$8000–87FF</th>
    <th>$8800–8FFF</th>
    <th>$9000–97FF</th>
  </tr>
</thead><tbody>
  <tr>
    <td><strong>Objects</strong></td>
    <td>0–127</td>
    <td>128–255</td>
    <td>—</td>
  </tr>
  <tr>
    <td><strong>BG/Win</strong>, if LCDC.4=1</td>
    <td>0–127</td>
    <td>128–255</td>
    <td>—</td>
  </tr>
  <tr>
    <td><strong>BG/Win</strong>, if LCDC.4=0</td>
    <td>—</td>
    <td>128–255</td>
    <td>0–127</td>
  </tr>
</tbody></table></div>

Tiles are always indexed using an 8-bit integer, but the addressing method may differ:

- The "**$8000 method**" uses $8000 as its base pointer and uses an unsigned addressing, meaning that tiles 0-127 are in block 0, and tiles 128-255 are in block 1.
- The "**$8800 method**" uses $9000 as its base pointer and uses a signed addressing, meaning that tiles 0-127 are in block 2, and tiles -128 to -1 are in block 1; or, to put it differently, "$8800 addressing" takes tiles 0-127 from block 2 and tiles 128-255 from block 1.

(You can notice that block 1 is shared by both addressing methods)

Objects always use "$8000 addressing", but the BG and Window can use either mode, controlled by [LCDC bit 4](https://gbdev.io/pandocs/single.html#lcdc4--bg-and-window-tile-data-area).

### Data format

Each tile occupies 16 bytes, where each line is represented by 2 bytes:

<table>
  <thead>
    <tr><th>Byte</th><th>1<sup>st</sup></th><th>2<sup>nd</sup></th><th>3<sup>rd</sup></th><th>4<sup>th</sup></th><th>...</th></tr>
  </thead>
  <tbody>
    <tr><td>Explanation</td><td colspan="2">Topmost line (top 8 pixels)</td><td colspan="2">Second line</td><td>Etc.</td></tr>
  </tbody>
</table>

For each line, the first byte specifies the least significant bit of the color
ID of each pixel, and the second byte specifies the most significant bit. In
both bytes, bit 7 represents the leftmost pixel, and bit 0 the rightmost. For
example, the tile data `$3C $7E $42 $42 $42 $42 $42 $42 $7E $5E $7E $0A $7C $56
$38 $7C` appears as follows:

<figure>
> _Diagrama `imgs/src/sprite.svg` omitido nesta cópia offline (era um SVG do Pan Docs). Ver https://gbdev.io/pandocs/single.html_
<figcaption>Sample tile data</figcaption>
</figure>

For the first row, the values `$3C $7E` are `00111100` and `01111110` in
binary. The leftmost bits are 0 and 0, thus the [color index](https://gbdev.io/pandocs/single.html#data-format) is binary `00`, or 0.
The next bits are 0 and 1, thus the [color index](https://gbdev.io/pandocs/single.html#data-format) is binary `10`, or 2 (remember to
flip the order of the bits!). The full eight-pixel row evaluates to 0 2 3 3 3 3
2 0.

A tool for viewing tiles can be found
[here](https://www.huderlem.com/demos/gameboy2bpp.html).

So, each pixel has a [color index](https://gbdev.io/pandocs/single.html#data-format) of 0 to 3. The color
numbers are translated into real colors (or gray shades) depending on
the current palettes, except that when the tile is used in an OBJ the
[color index](https://gbdev.io/pandocs/single.html#data-format) 0 means transparent. The palettes are defined through registers
[BGP](https://gbdev.io/pandocs/single.html#ff47--bgp-non-cgb-mode-only-bg-palette-data),
[OBP0 and OBP1](https://gbdev.io/pandocs/single.html#ff48ff49--obp0-obp1-non-cgb-mode-only-obj-palette-0-1-data), and
[BCPS/BGPI](https://gbdev.io/pandocs/single.html#ff68--bcpsbgpi-cgb-mode-only-background-color-palette-specification--background-palette-index),
[BCPD/BGPD](https://gbdev.io/pandocs/single.html#ff69--bcpdbgpd-cgb-mode-only-background-color-palette-data--background-palette-data),
[OCPS/OBPI and OCPD/OBPD](https://gbdev.io/pandocs/single.html#ff6aff6b--ocpsobpi-ocpdobpd-cgb-mode-only-obj-color-palette-specification--obj-palette-index-obj-color-palette-data--obj-palette-data)
(CGB Mode).

_Fonte desta seção: [`src/Tile_Data.md`](https://github.com/gbdev/pandocs/blob/fe246067b695b5404a4a6a47efb4fd6d921ececb/src/Tile_Data.md)_


---

<!-- fonte: src/Tile_Maps.md @ fe246067b695 -->

## VRAM Tile Maps

The Game Boy contains two 32×32 tile maps in VRAM at
the memory areas `$9800-$9BFF` and `$9C00-$9FFF`. Any of these maps can be used to
display the Background or the Window.

### Tile Indexes

Each tile map contains the 1-byte indexes of the
tiles to be displayed.

Tiles are obtained from the Tile Data Table using either of the two
addressing modes (described in [VRAM Tile Data](https://gbdev.io/pandocs/single.html#vram-tile-data)), which
can be selected via [the LCDC register](https://gbdev.io/pandocs/single.html#ff40--lcdc-lcd-control).

Since one tile has 8×8 pixels, each map holds a 256×256 pixels picture.
Only 160×144 of those pixels are displayed on the LCD at any given time.

### BG Map Attributes (CGB Mode only)

In CGB Mode, an additional map of 32×32 bytes is stored in VRAM Bank 1
(each byte defines attributes for the corresponding tile-number map
entry in VRAM Bank 0, that is, 1:9800 defines the attributes for the tile at
0:9800):

**`BG attributes`** — layout de bits (8 bits):

| Bits | Campo |
|---|---|
| 7 | Priority |
| 6 | Y flip |
| 5 | X flip |
| 3 | Bank |
| 2-0 | Color palette |


- **Priority**: `0` = No; `1` = [Color indices](https://gbdev.io/pandocs/single.html#data-format) 1–3 of the corresponding BG/Window tile are drawn over OBJ, regardless of [OBJ priority](https://gbdev.io/pandocs/single.html#byte-3--attributesflags)
- **Y flip**: `0` = Normal; `1` = Tile is drawn vertically mirrored
- **X flip**: `0` = Normal; `1` = Tile is drawn horizontally mirrored
- **Bank**: `0` = Fetch tile from VRAM bank 0; `1` = Fetch tile from VRAM bank 1
- **Color palette**: Which of BGP0–7 to use

Bit 4 is ignored by the hardware, but can be written to and read from normally.

Note that, for example, if the byte at `0:9800` is $2A, the attribute at `1:9800` doesn't define properties for ALL tiles $2A on-screen, but only the one at `0:9800`!

#### BG-to-OBJ Priority in CGB Mode

In CGB Mode, the priority between the BG (and window) layer and the OBJ layer is declared in three different places:
- [BG Map Attribute bit 7](https://gbdev.io/pandocs/single.html#bg-map-attributes-cgb-mode-only)
- [LCDC bit 0](https://gbdev.io/pandocs/single.html#lcdc0--bg-and-window-enablepriority)
- [OAM Attributes bit 7](https://gbdev.io/pandocs/single.html#byte-3--attributesflags)

We can infer the following rules from the table below:
* If the BG color index is 0, the OBJ will always have priority;
* Otherwise, if LCDC bit 0 is clear, the OBJ will always have priority;
* Otherwise, if both the BG Attributes and the OAM Attributes have bit 7 clear, the OBJ will have priority;
* Otherwise, BG will have priority.

The following table shows the relations between the 3 flags:

LCDC bit 0 | OAM attr bit 7 | BG attr bit 7 | Priority
:---------:|:--------------:|:-------------:|---------
0          | 0              | 0             | OBJ
0          | 0              | 1             | OBJ
0          | 1              | 0             | OBJ
0          | 1              | 1             | OBJ
1          | 0              | 0             | OBJ
1          | 0              | 1             | BG color 1–3, otherwise OBJ
1          | 1              | 0             | BG color 1–3, otherwise OBJ
1          | 1              | 1             | BG color 1–3, otherwise OBJ

[This test ROM](https://github.com/alloncm/MagenTests) can be used to observe the above.

:::warning

Keep in mind that:
* OAM Attributes bit 7 will grant OBJ priority when **clear**, not when **set**.
* Priority between all OBJs is resolved **before** priority with the BG layer is considered.
  Please refer [to this page](https://gbdev.io/pandocs/single.html#drawing-priority) for more details.

:::

### Background (BG)

The [SCY and SCX](https://gbdev.io/pandocs/single.html#ff42ff43--scy-scx-background-viewport-y-position-x-position)
registers can be used to scroll the Background, specifying the origin of the visible
160×144 pixel area within the total 256×256 pixel Background map.
The visible area of the Background wraps around the Background map (that is, when part of
the visible area goes beyond the map edge, it starts displaying the opposite side of the map).

In Non-CGB mode, the Background (and the Window) can be disabled using
[LCDC bit 0](https://gbdev.io/pandocs/single.html#lcdc0--bg-and-window-enablepriority).

### Window

Besides the Background, there is also a Window overlaying it.
The content of the Window is not scrollable; it is always
displayed starting at the top left tile of its tile map. The only way to adjust the Window
is by modifying its position on the screen, which is done via the WX and WY registers. The screen
coordinates of the top left corner of the Window are (WX-7,WY). The tiles
for the Window are stored in the Tile Data Table. Both the Background
and the Window share the same Tile Data Table.

Whether the Window is displayed can be toggled using
[LCDC bit 5](https://gbdev.io/pandocs/single.html#lcdc5--window-enable). But in Non-CGB mode this bit is only
functional as long as [LCDC bit 0](https://gbdev.io/pandocs/single.html#lcdc0--bg-and-window-enablepriority) is set.
Enabling the Window makes
[Mode 3](https://gbdev.io/pandocs/single.html#ppu-modes) slightly longer on scanlines where it's visible.
(See [WX and WY](https://gbdev.io/pandocs/single.html#ff4aff4b--wy-wx-window-y-position-x-position-plus-7)
for the definition of "Window visibility".)

:::tip Window Internal Line Counter

The window keeps an internal line counter that's functionally similar to `LY`, and increments alongside it. However, it only gets incremented when the window is visible, as described [here](https://gbdev.io/pandocs/single.html#window-rendering-criteria).

This line counter determines what window line is to be rendered on the current scanline.

:::

_Fonte desta seção: [`src/Tile_Maps.md`](https://github.com/gbdev/pandocs/blob/fe246067b695b5404a4a6a47efb4fd6d921ececb/src/Tile_Maps.md)_


---

<!-- fonte: src/OAM.md @ fe246067b695 -->

## Object Attribute Memory (OAM)

The Game Boy PPU can display up to 40 movable objects (or sprites), each 8×8 or
8×16 pixels. Because of a limitation of hardware, only ten objects
can be displayed per scanline. Object tiles have the same format as
BG tiles, but they are taken from tile blocks 0 and 1 located at
$8000-8FFF and have unsigned numbering.

Object attributes reside in the object attribute memory (OAM) at $FE00-FE9F.
(This corresponds to the sprite attribute table on a TMS9918 VDP.)
Each of the 40 entries consists of
four bytes with the following meanings:

### Byte 0 — Y Position

> _(imagem omitida nesta cópia offline)_

Y = Object's vertical position on the screen + 16. So for example:

- Y=0 hides an object,
- Y=2 hides an 8×8 object but displays the last two rows of an 8×16 object,
- Y=16 displays an object at the top of the screen,
- Y=144 displays an 8×16 object aligned with the bottom of the screen,
- Y=152 displays an 8×8 object aligned with the bottom of the screen,
- Y=154 displays the first six rows of an object at the bottom of the screen,
- Y=160 hides an object.

### Byte 1 — X Position

X = Object's horizontal position on the screen + 8. This works similarly
to the examples above, except that the width of an object is always 8. An
off-screen value (X=0 or X\>=168) hides the object, but the object still
contributes to the limit of ten objects per scanline.
This can cause objects later in OAM not to be drawn on that line.
A better way to hide an object is to set its Y-coordinate off-screen.

### Byte 2 — Tile Index

In 8×8 mode (LCDC bit 2 = 0), this byte specifies the object's only tile index ($00-$FF).
This unsigned value selects a tile from the memory area at $8000-$8FFF.
In CGB Mode this could be either in
VRAM bank 0 or 1, depending on bit 3 of the following byte.
In 8×16 mode (LCDC bit 2 = 1), the memory area at $8000-$8FFF is still interpreted
as a series of 8×8 tiles, where every 2 tiles form an object. In this mode, this byte
specifies the index of the first (top) tile of the object. This is enforced by the
hardware: the least significant bit of the tile index is ignored; that is, the top 8×8
tile is "NN & $FE", and the bottom 8×8 tile is "NN | $01".

### Byte 3 — Attributes/Flags

**`Attributes`** — layout de bits (8 bits):

| Bits | Campo |
|---|---|
| 7 | Priority |
| 6 | Y flip |
| 5 | X flip |
| 4 | DMG palette |
| 3 | Bank |
| 2-0 | CGB palette |


- **Priority**: `0` = No, `1` = BG and Window [color indices](https://gbdev.io/pandocs/single.html#data-format) 1–3 are drawn over this OBJ
- **Y flip**: `0` = Normal, `1` = Entire OBJ is vertically mirrored
- **X flip**: `0` = Normal, `1` = Entire OBJ is horizontally mirrored
- **DMG palette** *\[Non CGB Mode only\]*: `0` = OBP0, `1` = OBP1
- **Bank** *\[CGB Mode Only\]*: `0` = Fetch tile from VRAM bank 0, `1` = Fetch tile from VRAM bank 1
- **CGB palette** *\[CGB Mode Only\]*: Which of OBP0–7 to use

### Writing data to OAM

The recommended method is to write the data to a buffer in normal RAM
(typically WRAM) first, then to copy that buffer to OAM using
[the DMA transfer functionality](https://gbdev.io/pandocs/single.html#oam-dma-transfer).

While it is also possible to write data directly to the OAM area
[by accessing it normally](https://gbdev.io/pandocs/single.html#oam-memory-area-at-fe00-fe9f-is-accessible-during-modes-0-1),
this only works [during the HBlank and VBlank periods](https://gbdev.io/pandocs/single.html#ppu-modes).

### Object Priority and Conflicts

There are two kinds of "priorities" as far as objects are concerned.
The first one defines which objects are ignored when there are more than 10 on a
given scanline. The second one decides which object is displayed on top when some
overlap (the Game Boy being a 2D console, there is no Z coordinate).

#### Selection priority

During each scanline's OAM scan, the PPU compares [`LY`](https://gbdev.io/pandocs/single.html#ff44--ly-lcd-y-coordinate-read-only)
([using `LCDC` bit 2 to determine their size](https://gbdev.io/pandocs/single.html#lcdc2--obj-size)) to each
object's Y position to select up to 10 objects to be drawn on that line.
The PPU scans OAM sequentially (from $FE00 to $FE9F), selecting the first (up to)
10 suitably-positioned objects.

Since the PPU only checks the Y coordinate to select objects, even
off-screen objects count towards the 10-objects-per-scanline limit.
Merely setting an object's X coordinate to X&nbsp;=&nbsp;0 or X&nbsp;≥&nbsp;168
(160&nbsp;+&nbsp;8) will hide it, but it will still count towards the
limit, possibly causing another object later in OAM not
to be drawn. To keep off-screen objects from affecting on-screen ones, make
sure to set their Y coordinate to Y&nbsp;=&nbsp;0 or Y&nbsp;≥&nbsp;160
(144&nbsp;+&nbsp;16).
(Y&nbsp;≤&nbsp;8 also works if [object size](https://gbdev.io/pandocs/single.html#lcdc2--obj-size) is set to 8×8.)

#### Drawing priority

When **opaque** pixels from two different objects overlap, which pixel ends up
being displayed is determined by another kind of priority: the pixel belonging
to the higher-priority object wins. However, this priority is determined
differently when in CGB mode.

- **In Non-CGB mode**, the smaller the X coordinate, the higher the priority.
  When X coordinates are identical, the object located first in OAM has higher
  priority.
- **In CGB mode**, only the object's location in OAM determines its priority.
  The earlier the object, the higher its priority.

:::tip Interaction with "BG over OBJ" flag

Object drawing priority and ["BG over OBJ"](https://gbdev.io/pandocs/single.html#bg-map-attributes-cgb-mode-only) interact in a non-intuitive way.

Internally, the PPU first resolves priority between objects to
pick an "object pixel", which is the first non-transparent pixel encountered
when iterating over objects sorted by their drawing priority.
The "BG over OBJ" attribute is **never** considered in this process.

Only *after* object priority is resolved, the "object pixel" has the "BG over
OBJ" attribute of its object checked to determine whether it should be drawn
over the background.
This means that an object with a higher priority but with "BG over OBJ" enabled
will sort of "mask" lower-priority objects, even if those have "BG over OBJ"
disabled.

This can be exploited to only hide parts of an object behind the background
([video demonstration](https://youtu.be/B8sJGgCVvnk)).
A similar behaviour [can be seen on the NES](https://forums.nesdev.org/viewtopic.php?f=10&t=16861).

**In CGB Mode**, BG vs. OBJ priority is declared in more than one register, [please see this page](https://gbdev.io/pandocs/single.html#bg-to-obj-priority-in-cgb-mode) for more details.

:::

_Fonte desta seção: [`src/OAM.md`](https://github.com/gbdev/pandocs/blob/fe246067b695b5404a4a6a47efb4fd6d921ececb/src/OAM.md)_


---

<!-- fonte: src/OAM_DMA_Transfer.md @ fe246067b695 -->

## OAM DMA Transfer

### FF46 — DMA: OAM DMA source address & start

Writing to this register starts a DMA transfer from ROM or RAM to OAM
(Object Attribute Memory).  The written value specifies the
transfer source address divided by $100, that is, source and destination are:

```
Source:      $XX00-$XX9F   ;XX = $00 to $DF
Destination: $FE00-$FE9F
```

The transfer takes 160 M-cycles: 640 dots (1.4 lines) in normal speed,
or 320 dots (0.7 lines) in CGB Double Speed Mode.
This is much faster than a CPU-driven copy.

### OAM DMA bus conflicts

On DMG, during OAM DMA, the CPU can access only HRAM (memory at $FF80-$FFFE).
For this reason, the programmer must copy a short procedure (see below) into HRAM, and use
this procedure to start the transfer **from inside HRAM**, and wait until
the transfer has finished.

On CGB, the cartridge and WRAM are on separate buses.
This means that the CPU can access ROM or cartridge SRAM during OAM DMA from WRAM, or WRAM during OAM DMA from ROM or SRAM.
However, because a `call` writes a return address to the stack, and the stack and variables are usually in WRAM,
it's still recommended to busy-wait in HRAM for DMA to finish even on CGB.

:::warning Interrupts

An interrupt writes a return address to the stack and fetches the interrupt handler's instructions from ROM.
Thus, it's critical to prevent interrupts during OAM DMA, especially in a program that uses timer, serial, or joypad interrupts, since they are not synchronized to the LCD.
This can be done by executing DMA within the VBlank interrupt handler or through the `di` instruction.

:::

While an OAM DMA is in progress, the PPU cannot read OAM properly either.
Thus, most programs execute DMA during [Mode 1](https://gbdev.io/pandocs/single.html#ppu-modes), inside or immediately after their VBlank handler.
But it is also possible to execute it during display redraw (Modes 2 and 3),
allowing to display more than 40 objects on the screen (that is, for
example 40 objects in the top half, and other 40 objects in the bottom half of
the screen), at the cost of a couple lines that lack objects.
If the transfer is started during Mode 3, graphical glitches may happen.

The details:

* If OAM DMA is active during OAM scan (mode 2), most PPU revisions read each object
  as being off-screen and thus hidden on that line.
* If OAM DMA is active during rendering (mode 3), the PPU reads whatever 16-bit word
  the DMA unit is writing to OAM when the object is fetched.
  This causes an incorrect tile number and attributes for objects already determined to be in range.

<!-- TODO: find Hacktix test ROM -->
<!-- TODO: keep working on "Red from OAM", a reproducer that races the beam to overwrite tile number and attributes of objects previously seen in Mode 2 -->

### Best practices

This 10-byte routine starts a transfer and waits for it to finish.
Many games copy a routine like it into HRAM and call it during Mode 1.

```rgbasm
run_dma:
    ld a, HIGH(start address)
    ldh [$FF46], a  ; start DMA transfer (starts right after instruction)
    ld a, 40        ; delay for a total of 4×40 = 160 M-cycles
.wait
    dec a           ; 1 M-cycle
    jr nz, .wait    ; 3 M-cycles
    ret
```

If HRAM is tight, this more compact procedure saves 5 bytes of HRAM
at the cost of a few M-cycles spent jumping to the tail in HRAM.

```rgbasm
run_dma:          ; This part must be in ROM.
    ld a, HIGH(start address)
    ld bc, $2846  ; B: wait time; C: LOW($FF46)
    jp run_dma_tail


run_dma_tail:     ; This part must be in HRAM.
    ldh [c], a
.wait
    dec b
    jr nz, .wait
    ret z         ; Conditional `ret` is 1 M-cycle slower, which avoids
                  ; reading from the stack on the last M-cycle of DMA.
```

If starting a mid-frame transfer, wait for Mode 0 first
so that the transfer cleanly overlaps Mode 2 on the next two lines,
making objects invisible on those lines.

_Fonte desta seção: [`src/OAM_DMA_Transfer.md`](https://github.com/gbdev/pandocs/blob/fe246067b695b5404a4a6a47efb4fd6d921ececb/src/OAM_DMA_Transfer.md)_


---

<!-- fonte: src/Window.md @ fe246067b695 -->

## Window behavior

### FF4A–FF4B — WY, WX: Window Y position, X position plus 7

These two registers specify the on-screen coordinates of [the Window]'s top-left pixel.

The Window is visible (if enabled) when `WX` and `WY` are in the range \[0; 166\] and \[0; 143\] respectively.
Values `WX`=7, `WY`=0 place the Window at the top left of the screen, completely covering the background.

### Window mid-frame behavior

While the Window should work as just mentioned, writing to `WX`, `WY` etc. mid-frame displays more articulated behavior.
There are several aspects of the window that respond differently to various mid-frame interactions; the **tl;dr** is this:

- For the least glitchy results, only write to `WX`, `WY`, and `LCDC` during VBlank (possibly in your [VBlank interrupt handler]); if mid-frame writes are required, prefer writing during HBlank.
- If intending to hide the Window for part of the screen (e.g. to have a status bar at the *top* of the screen instead of the bottom), hide it by setting `WX` to a high value rather than writing to `LCDC`.

#### Window rendering criteria

The PPU keeps track of a “**Y condition**” throughout a frame.

- On each VBlank, the *Y condition* is cleared (becomes false).
- At the beginning of each scanline, if the value of `WY` is equal to [`LY`], the *Y condition* becomes true (and remains so for subsequent scanlines).

:::tip Note

On GBC, clearing the [Window enable bit] in `LCDC` resets the *Y condition*; `WY` must be set to `LY` or greater for the Window to display again in the current frame.

:::

Additionally, the PPU maintains a counter, initialized to 0 at the beginning of each scanline.
The counter is incremented for each pixel rendered; however, it also increments 7 times before the first pixel is actually rendered (this covers pixels discarded during the initial "fine scroll" adjustment).

When this counter is equal to `WX`, if the *Y condition* is true and the [Window enable bit] is set in `LCDC`, background rendering is reset, beginning anew from the active row of the Window's tilemap.
The coordinate of the active Window row is then incremented.

- This process can happen more than once per scanline, making the Window's "tilemap Y coordinate" increase more than once in the scanline.
  (This is demonstrated by the TODO test ROM.)

  However, this requires "disabling" the Window by briefly clearing its enable bit from `LCDC` first.
- If this process doesn't happen, the Window's "tilemap Y coordinate" does not increase; so, if the Window is hidden (by any means) on a given scanline, the row of pixels rendered the next time it's shown will be the same as if it had not been hidden in the first place, producing a sort of vertical striped stretching:

  ![Visual demonstration](https://github.com/mattcurrie/mealybug-tearoom-tests/raw/master/expected/DMG-blob/m2_win_en_toggle.png?raw=true)
- If `WX` is equal to 0, the Window is switched to before the initial "fine scroll" adjustment, causing it to be shifted left by <math><mi>SCX</mi> <mo>%</mo> <mn>8</mn></math> pixels.
- On monochrome systems, `WX` = 166 (which would normally show a single Window pixel, along the right edge of the screen) exhibits a bug: the Window spans the entire screen, but offset vertically by one scanline.
- On monochrome systems, if the Window is disabled via `LCDC`, but the other conditions are met *and* it would have started rendering exactly on a BG tile boundary, then where it would have started rendering, a single pixel with ID 0 (i.e. drawn as the first entry in [the BG palette]) is inserted; this offsets the remainder of the scanline.[^Window_star_trek]

[^Window_star_trek]: This was discovered as affecting the game *Star Trek 25th anniversary*; more information and a test ROM are available [in this thread](https://github.com/LIJI32/SameBoy/issues/278#issuecomment-1189712129).

[the Window]: #Window
[VBlank interrupt handler]: <#INT $40 — VBlank interrupt>
[Window enable bit]: <#LCDC.5 — Window enable>
[`LY`]: <#FF44 — LY: LCD Y coordinate \[read-only\]>
[the BG palette]: <#FF47 — BGP (Non-CGB Mode only): BG palette data>

_Fonte desta seção: [`src/Window.md`](https://github.com/gbdev/pandocs/blob/fe246067b695b5404a4a6a47efb4fd6d921ececb/src/Window.md)_


---

<!-- fonte: src/LCDC.md @ fe246067b695 -->

## LCD Control

### FF40 — LCDC: LCD control

**LCDC** is the main **LCD C**ontrol register. Its bits toggle what
elements are displayed on the screen, and how.

Layout de bits (8 bits):

| Bits | Campo |
|---|---|
| 7 | LCD & PPU enable |
| 6 | Window tile map |
| 5 | Window enable |
| 4 | BG & Window tiles |
| 3 | BG tile map |
| 2 | OBJ size |
| 1 | OBJ enable |
| 0 | BG & Window enable / priority |


- **[LCD & PPU enable](https://gbdev.io/pandocs/single.html#lcdc7--lcd-enable)**: `0` = Off; `1` = On
- **[Window tile map area](https://gbdev.io/pandocs/single.html#lcdc6--window-tile-map-area)**: `0` = 9800–9BFF; `1` = 9C00–9FFF
- **[Window enable](https://gbdev.io/pandocs/single.html#lcdc5--window-enable)**: `0` = Off; `1` = On
- **[BG & Window tile data area](https://gbdev.io/pandocs/single.html#lcdc4--bg-and-window-tile-data-area)**: `0` = 8800–97FF; `1` = 8000–8FFF
- **[BG tile map area](https://gbdev.io/pandocs/single.html#lcdc3--bg-tile-map-area)**: `0` = 9800–9BFF; `1` = 9C00–9FFF
- **[OBJ size](https://gbdev.io/pandocs/single.html#lcdc2--obj-size)**: `0` = 8×8; `1` = 8×16
- **[OBJ enable](https://gbdev.io/pandocs/single.html#lcdc1--obj-enable)**: `0` = Off; `1` = On
- **[BG & Window enable / priority](https://gbdev.io/pandocs/single.html#lcdc0--bg-and-window-enablepriority)** *\[Different meaning in CGB Mode\]*: `0` = Off; `1` = On

#### LCDC.7 — LCD enable

This bit controls whether the LCD is on and the PPU is active. Setting
it to 0 turns both off, which grants immediate and full access to VRAM,
OAM, etc.

:::danger CAUTION

Stopping LCD operation (Bit 7 from 1 to 0) may be performed
during VBlank ONLY, disabling the display outside
of the VBlank period may damage the hardware by burning in a black
horizontal line similar to that which appears when the GB is turned off.
This appears to be a serious issue. Nintendo is reported to reject any
games not following this rule.

:::

When the display is disabled the screen is blank, which on DMG is
displayed as a white "whiter" than color \#0.

On SGB, the screen doesn't turn white, it appears that the previous
picture sticks to the screen. (TODO: research this more.)

When re-enabling the LCD, the PPU will immediately start drawing again,
but the screen will stay blank during the first frame.

#### LCDC.6 — Window tile map area

This bit controls which background map the Window uses for rendering.
When it's clear (0), the $9800 tilemap is used, otherwise it's the $9C00
one.

#### LCDC.5 — Window enable

This bit controls whether the window shall be displayed or not.
This bit is overridden on DMG by [bit 0](https://gbdev.io/pandocs/single.html#lcdc0--bg-and-window-enablepriority)
if that bit is clear.

Changing the value of this register mid-frame triggers several more complex behaviours:
[see the corresponding chapter](https://gbdev.io/pandocs/single.html#window-mid-frame-behavior).

#### LCDC.4 — BG and Window tile data area

This bit controls which [addressing
mode](https://gbdev.io/pandocs/single.html#vram-tile-data) the BG and Window use to
pick tiles.

Objects (sprites) aren't affected by this, and will always use the $8000 addressing mode.

#### LCDC.3 — BG tile map area

This bit works similarly to [LCDC bit 6](https://gbdev.io/pandocs/single.html#lcdc6--window-tile-map-area):
if the bit is clear (0), the BG uses tilemap $9800, otherwise tilemap $9C00.

#### LCDC.2 — OBJ size

This bit controls the size of all objects (1 tile or 2 stacked vertically).

Be cautious when changing object size mid-frame.
Changing from 8×8 to 8×16 pixels mid-frame within 8 scanlines of the bottom of an object
causes the object's second tile to be visible for the rest of those 8 lines.
If the size is changed during mode 2 or 3,
remnants of objects in range could "leak" into the other tile and
cause artifacts.

#### LCDC.1 — OBJ enable

This bit toggles whether objects are displayed or not.

This can be toggled mid-frame, for example to avoid objects being
displayed on top of a status bar or text box.

(Note: toggling mid-scanline might have funky results on DMG?
Investigation needed.)

#### LCDC.0 — BG and Window enable/priority

LCDC.0 has different meanings depending on Game Boy type and Mode:

##### Non-CGB Mode (DMG, SGB and CGB in compatibility mode): BG and Window display

When Bit 0 is cleared, both background and window become blank (white),
and the [Window Display Bit](https://gbdev.io/pandocs/single.html#lcdc5--window-enable)
is ignored in that case. Only objects may still be displayed (if enabled
in Bit 1).

##### CGB Mode: BG and Window master priority

When Bit 0 is cleared, the background and window lose their priority -
the objects will be always displayed on top of background and window,
independently of the priority flags in OAM and BG Map attributes.

When Bit 0 is set, pixel priority is resolved [as described here](https://gbdev.io/pandocs/single.html#bg-to-obj-priority-in-cgb-mode).

### Using LCDC

LCDC is a powerful tool: each bit controls a lot of behavior, and can be
modified at any time during the frame.

One of the important aspects of LCDC is that unlike VRAM, the PPU never
locks it. It's thus possible to modify it mid-scanline!

### Faux-layer textbox/status bar

A problem often seen in 8-bit games is objects rendering on top
of the textbox/status bar. It's possible to prevent this using LCDC if
the textbox/status bar is "alone" on its scanlines:

- Set LCDC.1 to 1 for gameplay scanlines
- Set LCDC.1 to 0 for textbox/status bar scanlines

Usually, these bars are either at the top or bottom of the screen, so
the bit can be set by the VBlank and/or STAT handlers.
Hiding objects behind a right-side window is more challenging.

_Fonte desta seção: [`src/LCDC.md`](https://github.com/gbdev/pandocs/blob/fe246067b695b5404a4a6a47efb4fd6d921ececb/src/LCDC.md)_


---

<!-- fonte: src/STAT.md @ fe246067b695 -->

## LCD Status Registers

:::tip TERMINOLOGY

A *dot* is the shortest period over which the PPU can output one pixel: is it equivalent to 1 T-cycle on DMG or on CGB Normal Speed mode or 2 T-cycles on CGB Double Speed mode. On each dot during mode 3, either the PPU outputs a pixel or the fetcher is stalling the [FIFOs](https://gbdev.io/pandocs/single.html#pixel-fifo).

:::

### FF44 — LY: LCD Y coordinate \[read-only\]

LY indicates the current horizontal line, which might be about to be drawn,
being drawn, or just been drawn. LY can hold any value from 0 to 153, with
values from 144 to 153 indicating the VBlank period.

### FF45 — LYC: LY compare

The Game Boy constantly compares the value of the LYC and LY registers.
When both values are identical, the "LYC=LY" flag in the STAT register
is set, and (if enabled) a STAT interrupt is requested.

### FF41 — STAT: LCD status

Layout de bits (8 bits):

| Bits | Campo |
|---|---|
| 6 | LYC int select |
| 5 | Mode 2 int select |
| 4 | Mode 1 int select |
| 3 | Mode 0 int select |
| 2 | LYC == LY |
| 1-0 | PPU mode |


- **LYC int select** (*Read/Write*): If set, selects the `LYC` == `LY` condition for [the STAT interrupt](https://gbdev.io/pandocs/single.html#int-48--stat-interrupt).
- **Mode 2 int select** (*Read/Write*): If set, selects the Mode 2 condition for [the STAT interrupt](https://gbdev.io/pandocs/single.html#int-48--stat-interrupt).
- **Mode 1 int select** (*Read/Write*): If set, selects the Mode 1 condition for [the STAT interrupt](https://gbdev.io/pandocs/single.html#int-48--stat-interrupt).
- **Mode 0 int select** (*Read/Write*): If set, selects the Mode 0 condition for [the STAT interrupt](https://gbdev.io/pandocs/single.html#int-48--stat-interrupt).
- **LYC == LY** (*Read-only*): Set when [LY](https://gbdev.io/pandocs/single.html#ff44--ly-lcd-y-coordinate-read-only) contains the same value as [LYC](https://gbdev.io/pandocs/single.html#ff45--lyc-ly-compare); it is constantly updated.
- **PPU mode** (*Read-only*): Indicates [the PPU's current status](https://gbdev.io/pandocs/single.html#ppu-modes). Reports 0 instead when the [PPU is disabled](https://gbdev.io/pandocs/single.html#lcdc7--lcd-enable).
  
#### Spurious STAT interrupts

A hardware quirk in the monochrome Game Boy makes the LCD interrupt
sometimes trigger when writing to STAT (including writing $00) during
OAM scan, HBlank, VBlank, or LY=LYC. It behaves as if $FF were
written for one M-cycle, and then the written value were written the next
M-cycle. Because the GBC in DMG mode does not have this quirk, two games
that depend on this quirk (Ocean's *Road Rash* and Vic Tokai's *Xerd
no Densetsu*) will not run on a GBC.

_Fonte desta seção: [`src/STAT.md`](https://github.com/gbdev/pandocs/blob/fe246067b695b5404a4a6a47efb4fd6d921ececb/src/STAT.md)_


---

<!-- fonte: src/Scrolling.md @ fe246067b695 -->

## Viewport position (Scrolling)

These registers can be accessed even during Mode 3, but modifications may not take
effect immediately (see further below).

### FF42–FF43 — SCY, SCX: Background viewport Y position, X position

These two registers specify the top-left coordinates of the visible 160×144 pixel area within the
256×256 pixels BG map. Values in the range 0–255 may be used.

The PPU calculates the bottom-right coordinates of the viewport with those formulas: `bottom := (SCY + 143) % 256` and `right := (SCX + 159) % 256`.
As suggested by the modulo operations, in case the values are larger than 255 they will "wrap around" towards the top-left corner of the tilemap.

<figure><figcaption>

Example from the homebrew game *Mindy's Hike*:

</figcaption>

![VRAM view diagram](imgs/scrolling_diagram.png)

</figure>

### Mid-frame behavior

The scroll registers are re-read on each [tile fetch](https://gbdev.io/pandocs/single.html#get-tile), except for the low 3 bits of `SCX`, which are only read at the beginning of the scanline (for the initial shifting of pixels).

All models before the CGB-D read the Y coordinate once for each bitplane (so a very precisely timed `SCY` write allows "desyncing" them), but CGB-D and later use the same Y coordinate for both no matter what.

_Fonte desta seção: [`src/Scrolling.md`](https://github.com/gbdev/pandocs/blob/fe246067b695b5404a4a6a47efb4fd6d921ececb/src/Scrolling.md)_


---

<!-- fonte: src/Palettes.md @ fe246067b695 -->

## Palettes

### LCD Monochrome Palettes

Non-Color games have access to one palette for the background (and Window), and two for OBJs.

In CGB Mode the color palettes are taken from [CGB palette memory](https://gbdev.io/pandocs/single.html#lcd-color-palettes-cgb-only)
instead.

#### FF47 — BGP (Non-CGB Mode only): BG palette data

This register assigns gray shades to the [color indices](https://gbdev.io/pandocs/single.html#data-format) of the BG and Window tiles.

**`Color for...`** — layout de bits (8 bits):

| Bits | Campo |
|---|---|
| 7-6 | ID 3 |
| 5-4 | ID 2 |
| 3-2 | ID 1 |
| 1-0 | ID 0 |


Each of the two-bit values map to a color thusly:

Value | Color
------|-------
  0   | White
  1   | Light gray
  2   | Dark gray
  3   | Black

#### FF48–FF49 — OBP0, OBP1 (Non-CGB Mode only): OBJ palette 0, 1 data

These registers assigns gray shades to the color indexes of the OBJs that use the corresponding palette.
They work exactly like [`BGP`](https://gbdev.io/pandocs/single.html#ff47--bgp-non-cgb-mode-only-bg-palette-data), except that the lower two bits are ignored because color index 0 is transparent for OBJs.

### LCD Color Palettes (CGB only)

The GBC provides 8 palettes for the background (and Window), and 8 for OBJs; they are selected via the [attribute maps](https://gbdev.io/pandocs/single.html#bg-map-attributes-cgb-mode-only) and [OAM attributes](https://gbdev.io/pandocs/single.html#byte-3--attributesflags) respectively.

:::tip NOTE

All background colors are initialized as white [by the boot ROM](https://gbdev.io/pandocs/single.html#power-up-sequence).

:::

Colors on the Game Boy Color are stored as RGB555, meaning a single color is composed of three 5-bit components, one for each of red, green, and blue.
Each 15-bit color occupies the lower part of a 16-bit word[^Palettes_bit15]:

> _Diagrama `imgs/src/rgb555.svg` omitido nesta cópia offline (era um SVG do Pan Docs). Ver https://gbdev.io/pandocs/single.html_

The color palettes are stored in two dedicated banks of palette RAM (or <abbr title="Color RAM">CRAM</abbr> for *color RAM*), 64 bytes each[^Palettes_cram_size]: one for background/window palettes and the other for OBJ palettes.

The two bytes of each color are stored in **little-endian** byte order, meaning that the low byte comes first.
For example, the two palettes shown in the previous diagram would be stored like this:

> _Diagrama `imgs/src/color_ram.svg` omitido nesta cópia offline (era um SVG do Pan Docs). Ver https://gbdev.io/pandocs/single.html_

Unlike VRAM, OAM, or wave RAM, CRAM is not exposed in the memory map and cannot be accessed directly.
Instead, each bank of CRAM is accessed through a pair of registers: one register is used to select a CRAM address, and the other provides read/write access to the byte at that address.
Much like VRAM, the CRAM data registers are inaccessible when the PPU is reading from CRAM, that is, during [Mode 3](https://gbdev.io/pandocs/single.html#ppu-modes): writes are ignored, and reads return $FF.

[^Palettes_bit15]:
The 16th bit, bit 15, is **ignored** by the rendering process.
Conventionally, that bit is generally clear (for example, the canonical pure white is `$7FFF` and not `$FFFF`), but the hardware treats both identically: it's fine to fill color RAM with $FF bytes to set it to all-white.

[^Palettes_cram_size]:
2 bytes/color × 4 colors/palette × 8 palettes = 64 bytes.

#### FF68 — BGPI (CGB Mode only): Background palette index

**`BGPI`** — layout de bits (8 bits):

| Bits | Campo |
|---|---|
| 7 | Auto-increment |
| 5-0 | Address |


- **Auto-increment**: `0` = Disabled; `1` = Enabled
- **Address**: Specifies which byte of BG Palette Memory can be accessed through
  [`BGPD`](https://gbdev.io/pandocs/single.html#ff69--bgpd-cgb-mode-only-background-palette-data)

Unlike `BGPD`, this register can be freely accessed outside VBlank and HBlank.

#### FF69 — BGPD (CGB Mode only): Background palette data

As each color is two bytes in size, you must read/write this register *twice* to access a whole color.

This is made much easier through the use of the address auto-increment: `BGPI`'s "address" field is automatically incremented (wrapping around from 63 back to 0) after each write to this register, even if the write fails due to CRAM being inaccessible.
Reads, however, never trigger auto-increment.

#### FF6A–FF6B — OBPI, OBPD (CGB Mode only): OBJ palette index, OBJ palette data

These registers function exactly like BGPI and BGPD respectively; the 64 bytes of OBJ palette memory are entirely separate from Background palette memory, but function the same.

Note that while 4 colors are stored per OBJ palette, color #0 is never used, as it's always transparent. It's thus fine to write garbage values, or even leave it uninitialized.

:::tip NOTE

In CGB mode, the boot ROM leaves all object colors uninitialized (and thus somewhat random/unreliable), aside from setting the first byte of OBJ0 color #0 to $00, which is unused.

In DMG compatibility mode, the boot ROM sets the first 2 object palettes which are used by OBP0/OBP1, [as explained here](https://gbdev.io/pandocs/single.html#compatibility-palettes).

:::

#### RGB Translation by CGBs

![sRGB versus CGB color mixing](imgs/VGA_versus_CGB.png)

When developing graphics on PCs, note that the RGB values will have
different appearance on CGB displays as on VGA/HDMI monitors calibrated
to sRGB color. Because the GBC is not lit, the highest intensity will
produce light gray rather than white. The intensities are not
linear; the values $10-$1F will all appear very bright, while medium and
darker colors are ranged at $00-0F.

The CGB display's pigments aren't perfectly saturated. This means the
colors mix quite oddly: increasing the intensity of only one R/G/B color
will also influence the other two R/G/B colors. For example, a color
setting of $03EF (Blue=$00, Green=$1F, Red=$0F) will appear as Neon Green
on VGA displays, but on the CGB it'll produce a decently washed out
Yellow. See the image above.

#### RGB Translation by GBAs

Even though GBA is described to be compatible to CGB games, most CGB
games are completely unplayable on older GBAs because most colors are
invisible (black). Of course, colors such like Black and White will
appear the same on both CGB and GBA, but medium intensities are arranged
completely different. Intensities in range $00–07 are invisible/black
(unless eventually under best sunlight circumstances, and when gazing at
the screen under obscure viewing angles), unfortunately, these
intensities are regularly used by most existing CGB games for medium and
darker colors.

:::tip WORKAROUND

Newer CGB games may avoid this effect by changing palette data when
detecting GBA hardware ([see how](https://gbdev.io/pandocs/single.html#detecting-cgb-and-gba-functions)).
Based on measurements of GBC and GBA palettes using the
[144p Test Suite](https://github.com/pinobatch/240p-test-mini/tree/master/gameboy),
a fairly close approximation is `GBA = GBC × 3/4 + $08` for each R/G/B
component. The result isn't quite perfect, and it may turn
out that the color mixing is different also; anyways, it'd be still
ways better than no conversion.

:::

This problem with low brightness levels does not affect later GBA SP
units and Game Boy Player. Thus ideally, the player should have control
of this brightness correction.

_Fonte desta seção: [`src/Palettes.md`](https://github.com/gbdev/pandocs/blob/fe246067b695b5404a4a6a47efb4fd6d921ececb/src/Palettes.md)_


---

<!-- fonte: src/Rendering.md @ fe246067b695 -->

## Rendering overview

### Terminology

The entire frame is not drawn atomically; instead, the image is drawn by the **<abbr>PPU</abbr>** (Pixel-Processing Unit) progressively, **directly to the screen**.
A frame consists of 154 **scanlines**; during the first 144, the screen is drawn top to bottom, left to right.

The main implication of this rendering process is the existence of **raster effects**: modifying some rendering parameters in the middle of rendering.
The most famous raster effect is modifying the [scrolling registers](https://gbdev.io/pandocs/single.html#viewport-position-scrolling) between scanlines to create a ["wavy" effect](https://gbdev.io/guides/deadcscroll#effects).

A "**dot**" = one 2<sup>22</sup> Hz (≅ 4.194 MHz) time unit.
Dots remain the same regardless of whether the CPU is in [Double Speed mode](https://gbdev.io/pandocs/single.html#ff4d--key1spd-cgb-mode-only-prepare-speed-switch), so there are 4 dots per Normal Speed M-cycle, and 2 per Double Speed M-cycle.

:::tip NOTE

A frame is not exactly one 60<sup>th</sup> of a second: the Game Boy runs slightly slower than 60 Hz, as one frame takes ~16.74 ms instead of ~16.67 (the error is 0.45%).

:::

### PPU modes

<figure><figcaption>

During a frame, the Game Boy's PPU cycles between four modes as follows:

</figcaption>

> _Diagrama `imgs/src/ppu_modes_timing.svg` omitido nesta cópia offline (era um SVG do Pan Docs). Ver https://gbdev.io/pandocs/single.html_

</figure>

While the PPU is accessing some video-related memory, [that memory is inaccessible to the CPU](https://gbdev.io/pandocs/single.html#accessing-vram-and-oam) (writes are ignored, and reads return garbage values, usually $FF).

Mode | Action                                     | Duration                             | Accessible video memory
----:|--------------------------------------------|--------------------------------------|-------------------------
  2  | Searching for OBJs which overlap this line | 80 dots                              | VRAM, CGB palettes
  3  | Sending pixels to the LCD                  | Between 172 and 289 dots, see below  | None
  0  | Waiting until the end of the scanline      | 376 - mode 3's duration              | VRAM, OAM, CGB palettes
  1  | Waiting until the next frame               | 4560 dots (10 scanlines)             | VRAM, OAM, CGB palettes

### Mode 3 length

During Mode 3, by default the PPU outputs one pixel to the screen per dot, from left to right; the screen is 160 pixels wide, so the minimum Mode 3 length is 160 + 12[^Rendering_first12] = 172 dots.

Unlike most game consoles, the Game Boy does not always output pixels steadily[^Rendering_crt]: some features cause the rendering process to stall for a couple dots.
Any extra time spent stalling *lengthens* Mode 3; but since scanlines last for a fixed number of dots, Mode 0 is therefore shortened by that same amount of time.

Three things can cause Mode 3 "penalties":

- **Background scrolling**: At the very beginning of Mode 3, rendering is paused for [`SCX`](https://gbdev.io/pandocs/single.html#ff42ff43--scy-scx-background-viewport-y-position-x-position) % 8 dots while the same number of pixels are discarded from the leftmost tile.
- **Window**: After the last non-window pixel is emitted, a 6-dot penalty is incurred while the BG fetcher is being set up for the window.
- **Objects**: Each object drawn during the scanline (even partially) incurs a 6- to 11-dot penalty ([see below](https://gbdev.io/pandocs/single.html#obj-penalty-algorithm)).

On DMG and GBC in DMG mode, mid-scanline writes to [`BGP`](https://gbdev.io/pandocs/single.html#ff47--bgp-non-cgb-mode-only-bg-palette-data) allow observing this behavior precisely, as any delay shifts the write's effect to the left by that many dots.

#### OBJ penalty algorithm

Only the OBJ's leftmost pixel matters here, transparent or not; it is designated as "The Pixel" in the following.

1. Determine the tile (background or window) that The Pixel is within. (This is affected by horizontal scrolling and/or the window!)
2. If that tile has **not** been considered by a previous OBJ yet[^Rendering_order]:
   1. Count how many of that tile's pixels are strictly to the right of The Pixel.
   2. Subtract 2.
   3. Incur this many dots of penalty, or zero if negative (from waiting for the BG fetch to finish).
3. Incur a flat, 6-dot penalty (from fetching the OBJ's tile).

**Exception**: an OBJ with an OAM X position of 0 (thus, completely off the left side of the screen) always incurs a 11-dot penalty, regardless of `SCX`.


[^Rendering_first12]: The 12 extra dots of penalty come from two tile fetches at the beginning of Mode 3. One is the first tile in the scanline (the one that gets shifted by `SCX` % 8 pixels), the other is simply discarded.

[^Rendering_crt]: The Game Boy can afford to "take pauses", because it writes to a LCD it fully controls; by contrast, home consoles like the NES or SNES are on a schedule imposed by the screen they are hooked up to. Taking pauses arguably simplified the PPU's design while allowing greater flexibility to game developers.

[^Rendering_order]: Since pixels are emitted from left to right, OBJs overlapping the scanline are considered from [leftmost](https://gbdev.io/pandocs/single.html#byte-1--x-position) to rightmost, with ties broken by their index / OAM address (lowest first).

_Fonte desta seção: [`src/Rendering.md`](https://github.com/gbdev/pandocs/blob/fe246067b695b5404a4a6a47efb4fd6d921ececb/src/Rendering.md)_


---

<!-- fonte: src/pixel_fifo.md @ fe246067b695 -->

## Pixel FIFO

### Introduction

FIFO stands for *First In, First Out*. The first pixel to be pushed to the
FIFO is the first pixel to be popped off. In theory that sounds great,
in practice there are a lot of intricacies.

There are two pixel FIFOs. One for background pixels and one for object
(sprite) pixels. These two FIFOs are not shared. They are independent
of each other. The two FIFOs are mixed only when popping items. Objects
take priority unless they're transparent (color 0) which will be
explained in detail later. Each FIFO can hold up to 16 pixels. The FIFO
and Pixel Fetcher work together to ensure that the FIFO always contains
at least 8 pixels at any given time, as 8 pixels are required for the
Pixel Rendering operation to take place. Each FIFO is manipulated only
during mode 3 (pixel transfer).

Each pixel in the FIFO has four properties:
- Color: a value between 0 and 3
- Palette: on CGB a value between 0 and 7 and on DMG this only applies to objects
- Sprite Priority: on CGB this is the OAM index for the object and on DMG this doesn't exist
- Background Priority: holds the value of the [OBJ-to-BG Priority](https://gbdev.io/pandocs/single.html#object-attribute-memory-oam) bit

### FIFO Pixel Fetcher

The fetcher fetches a row of 8 background or window pixels and queues
them up to be mixed with object pixels. The pixel fetcher has 5 steps.
The first four steps take 2 dots each and the fifth step is attempted
every dot until it succeeds. The order of the steps are as follows:

- Get tile
- Get tile data low
- Get tile data high
- Sleep
- Push

#### Get Tile

This step determines which background/window tile to fetch pixels from.
By default the tilemap used is the one at $9800 but certain conditions
can change that.

When LCDC.3 is enabled and the X coordinate of the current scanline is
not inside the window then tilemap $9C00 is used.

When LCDC.6 is enabled and the X coordinate of the current scanline is
inside the window then tilemap $9C00 is used.

The fetcher keeps track of which X and Y coordinate of the tile it's on:

If the current tile is a window tile, the X coordinate for the window
tile is used, otherwise the following formula is used to calculate
the X coordinate: ((SCX / 8) + fetcher's X coordinate) & $1F. Because of
this formula, fetcherX can be between 0 and 31.

If the current tile is a window tile, the Y coordinate for the window
tile is used, otherwise the following formula is used to calculate
the Y coordinate: (currentScanline + SCY) & 255. Because of this formula,
fetcherY can be between 0 and 255.

The fetcher's X and Y coordinate can then be used to get the tile from
VRAM. However, if the PPU's access to VRAM is [blocked](https://gbdev.io/pandocs/single.html#vram-access)
then the value for the tile is read as $FF.

CGB can access both tile index and the attributes in the same clock
dot.

#### Get Tile Data Low

Check LCDC.4 for which tilemap to use. At this step CGB also needs to
check which VRAM bank to use and check if the tile is flipped vertically.
Once the tilemap, VRAM and vertical flip is calculated the tile data
is retrieved from VRAM. However, if the PPU's access to VRAM is
[blocked](https://gbdev.io/pandocs/single.html#vram-access) then the tile data is read as $FF.

The tile data retrieved in this step will be used in the push steps.

#### Get Tile Data High

Same as Get Tile Data Low except the tile address is incremented by 1.

The tile data retrieved in this step will be used in the push steps.

This also pushes a row of background/window pixels to the FIFO. This
extra push is not part of the 8 steps, meaning there's 3 total chances to
push pixels to the background FIFO every time the complete fetcher steps
are performed.

#### Push

Pushes a row of background/window pixels to the FIFO. Since tiles are 8
pixels wide, a "row" of pixels is 8 pixels from the tile to be rendered
based on the X and Y coordinates calculated in the previous steps.

Pixels are only pushed to the background FIFO if it's empty.

This is where the tile data retrieved in the two Tile Data steps will
come in handy. Depending on if the tile is flipped horizontally the
pixels will be pushed to the background FIFO differently. If the tile
is flipped horizontally the pixels will be pushed LSB first. Otherwise
they will be pushed MSB first.

#### Sleep

Do nothing.

#### VRAM Access

At various times during PPU operation read access to VRAM is blocked and
the value read is $FF:
- LCD turning off
- At scanline 0 on CGB when not in double speed mode
- When switching from mode 3 to mode 0
- On CGB when searching OAM and index 37 is reached

At various times during PPU operation read access to VRAM is restored:
- At scanline 0 on DMG and CGB when in double speed mode
- On DMG when searching OAM and index 37 is reached
- After switching from mode 2 (oam search) to mode 3 (pixel transfer)

NOTE: These conditions are checked only when entering STOP mode and the
PPU's access to VRAM is always restored upon leaving STOP mode.

### Mode 3 Operation

As stated before the pixel FIFO only operates during mode 3 (pixel
transfer). At the beginning of mode 3 both the background and OAM FIFOs
are cleared.

#### The Window

When rendering the window the background FIFO is cleared and the fetcher
is reset to step 1. When WX is 0 and the SCX & 7 > 0 mode 3 is shortened
by 1 dot.

When the window has already started rendering there is a bug that occurs
when WX is changed mid-scanline. When the value of WX changes after the
window has started rendering and the new value of WX is reached again,
a pixel with color value of 0 and the lowest priority is pushed onto the
background FIFO.

#### Sprites

The following is performed for each object on the current scanline if
LCDC.1 is enabled (this condition is ignored on CGB) and the X coordinate
of the current scanline has an object on it. If those conditions are not
met then object fetching is [canceled](https://gbdev.io/pandocs/single.html#object-fetch-canceling).

At this point the [fetcher](https://gbdev.io/pandocs/single.html#fifo-pixel-fetcher) is advanced one step
until it's at step 5 or until the background FIFO is not empty. Advancing
the fetcher one step here lengthens mode 3 by 1 dot. This process may
be [canceled](https://gbdev.io/pandocs/single.html#object-fetch-canceling) after the fetcher has advanced a
step.

When SCX & 7 > 0 and there is an object at X coordinate 0 of the current
scanline then mode 3 is lengthened. The amount of dots this lengthens
mode 3 by is whatever the lower 3 bits of SCX are. After this penalty is
applied object fetching may be canceled. Note that the timing of the
penalty is not confirmed. It may happen before or after waiting for the
fetcher. More research needs to be done.

After checking for objects at X coordinate 0 the fetcher is advanced two
steps. The first advancement lengthens mode 3 by 1 dot and the second
advancement lengthens mode 3 by 3 dots. After each fetcher advancement
there is a chance for an object fetch cancel to occur.

The lower address for the row of pixels of the target object tile is now
retrieved and lengthens mode 3 by 1 dot. Once the address is retrieved
this is the last chance for object fetch cancel to occur. Exiting
object fetch lengthens mode 3 by 1 dot. The upper address for the
target object tile is now retrieved and does not shorten mode 3.

At this point [VRAM Access](https://gbdev.io/pandocs/single.html#vram-access) is checked for the lower and
upper addresses for the target object. Before any mixing is done, if the
OAM FIFO doesn't have at least 8 pixels in it then transparent pixels
with the lowest priority are pushed onto the OAM FIFO. Once this is done
each pixel of the target object row is checked. On CGB, horizontal flip
is checked here. If the target object pixel is not white and the pixel in
the OAM FIFO *is* white, or if the pixel in the OAM FIFO has higher
priority than the target object's pixel, then the pixel in the OAM FIFO
is replaced with the target object's properties.

Now it's time to [render a pixel](https://gbdev.io/pandocs/single.html#pixel-rendering)! The same process
described in Object Fetch Canceling is performed: a pixel is rendered and
the fetcher is advanced one step. This advancement lengthens mode 3 by 1
dot if the X coordinate of the current scanline is not 160. If the X
coordinate is 160 the PPU stops processing objects (because they won't be
visible).

Everything in this section is repeated for every object on the current
scanline unless it was decided that fetching should be canceled or the
X coordinate is 160.

#### Pixel Rendering

This is where the background FIFO and OAM FIFO are mixed. There are
conditions where either a background pixel or an object pixel will have
display priority.

If there are pixels in the background and OAM FIFOs then a pixel is
popped off each. If the OAM pixel is not transparent and LCDC.1 is
enabled then the OAM pixel's background priority property is used if it's
the same or higher priority as the background pixel's background priority.

Pixels won't be pushed to the LCD if there is nothing in the background
FIFO or the current pixel is pixel 160 or greater.

If LCDC.0 is disabled then the background is disabled on DMG and the
background pixel won't have priority on CGB. When the background pixel
is disabled the pixel color value will be 0, otherwise the color value
will be whatever color pixel was popped off the background FIFO. When the
pixel popped off the background FIFO has a color value other than 0 and
it has priority then the object pixel will be discarded.

At this point, on DMG, the color of the pixel is retrieved from the BGP
register and pushed to the LCD. On CGB when [palette access](https://gbdev.io/pandocs/single.html#cgb-palette-access)
is blocked a black pixel is pushed to the LCD.

When an object pixel has priority, the color value is retrieved from the
popped pixel from the OAM FIFO. On DMG the color for the pixel is
retrieved from either the OBP1 or OBP0 register depending on the pixel's
palette property. If the palette property is 1 then OBP1 is used,
otherwise OBP0 is used. The pixel is then pushed to the LCD. On CGB when
palette access is blocked, a black pixel is pushed to the LCD.

The pixel is then finally pushed to the LCD.

#### CGB Palette Access

At various times during PPU operation read access to the CGB palette is
blocked and a black pixel pushed to the LCD when rendering pixels:
- LCD turning off
- First HBlank of the frame
- When searching OAM and index 37 is reached
- After switching from mode 2 (oam search) to mode 3 (pixel transfer)
- When entering HBlank (mode 0) and not in double speed mode, blocked 2 dots later no matter what

At various times during PPU operation read access to the CGB palette is
restored and pixels are pushed to the LCD normally when rendering pixels:
- At the end of mode 2 (oam search)
- For only 2 dots when entering HBlank (mode 0) and in double speed mode

:::tip Note

These conditions are checked only when entering STOP mode and the
PPU's access to CGB palettes is always restored upon leaving STOP mode.

:::

#### Object Fetch Canceling

Object fetching may be canceled if LCDC.1 is disabled while the PPU is
fetching an object from OAM. This canceling lengthens mode 3 by the amount
of dots the previous instruction took plus the residual dots left for
the PPU to process. When OAM fetching is canceled, a pixel is [rendered](https://gbdev.io/pandocs/single.html#pixel-rendering), and
the [fetcher](https://gbdev.io/pandocs/single.html#fifo-pixel-fetcher) is advanced one step. This advancement
lengthens mode 3 by 1 dot if the current pixel is not 160. If the
current pixel is 160 the PPU stops processing objects because they won't
be visible.

_Fonte desta seção: [`src/pixel_fifo.md`](https://github.com/gbdev/pandocs/blob/fe246067b695b5404a4a6a47efb4fd6d921ececb/src/pixel_fifo.md)_


---

<!-- fonte: src/Accessing_VRAM_and_OAM.md @ fe246067b695 -->

## Accessing VRAM and OAM

:::warning Warning

When the PPU is [drawing the screen](https://gbdev.io/pandocs/single.html#rendering-overview), it is often directly reading from Video Memory (VRAM) and from the Object Attribute Memory (OAM).
During these periods, the Game Boy CPU cannot access VRAM and OAM.

That means that any attempts to write to VRAM or OAM are ignored (data remains unchanged).
And any attempts to read from VRAM or OAM will return undefined data (typically $FF).

For this reason the program should verify if VRAM/OAM is accessible before actually reading or writing to it.
This is usually done by reading the Mode bits from [the STAT Register](https://gbdev.io/pandocs/single.html#ff41--stat-lcd-status).
When doing this (as described in the examples below) you should take care that **no interrupts occur between the wait loops and the following memory access**;
the memory is guaranteed to be accessible only for a few cycles (less than Mode 2's length) just after a wait loop exits.

:::

### VRAM (memory area at $8000-$9FFF) is accessible during Modes 0-2

```
Mode 0 - HBlank Period,
Mode 1 - VBlank Period, and
Mode 2 - Searching OAM Period
```

A typical procedure that waits for accessibility of VRAM would be:

```rgbasm
    ld   hl, $FF41     ; STAT Register
.wait
    bit  1, [hl]       ; Wait until Mode is 0 or 1
    jr   nz, .wait
```

Even if the procedure gets executed at the *end* of Mode 0 or 1, it is
still safe to assume that VRAM can be accessed for a few more cycles
because in either case the following period is Mode 2, which allows
access to VRAM also. However, be careful about STAT interrupts or
other interrupts that could cause the PPU to be back in Mode 3 by the
time it returns. In CGB Mode an alternate method to write data to VRAM
is to use the HDMA Function (FF51-FF55).

If you do not require any STAT interrupts, another way to synchronize to the
start of Mode 0 is to disable all the individual STAT interrupts except Mode 0
(STAT bit 3), enable STAT interrupts (IE bit 1), disable IME (by executing `di`),
and use the `halt` instruction. This allows
use of the entire Mode 0 on one line and Mode 2 on the following line,
which sum to 165 to 288 dots. For comparison, at normal speed (4 dots
per machine cycle), a copy from stack that takes
9 cycles per 2 bytes can push 8 bytes (half a tile) in 144 dots, which
fits within the worst case timing for mode 0+2.

### OAM (memory area at $FE00-$FE9F) is accessible during Modes 0-1

```
Mode 0 - HBlank Period
Mode 1 - VBlank Period
```

During those modes, OAM can be accessed directly or by doing a DMA
transfer (FF46). Outside those modes, DMA out-prioritizes the PPU in
accessing OAM, and the PPU will read $FF from OAM during that time.

A typical procedure that waits for accessibility of OAM would be:

```rgbasm
    ld   hl, $FF41    ; STAT Register
    ; Wait until Mode is -NOT- 0 or 1
.waitNotBlank
    bit  1, [hl]
    jr   z, .waitNotBlank
    ; Wait until Mode 0 or 1 -BEGINS- (but we know that Mode 0 is what will begin)
.waitBlank
    bit  1, [hl]
    jr   nz, .waitBlank
```

The two wait loops ensure that Mode 0 (and Mode 1 if we are at the end
of a frame) will last for a few clock
cycles after completion of the procedure. If we need to wait for the VBlank period, it would be
better to skip the whole procedure, and use a STAT interrupt instead. In any case,
doing a DMA transfer is more efficient than writing to OAM directly.

:::tip NOTE

While the display is disabled, both VRAM and OAM are accessible.
The downside is that the screen is blank (white) during this
period, so disabling the display would be recommended only during
initialization.

:::

_Fonte desta seção: [`src/Accessing_VRAM_and_OAM.md`](https://github.com/gbdev/pandocs/blob/fe246067b695b5404a4a6a47efb4fd6d921ececb/src/Accessing_VRAM_and_OAM.md)_


---

<!-- fonte: src/OAM_Corruption_Bug.md @ fe246067b695 -->

## OAM Corruption Bug

There is a flaw in the Game Boy hardware that causes rubbish data to be written
to object attribute memory (OAM) if the following instructions are used while their 16-bit content
(before the operation) is in the range $FE00&ndash;$FEFF and the PPU is in mode 2:

```rgbasm
 inc rr         dec rr       ; rr = bc, de, or hl
 ld a, [hli]    ld a, [hld]
 ld [hli], a    ld [hld], a
```

Objects 0 and 1 ($FE00 & $FE04) are not affected by this bug.

Game Boy Color and Advance are not affected by this bug, even when
running monochrome software.

### Accurate Description

The OAM Corruption Bug (or OAM Bug) actually consists of two different bugs:

- Attempting to read or write from OAM (Including the $FEA0-$FEFF
  region) while the PPU is in mode 2 (OAM scan) will corrupt it.
- Performing an increase or decrease operation on any 16-bit register
  (BC, DE, HL, SP or PC) while that register is in the OAM range
  ($FE00–$FEFF) will trigger an access to OAM, causing a corruption.
  This happens because the CPU's increment and decrement unit (IDU)
  for 16-bit numbers is directly tied to the address bus.
  During IDU operation, the value is output as an address,
  even if a read or write is not asserted.

### Affected Operations

The following operations are affected by this bug:

- Any memory access instruction, if it accesses OAM
- `inc rr`, `dec rr` - if `rr` is a 16-bit register pointing to OAM,
  it will trigger a write and corrupt OAM
- `ld [hli], a`, `ld [hld], a`, `ld a, [hli]`, `ld a, [hld]`- these
  will trigger a corruption twice if `hl` points to OAM; once for the
  usual memory access, and once for the extra write triggered by the
  `inc`/`dec`
- `pop rr`, the `ret` family - For some reason, `pop` will trigger the
  bug only 3 times (instead of the expected 4 times); one read, one
  glitched write, and another read without a glitched write. This also
  applies to the `ret` instructions.
- `push rr`, the `call` family, `rst xx` and interrupt handling -
  Pushing to the stack will trigger the bug 4 times; two usual writes
  and two glitched writes caused by the implied `dec sp`. However, since one
  glitched write occurs in the same M-cycle as a actual write, this will
  effectively behave like 3 writes.
- Executing code from OAM - If PC is inside OAM (reading $FF,
  that is, `rst $38`) the bug will trigger twice, once for increasing PC
  inside OAM (triggering a write), and once for reading from OAM. If a
  multi-byte opcode is executed from $FDFF or $FDFE, and bug will
  similarly trigger twice for every read from OAM.

### Corruption Patterns

The OAM is split into 20 rows of 8 bytes each, and during mode 2 the PPU
reads those rows consecutively; one every 1 M-cycle. The operations
patterns rely on type of operation (read/write/both) used on OAM during
that M-cycle, as well as the row currently accessed by the PPU. The
actual read/write address used, or the written value have no effect.
Additionally, keep in mind that OAM uses a 16-bit data bus, so all
operations are on 16-bit words.

#### Write Corruption

A "write corruption" corrupts the currently access row in the following
manner, as long as it's not the first row (containing the first two
objects):

- The first word in the row is replaced with this bitwise expression:
  `((a ^ c) & (b ^ c)) ^ c`, where `a` is the original value of that
  word, `b` is the first word in the preceding row, and `c` is the
  third word in the preceding row.
- The last three words are copied from the last three words in the
  preceding row.

#### Read Corruption

A "read corruption" works similarly to a write corruption, except the
bitwise expression is `b | (a & c)`.

#### Write During Increase/Decrease

If a register is increased or decreased in the same M-cycle of a write,
this will effectively trigger two writes in a single M-cycle. However,
this case behaves just like a single write.

#### Read During Increase/Decrease

If a register is increased or decreased in the same M-cycle of a write,
this will effectively trigger both a read **and** a write in a single
M-cycle, resulting in a more complex corruption pattern:

- This corruption will not happen if the accessed row is one of the
  first four, as well as if it's the last row:
  - The first word in the row preceding the currently accessed row
    is replaced with the following bitwise expression:
    `(b & (a | c | d)) | (a & c & d)` where `a` is the first word
    two rows before the currently accessed row, `b` is the first
    word in the preceding row (the word being corrupted), `c` is the
    first word in the currently accessed row, and `d` is the third
    word in the preceding row.
  - The contents of the preceding row is copied (after the
    corruption of the first word in it) both to the currently
    accessed row and to two rows before the currently accessed row
- Regardless of whether the previous corruption occurred or not, a
  normal read corruption is then applied.

_Fonte desta seção: [`src/OAM_Corruption_Bug.md`](https://github.com/gbdev/pandocs/blob/fe246067b695b5404a4a6a47efb4fd6d921ececb/src/OAM_Corruption_Bug.md)_


---
