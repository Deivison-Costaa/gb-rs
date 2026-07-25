# Cartucho — cabeçalho e MBCs

> **Fonte de verdade deste projeto (regra R1 do CLAUDE.md).**
> Cópia do [Pan Docs](https://gbdev.io/pandocs/) (domínio público, CC0),
> fixada no commit [`fe246067b695`](https://github.com/gbdev/pandocs/tree/fe246067b695b5404a4a6a47efb4fd6d921ececb).
> Não editar à mão: regenerado por `scripts/fetch-reference-docs.sh`.

Cobre o ROADMAP 0.3, 0.4 e todo o marco M5. Inclui MBC1, MBC2, MBC3 e MBC5.

**Nesta página:**

- The Cartridge Header
- MBCs
- nombc
- MBC1
- MBC2
- MBC3
- MBC5

---

<!-- fonte: src/The_Cartridge_Header.md @ fe246067b695 -->

## The Cartridge Header

Each cartridge contains a header, located at the address range `$0100`—`$014F`.
The cartridge header provides the following information about the game itself and the hardware it expects to run on:

### 0100-0103 — Entry point

After displaying the Nintendo logo, the built-in [boot ROM](https://gbdev.io/pandocs/single.html#power-up-sequence) jumps to the address `$0100`, which should then jump to the actual main program in the cartridge.
Most commercial games fill this 4-byte area with a [`nop` instruction](https://rgbds.gbdev.io/docs/gbz80.7#NOP) followed by a [`jp $0150`](https://rgbds.gbdev.io/docs/gbz80.7#JP_n16).

### 0104-0133 — Nintendo logo

This area contains a bitmap image that is displayed when the Game Boy is powered on.
It must match the following (hexadecimal) dump, otherwise [the boot ROM](https://gbdev.io/pandocs/single.html#power-up-sequence) won't allow the game to run:

```
CE ED 66 66 CC 0D 00 0B 03 73 00 83 00 0C 00 0D
00 08 11 1F 88 89 00 0E DC CC 6E E6 DD DD D9 99
BB BB 67 63 6E 0E EC CC DD DC 99 9F BB B9 33 3E
```

The way the pixels are encoded is as follows: ([more visual aid](https://codeberg.org/ISSOtm/gb-bootroms/src/commit/2dce25910043ce2ad1d1d3691436f2c7aabbda00/src/dmg.asm#L259-L269))

- The bytes `$0104`—`$011B` encode the top half of the logo while the bytes `$011C`–`$0133` encode the bottom half.
- For each half, each nibble encodes 4 pixels (the MSB corresponds to the leftmost pixel, the LSB to the rightmost); a pixel is lit if the corresponding bit is set.
- The 4-pixel "groups" are laid out top to bottom, left to right.
- Finally, the monochrome models upscale the entire thing by a factor of 2 (leading to somewhat chunky pixels).

The Game Boy's boot procedure [first displays the logo and then checks](https://gbdev.io/pandocs/single.html#bypass) that it matches the dump above.
If it doesn't, the boot ROM **locks itself up**.

The CGB and later models [only check the top half of the logo](Power_Up_Sequence.html?highlight=half#behavior) (the first `$18` bytes).

### 0134-0143 — Title

These bytes contain the title of the game in upper case ASCII.
If the title is less than 16 characters long, the remaining bytes should be padded with `$00`s.

Parts of this area actually have a different meaning on later cartridges, reducing the actual title size to 15 (`$0134`–`$0142`) or 11 (`$0134`–`$013E`) characters; see below.

### 013F-0142 — Manufacturer code

In older cartridges these bytes were part of the Title (see above).
In newer cartridges they contain a 4-character manufacturer code (in uppercase ASCII).
The purpose of the manufacturer code is unknown.

### 0143 — CGB flag

In older cartridges this byte was part of the Title (see above).
The CGB and later models interpret this byte to decide whether to enable Color mode ("CGB Mode") or to fall back to monochrome compatibility mode ("Non-CGB Mode").

Typical values are:

Value | Meaning
------|----------------------------------------------------------------------------------------------------
`$80` | The game supports CGB enhancements, but is backwards compatible with monochrome Game Boys
`$C0` | The game works on CGB only (the hardware ignores bit 6, so this really functions the same as `$80`)

Setting bit 7 will trigger a write of this register value to [KEY0 register](https://gbdev.io/pandocs/single.html#ff4c--key0sys-cgb-mode-only-cpu-mode-select) which sets the CPU mode.

### 0144–0145 — New licensee code

This area contains a two-character ASCII "licensee code" indicating the game's publisher.
It is only meaningful if the [Old licensee](https://gbdev.io/pandocs/single.html#014b--old-licensee-code) is exactly `$33` (which is the case for essentially all games made after the SGB was released); otherwise, the old code must be considered.

Sample licensee codes:

Code | Publisher
-----|-------------------------
`00` | None
`01` | [Nintendo Research & Development 1](https://en.wikipedia.org/wiki/Nintendo_Research_%26_Development_1)
`08` | [Capcom](https://en.wikipedia.org/wiki/Capcom)
`13` | [EA (Electronic Arts)](https://en.wikipedia.org/wiki/Electronic_Arts)
`18` | [Hudson Soft](https://en.wikipedia.org/wiki/Hudson_Soft)
`19` | [B-AI](https://www.giantbomb.com/b-ai/3010-8160)
`20` | [KSS](https://en.wikipedia.org/wiki/KSS_(company))
`22` | [Planning Office WADA](https://www.mobygames.com/company/15869/planning-office-wada)
`24` | [PCM Complete](https://www.mobygames.com/company/9489/pcm-complete)
`25` | [San-X](https://en.wikipedia.org/wiki/San-X)
`28` | [Kemco](https://en.wikipedia.org/wiki/Kemco)
`29` | [SETA Corporation](https://en.wikipedia.org/wiki/SETA_Corporation)
`30` | [Viacom](https://en.wikipedia.org/wiki/Viacom_(1952%E2%80%932005))
`31` | [Nintendo](https://en.wikipedia.org/wiki/Nintendo)
`32` | [Bandai](https://en.wikipedia.org/wiki/Bandai)
`33` | [Ocean Software](https://en.wikipedia.org/wiki/Ocean_Software)/[Acclaim Entertainment](https://en.wikipedia.org/wiki/Acclaim_Entertainment)
`34` | [Konami](https://en.wikipedia.org/wiki/Konami)
`35` | [HectorSoft](https://www.mobygames.com/company/12239/hectorsoft)
`37` | [Taito](https://en.wikipedia.org/wiki/Taito)
`38` | [Hudson Soft](https://en.wikipedia.org/wiki/Hudson_Soft)
`39` | [Banpresto](https://en.wikipedia.org/wiki/Banpresto)
`41` | [Ubi Soft](https://en.wikipedia.org/wiki/Ubisoft)[^The_Cartridge_Header_ubisoft]
`42` | [Atlus](https://en.wikipedia.org/wiki/Atlus)
`44` | [Malibu Interactive](https://en.wikipedia.org/wiki/Malibu_Comics)
`46` | [Angel](https://www.mobygames.com/company/5083/angel)
`47` | [Bullet-Proof Software](https://en.wikipedia.org/wiki/Blue_Planet_Software)[^The_Cartridge_Header_blueplanet]
`49` | [Irem](https://en.wikipedia.org/wiki/Irem)
`50` | [Absolute](https://en.wikipedia.org/wiki/Absolute_Entertainment)
`51` | [Acclaim Entertainment](https://en.wikipedia.org/wiki/Acclaim_Entertainment)
`52` | [Activision](https://en.wikipedia.org/wiki/Activision)
`53` | [Sammy USA Corporation](https://en.wikipedia.org/wiki/Sammy_Corporation)
`54` | [Konami](https://en.wikipedia.org/wiki/Konami)
`55` | [Hi Tech Expressions](https://tvtropes.org/pmwiki/pmwiki.php/Creator/HiTechExpressions)
`56` | [LJN](https://en.wikipedia.org/wiki/LJN)
`57` | [Matchbox](https://en.wikipedia.org/wiki/Matchbox_(brand))
`58` | [Mattel](https://en.wikipedia.org/wiki/Mattel)
`59` | [Milton Bradley Company](https://en.wikipedia.org/wiki/Milton_Bradley_Company)
`60` | [Titus Interactive](https://en.wikipedia.org/wiki/Titus_Interactive)
`61` | [Virgin Games Ltd.](https://en.wikipedia.org/wiki/Virgin_Interactive_Entertainment)[^The_Cartridge_Header_virgin]
`64` | [Lucasfilm Games](https://en.wikipedia.org/wiki/Lucasfilm_Games)[^The_Cartridge_Header_lucasfilm]
`67` | [Ocean Software](https://en.wikipedia.org/wiki/Ocean_Software)
`69` | [EA (Electronic Arts)](https://en.wikipedia.org/wiki/Electronic_Arts)
`70` | [Infogrames](https://en.wikipedia.org/wiki/Atari_SA)[^The_Cartridge_Header_atari]
`71` | [Interplay Entertainment](https://en.wikipedia.org/wiki/Interplay_Entertainment)
`72` | [Broderbund](https://en.wikipedia.org/wiki/Broderbund)
`73` | [Sculptured Software](https://en.wikipedia.org/wiki/Iguana_Entertainment)[^The_Cartridge_Header_sculptured]
`75` | [The Sales Curve Limited](https://en.wikipedia.org/wiki/SCi_Games)[^The_Cartridge_Header_sci]
`78` | [THQ](https://en.wikipedia.org/wiki/THQ)
`79` | [Accolade](https://en.wikipedia.org/wiki/Accolade,_Inc.)[^The_Cartridge_Header_infogrames]
`80` | [Misawa Entertainment](https://www.mobygames.com/company/8225/misawa-entertainment-coltd)
`83` | [LOZC G.](https://en.wikipedia.org/wiki/Category:LOZC_G._Amusements_games)
`86` | [Tokuma Shoten](https://en.wikipedia.org/wiki/Tokuma_Shoten)
`87` | Tsukuda Original
`91` | [Chunsoft Co.](https://en.wikipedia.org/wiki/Spike_Chunsoft)[^The_Cartridge_Header_spike]
`92` | [Video System](https://en.wikipedia.org/wiki/Category:Video_System_games)
`93` | [Ocean Software](https://en.wikipedia.org/wiki/Ocean_Software)/[Acclaim Entertainment](https://en.wikipedia.org/wiki/Acclaim_Entertainment)
`95` | [Varie](https://en.wikipedia.org/wiki/Varie)
`96` | [Yonezawa](https://en.wikipedia.org/wiki/Sega_Fave)[^The_Cartridge_Header_segabuy]/S'Pal
`97` | [Kaneko](https://en.wikipedia.org/wiki/Kaneko)
`99` | [Pack-In-Video](https://en.wikipedia.org/wiki/Pack-In-Video)
`9H` | Bottom Up
`A4` | [Konami](https://en.wikipedia.org/wiki/Konami) (Yu-Gi-Oh!)
`BL` | [MTO](https://en.wikipedia.org/wiki/MTO_(video_game_company))
`DK` | [Kodansha](https://en.wikipedia.org/wiki/Kodansha)

### 0146 — SGB flag

This byte specifies whether the game supports SGB functions.
The SGB will ignore any [command packets](https://gbdev.io/pandocs/single.html#command-packet-transfers) if this byte is set to a value other than `$03` (typically `$00`).

### 0147 — Cartridge type

This byte indicates what kind of hardware is present on the cartridge — most notably its [mapper](https://gbdev.io/pandocs/single.html#mbcs).

Code  | Type
------|--------------------------------
`$00` | ROM ONLY
`$01` | MBC1
`$02` | MBC1+RAM
`$03` | MBC1+RAM+BATTERY
`$05` | MBC2
`$06` | MBC2+BATTERY
`$08` | ROM+RAM [^The_Cartridge_Header_rom_ram]
`$09` | ROM+RAM+BATTERY [^The_Cartridge_Header_rom_ram]
`$0B` | MMM01
`$0C` | MMM01+RAM
`$0D` | MMM01+RAM+BATTERY
`$0F` | MBC3+TIMER+BATTERY
`$10` | MBC3+TIMER+RAM+BATTERY [^The_Cartridge_Header_mbc30]
`$11` | MBC3
`$12` | MBC3+RAM [^The_Cartridge_Header_mbc30]
`$13` | MBC3+RAM+BATTERY [^The_Cartridge_Header_mbc30]
`$19` | MBC5
`$1A` | MBC5+RAM
`$1B` | MBC5+RAM+BATTERY
`$1C` | MBC5+RUMBLE
`$1D` | MBC5+RUMBLE+RAM
`$1E` | MBC5+RUMBLE+RAM+BATTERY
`$20` | MBC6
`$22` | MBC7+SENSOR+RUMBLE+RAM+BATTERY
`$FC` | POCKET CAMERA
`$FD` | BANDAI TAMA5
`$FE` | HuC3
`$FF` | HuC1+RAM+BATTERY

[^The_Cartridge_Header_rom_ram]:
No licensed cartridge makes use of this option. The exact behavior is unknown.

[^The_Cartridge_Header_mbc30]:
MBC3 with 64 KiB of SRAM refers to MBC30, used only in _Pocket Monsters: Crystal Version_ (the Japanese version of _Pokémon Crystal Version_).

### 0148 — ROM size

This byte indicates how much ROM is present on the cartridge.
In most cases, the ROM size is given by `32 KiB × (1 << <value>)`:

Value | ROM size  | Number of ROM banks
------|-----------|----------------------
`$00` |  32 KiB   | 2 (no banking)
`$01` |  64 KiB   | 4
`$02` | 128 KiB   | 8
`$03` | 256 KiB   | 16
`$04` | 512 KiB   | 32
`$05` |   1 MiB   | 64
`$06` |   2 MiB   | 128
`$07` |   4 MiB   | 256
`$08` |   8 MiB   | 512
`$52` | 1.1 MiB   | 72 [^The_Cartridge_Header_weird_rom_sizes]
`$53` | 1.2 MiB   | 80 [^The_Cartridge_Header_weird_rom_sizes]
`$54` | 1.5 MiB   | 96 [^The_Cartridge_Header_weird_rom_sizes]

[^The_Cartridge_Header_weird_rom_sizes]:
Only listed in unofficial docs. No cartridges or ROM files using these sizes are known.
As the other ROM sizes are all powers of 2, these are likely inaccurate.
The source of these values is unknown.

### 0149 — RAM size

This byte indicates how much RAM is present on the cartridge, if any.

If the [cartridge type](https://gbdev.io/pandocs/single.html#0147--cartridge-type) does not include "RAM" in its name, this should be set to 0.
This includes MBC2, since its 512 × 4 bits of memory are built directly into the mapper.

Code  | SRAM size | Comment
------|-----------|-----------------------
`$00` |   0       | No RAM
`$01` |   –       | Unused [^The_Cartridge_Header_2kib_sram]
`$02` |   8 KiB   |  1 bank
`$03` |  32 KiB   |  4 banks of 8 KiB each
`$04` | 128 KiB   | 16 banks of 8 KiB each
`$05` |  64 KiB   |  8 banks of 8 KiB each

[^The_Cartridge_Header_2kib_sram]:
Listed in various unofficial docs as 2 KiB.
However, a 2 KiB RAM chip was never used in a cartridge.
The source of this value is unknown.

Various "PD" ROMs ("Public Domain" homebrew ROMs, generally tagged with `(PD)` in the filename) are known to use the `$01` RAM Size tag, but this is believed to have been a mistake with early homebrew tools, and the PD ROMs often don't use cartridge RAM at all.

### 014A — Destination code

This byte specifies whether this version of the game is intended to be sold in Japan or elsewhere.

Only two values are defined:

Code  | Destination
------|------------------------------
`$00` | Japan (and possibly overseas)
`$01` | Overseas only

### 014B — Old licensee code

This byte is used in older (pre-SGB) cartridges to specify the game's publisher.
However, the value `$33` indicates that the [New licensee codes](https://gbdev.io/pandocs/single.html#01440145--new-licensee-code) must be considered instead.
(The SGB will ignore any [command packets](https://gbdev.io/pandocs/single.html#command-packet-transfers) unless this value is `$33`.)

Here is a list of known Old licensee codes ([source](https://raw.githubusercontent.com/gb-archive/salvage/master/txt-files/gbrom.txt)).

HEX   | Licensee
------|------------
`00`  | None
`01`  | [Nintendo](https://en.wikipedia.org/wiki/Nintendo)
`08`  | [Capcom](https://en.wikipedia.org/wiki/Capcom)
`09`  | [HOT-B](https://en.wikipedia.org/wiki/Category:Hot_B_games)
`0A`  | [Jaleco](https://en.wikipedia.org/wiki/Jaleco)
`0B`  | [Coconuts Japan](https://en.wikipedia.org/wiki/Category:Coconuts_Japan_games)
`0C`  | [Elite Systems](https://en.wikipedia.org/wiki/Elite_Systems)
`13`  | [EA (Electronic Arts)](https://en.wikipedia.org/wiki/Electronic_Arts)
`18`  | [Hudson Soft](https://en.wikipedia.org/wiki/Hudson_Soft)
`19`  | [ITC Entertainment](https://en.wikipedia.org/wiki/ITC_Entertainment)
`1A`  | [Yanoman](https://en.wikipedia.org/wiki/Category:Yanoman_games)
`1D`  | [Japan Clary](https://www.mobygames.com/company/7639/japan-clary-business/)
`1F`  | [Virgin Games Ltd.](https://en.wikipedia.org/wiki/Virgin_Interactive_Entertainment)[^The_Cartridge_Header_virgin]
`24`  | [PCM Complete](https://www.mobygames.com/company/9489/pcm-complete)
`25`  | [San-X](https://en.wikipedia.org/wiki/San-X)
`28`  | [Kemco](https://en.wikipedia.org/wiki/Kemco)
`29`  | [SETA Corporation](https://en.wikipedia.org/wiki/SETA_Corporation)
`30`  | [Infogrames](https://en.wikipedia.org/wiki/Atari_SA)[^The_Cartridge_Header_atari]
`31`  | [Nintendo](https://en.wikipedia.org/wiki/Nintendo)
`32`  | [Bandai](https://en.wikipedia.org/wiki/Bandai)
`33`  | Indicates that the [New licensee code](https://gbdev.io/pandocs/single.html#01440145--new-licensee-code) should be used instead.
`34`  | [Konami](https://en.wikipedia.org/wiki/Konami)
`35`  | [HectorSoft](https://www.mobygames.com/company/12239/hectorsoft)
`38`  | [Capcom](https://en.wikipedia.org/wiki/Capcom)
`39`  | [Banpresto](https://en.wikipedia.org/wiki/Banpresto)
`3C`  | Entertainment Interactive (stub)
`3E`  | [Gremlin](https://en.wikipedia.org/wiki/Gremlin_Interactive)
`41`  | [Ubi Soft](https://en.wikipedia.org/wiki/Ubisoft)[^The_Cartridge_Header_ubisoft]
`42`  | [Atlus](https://en.wikipedia.org/wiki/Atlus)
`44`  | [Malibu Interactive](https://en.wikipedia.org/wiki/Malibu_Comics)
`46`  | [Angel](https://www.mobygames.com/company/5083/angel)
`47`  | [Spectrum HoloByte](https://en.wikipedia.org/wiki/Spectrum_HoloByte)
`49`  | [Irem](https://en.wikipedia.org/wiki/Irem)
`4A`  | [Virgin Games Ltd.](https://en.wikipedia.org/wiki/Virgin_Interactive_Entertainment)[^The_Cartridge_Header_virgin]
`4D`  | [Malibu Interactive](https://en.wikipedia.org/wiki/Malibu_Comics)
`4F`  | [U.S. Gold](https://en.wikipedia.org/wiki/U.S._Gold)
`50`  | [Absolute](https://en.wikipedia.org/wiki/Absolute_Entertainment)
`51`  | [Acclaim Entertainment](https://en.wikipedia.org/wiki/Acclaim_Entertainment)
`52`  | [Activision](https://en.wikipedia.org/wiki/Activision)
`53`  | [Sammy USA Corporation](https://en.wikipedia.org/wiki/Sammy_Corporation)
`54`  | [GameTek](https://en.wikipedia.org/wiki/GameTek)
`55`  | [Park Place](https://en.wikipedia.org/wiki/Park_Place_Entertainment)[^The_Cartridge_Header_caesars]
`56`  | [LJN](https://en.wikipedia.org/wiki/LJN)
`57`  | [Matchbox](https://en.wikipedia.org/wiki/Matchbox_(brand))
`59`  | [Milton Bradley Company](https://en.wikipedia.org/wiki/Milton_Bradley_Company)
`5A`  | [Mindscape](https://en.wikipedia.org/wiki/Mindscape_(company))
`5B`  | [Romstar](https://en.wikipedia.org/wiki/Romstar)
`5C`  | [Naxat Soft](https://en.wikipedia.org/wiki/Kaga_Create)[^The_Cartridge_Header_kaga]
`5D`  | [Tradewest](https://en.wikipedia.org/wiki/Tradewest)
`60`  | [Titus Interactive](https://en.wikipedia.org/wiki/Titus_Interactive)
`61`  | [Virgin Games Ltd.](https://en.wikipedia.org/wiki/Virgin_Interactive_Entertainment)[^The_Cartridge_Header_virgin]
`67`  | [Ocean Software](https://en.wikipedia.org/wiki/Ocean_Software)
`69`  | [EA (Electronic Arts)](https://en.wikipedia.org/wiki/Electronic_Arts)
`6E`  | [Elite Systems](https://en.wikipedia.org/wiki/Elite_Systems)
`6F`  | [Electro Brain](https://en.wikipedia.org/wiki/Electro_Brain)
`70`  | [Infogrames](https://en.wikipedia.org/wiki/Atari_SA)[^The_Cartridge_Header_atari]
`71`  | [Interplay Entertainment](https://en.wikipedia.org/wiki/Interplay_Entertainment)
`72`  | [Broderbund](https://en.wikipedia.org/wiki/Broderbund)
`73`  | [Sculptured Software](https://en.wikipedia.org/wiki/Iguana_Entertainment)[^The_Cartridge_Header_sculptured]
`75`  | [The Sales Curve Limited](https://en.wikipedia.org/wiki/SCi_Games)[^The_Cartridge_Header_sci]
`78`  | [THQ](https://en.wikipedia.org/wiki/THQ)
`79`  | [Accolade](https://en.wikipedia.org/wiki/Accolade,_Inc.)[^The_Cartridge_Header_infogrames]
`7A`  | [Triffix Entertainment](https://www.mobygames.com/company/4307/triffix-entertainment-inc)
`7C`  | [MicroProse](https://en.wikipedia.org/wiki/MicroProse)
`7F`  | [Kemco](https://en.wikipedia.org/wiki/Kemco)
`80`  | [Misawa Entertainment](https://www.mobygames.com/company/8225/misawa-entertainment-coltd)
`83`  | [LOZC G.](https://en.wikipedia.org/wiki/Category:LOZC_G._Amusements_games)
`86`  | [Tokuma Shoten](https://en.wikipedia.org/wiki/Tokuma_Shoten)
`8B`  | [Bullet-Proof Software](https://en.wikipedia.org/wiki/Blue_Planet_Software)[^The_Cartridge_Header_blueplanet]
`8C`  | [Vic Tokai Corp.](https://en.wikipedia.org/wiki/Tokai_Communications)[^The_Cartridge_Header_tokaicomm]
`8E`  | [Ape Inc.](https://en.wikipedia.org/wiki/Creatures_Inc.)[^The_Cartridge_Header_creatures]
`8F`  | [I'Max](https://en.wikipedia.org/wiki/I%27MAX)[^The_Cartridge_Header_imax]
`91`  | [Chunsoft Co.](https://en.wikipedia.org/wiki/Spike_Chunsoft)[^The_Cartridge_Header_spike]
`92`  | [Video System](https://en.wikipedia.org/wiki/Category:Video_System_games)
`93`  | [Tsubaraya Productions](https://en.wikipedia.org/wiki/Tsuburaya_Productions)
`95`  | [Varie](https://en.wikipedia.org/wiki/Varie)
`96`  | [Yonezawa](https://en.wikipedia.org/wiki/Sega_Fave)[^The_Cartridge_Header_segabuy]/S'Pal
`97`  | [Kemco](https://en.wikipedia.org/wiki/Kemco)
`99`  | Arc
`9A`  | [Nihon Bussan](https://en.wikipedia.org/wiki/Nihon_Bussan)
`9B`  | [Tecmo](https://en.wikipedia.org/wiki/Tecmo)
`9C`  | [Imagineer](https://en.wikipedia.org/wiki/Imagineer_(Japanese_company))
`9D`  | [Banpresto](https://en.wikipedia.org/wiki/Banpresto)
`9F`  | Nova
`A1`  | [Hori Electric](https://www.mobygames.com/company/8959/hori-electric-co-ltd/)
`A2`  | [Bandai](https://en.wikipedia.org/wiki/Bandai)
`A4`  | [Konami](https://en.wikipedia.org/wiki/Konami)
`A6`  | Kawada
`A7`  | [Takara](https://en.wikipedia.org/wiki/Takara)
`A9`  | [Technos Japan](https://en.wikipedia.org/wiki/Techn%C5%8Ds_Japan)
`AA`  | [Broderbund](https://en.wikipedia.org/wiki/Broderbund)
`AC`  | [Toei Animation](https://en.wikipedia.org/wiki/Toei_Animation)
`AD`  | [Toho](https://en.wikipedia.org/wiki/Toho)
`AF`  | [Namco](https://en.wikipedia.org/wiki/Namco)
`B0`  | [Acclaim Entertainment](https://en.wikipedia.org/wiki/Acclaim_Entertainment)
`B1`  | [ASCII Corporation](https://en.wikipedia.org/wiki/ASCII_Corporation) or Nexsoft
`B2`  | [Bandai](https://en.wikipedia.org/wiki/Bandai)
`B4`  | [Square Enix](https://en.wikipedia.org/wiki/Square_Enix)
`B6`  | [HAL Laboratory](https://en.wikipedia.org/wiki/HAL_Laboratory)
`B7`  | [SNK](https://en.wikipedia.org/wiki/SNK)
`B9`  | [Pony Canyon](https://en.wikipedia.org/wiki/Pony_Canyon)
`BA`  | [Culture Brain](https://en.wikipedia.org/wiki/Culture_Brain)
`BB`  | [Sunsoft](https://en.wikipedia.org/wiki/Sunsoft)
`BD`  | [Sony Imagesoft](https://en.wikipedia.org/wiki/Sony_Imagesoft)
`BF`  | [Sammy Corporation](https://en.wikipedia.org/wiki/Sammy_Corporation)
`C0`  | [Taito](https://en.wikipedia.org/wiki/Taito)
`C2`  | [Kemco](https://en.wikipedia.org/wiki/Kemco)
`C3`  | [Square](https://en.wikipedia.org/wiki/Square_(video_game_company))
`C4`  | [Tokuma Shoten](https://en.wikipedia.org/wiki/Tokuma_Shoten)
`C5`  | [Data East](https://en.wikipedia.org/wiki/Data_East)
`C6`  | [Tonkin House](https://en.wikipedia.org/wiki/Tonkin_House)
`C8`  | [Koei](https://en.wikipedia.org/wiki/Koei)
`C9`  | UFL
`CA`  | [Ultra Games](https://en.wikipedia.org/wiki/Ultra_Games)
`CB`  | [VAP, Inc.](https://en.wikipedia.org/wiki/VAP,_Inc.)
`CC`  | [Use Corporation](https://en.wikipedia.org/wiki/Category:Use_Corporation_games)
`CD`  | [Meldac](https://en.wikipedia.org/wiki/Meldac)
`CE`  | [Pony Canyon](https://en.wikipedia.org/wiki/Pony_Canyon)
`CF`  | [Angel](https://www.mobygames.com/company/5083/angel)
`D0`  | [Taito](https://en.wikipedia.org/wiki/Taito)
`D1`  | [SOFEL (Software Engineering Lab)](https://en.wikipedia.org/wiki/SOFEL)
`D2`  | [Quest](https://en.wikipedia.org/wiki/Quest_Corporation)
`D3`  | [Sigma Enterprises](https://www.mobygames.com/company/5001/sigma-enterprises-inc)
`D4`  | [ASK Kodansha Co.](https://www.mobygames.com/company/5166/ask-co-ltd/)
`D6`  | [Naxat Soft](https://en.wikipedia.org/wiki/Kaga_Create)[^The_Cartridge_Header_kaga]
`D7`  | [Copya System](https://en.wikipedia.org/wiki/Category:Copya_Systems_games)
`D9`  | [Banpresto](https://en.wikipedia.org/wiki/Banpresto)
`DA`  | [Tomy](https://en.wikipedia.org/wiki/Tomy)
`DB`  | [LJN](https://en.wikipedia.org/wiki/LJN)
`DD`  | [Nippon Computer Systems](https://www.ncsx.co.jp/)
`DE`  | [Human Ent.](https://en.wikipedia.org/wiki/Human_Entertainment)
`DF`  | [Altron](https://en.wikipedia.org/wiki/Category:Altron_games)
`E0`  | [Jaleco](https://en.wikipedia.org/wiki/Jaleco)
`E1`  | [Towa Chiki](https://en.wikipedia.org/wiki/Towa_Chiki)
`E2`  | [Yutaka](https://en.wikipedia.org/wiki/Yutaka_(video_game_company)) # Needs more info
`E3`  | [Varie](https://en.wikipedia.org/wiki/Varie)
`E5`  | [Epoch](https://en.wikipedia.org/wiki/Epoch_Co.)
`E7`  | [Athena](https://en.wikipedia.org/wiki/Athena_(game_developer))
`E8`  | [Asmik Ace Entertainment](https://en.wikipedia.org/wiki/Asmik_Ace)
`E9`  | [Natsume](https://en.wikipedia.org/wiki/Natsume_Inc.)
`EA`  | [King Records](https://en.wikipedia.org/wiki/King_Records_(Japan))
`EB`  | [Atlus](https://en.wikipedia.org/wiki/Atlus)
`EC`  | Epic/[Sony Records](https://en.wikipedia.org/wiki/Sony_Music)
`EE`  | [IGS](https://web.archive.org/web/20240825224157/https://igs-entertainment.co/)
`F0`  | [A Wave](https://www.mobygames.com/company/9123/a-wave-inc/)
`F3`  | [Extreme Entertainment](https://www.mobygames.com/company/4221/extreme-entertainment-group-inc)
`FF`  | [LJN](https://en.wikipedia.org/wiki/LJN)

[^The_Cartridge_Header_ubisoft]: Later known as [Ubisoft](https://en.wikipedia.org/wiki/Ubisoft).

[^The_Cartridge_Header_blueplanet]: Later succeeded by [Blue Planet Software](https://en.wikipedia.org/wiki/Blue_Planet_Software), then acquired by [The Tetris Company](https://en.wikipedia.org/wiki/The_Tetris_Company) in 2020.

[^The_Cartridge_Header_virgin]: Later known as [Virgin Mastertronic Ltd., then Virgin Interactive Entertainment, then Avalon Interactive Group, Ltd.](https://en.wikipedia.org/wiki/Virgin_Interactive_Entertainment).

[^The_Cartridge_Header_lucasfilm]: Later known as [LucasArts](https://en.wikipedia.org/wiki/Lucasfilm_Games) between 1990-2021.

[^The_Cartridge_Header_atari]: Later known as [Atari SA](https://en.wikipedia.org/wiki/Atari_SA).

[^The_Cartridge_Header_sculptured]: Later accquired by [Iguana Entertainment](https://en.wikipedia.org/wiki/Iguana_Entertainment) in 1995. Parent studio owned by [Acclaim Entertainment](https://en.wikipedia.org/wiki/Acclaim_Entertainment).

[^The_Cartridge_Header_sci]: Later known as [SCi (Sales Curve Interactive), then SCi Entertainment Group plc, then Eidos](https://en.wikipedia.org/wiki/SCi_Games), then acquired by [Square Enix](https://en.wikipedia.org/wiki/Square_Enix) in 2009.

[^The_Cartridge_Header_spike]: Later known as [Spike Chunsoft Co., Ltd.](https://en.wikipedia.org/wiki/Spike_Chunsoft).

[^The_Cartridge_Header_kaga]: Later known as [Kaga Create](https://en.wikipedia.org/wiki/Kaga_Create).

[^The_Cartridge_Header_tokaicomm]: Known as Vic Tokai Corporation until 2011 when the name changed to Tokai Communications.

[^The_Cartridge_Header_infogrames]: Later Infogrames North America, Inc.

[^The_Cartridge_Header_caesars]: Later named Caesars Entertainment, Inc.

[^The_Cartridge_Header_creatures]: Now known as Creatures, Inc.

[^The_Cartridge_Header_imax]: Not to be confused with the IMAX motion picture film format.

[^The_Cartridge_Header_segabuy]: Merged into Sega as Sega-Yonezawa, later becoming Sega Toys, and finally Sega Fave.

### 014C — Mask ROM version number

This byte specifies the version number of the game.
It is usually `$00`.

### 014D — Header checksum

This byte contains an 8-bit checksum computed from the cartridge header bytes $0134–014C.
The boot ROM computes the checksum as follows:

```c
uint8_t checksum = 0;
for (uint16_t address = 0x0134; address <= 0x014C; address++) {
    checksum = checksum - rom[address] - 1;
}
```

The boot ROM verifies this checksum.
If the byte at `$014D` does not match the lower 8 bits of `checksum`, the boot ROM will lock up and the program in the
cartridge **won't run**.

### 014E-014F — Global checksum

These bytes contain a 16-bit (big-endian) checksum simply computed as the sum of
all the bytes of the cartridge ROM (except these two checksum bytes).

This checksum is not verified, except by Pokémon Stadium's "GB Tower" emulator (presumably to detect Transfer Pak errors).

_Fonte desta seção: [`src/The_Cartridge_Header.md`](https://github.com/gbdev/pandocs/blob/fe246067b695b5404a4a6a47efb4fd6d921ececb/src/The_Cartridge_Header.md)_


---

<!-- fonte: src/MBCs.md @ fe246067b695 -->

## MBCs

As the Game Boy's 16-bit address bus offers only limited space for
ROM and RAM addressing, many games are using Memory Bank Controllers
(MBCs) to expand the available address space by bank switching.
These MBC chips are located in the game cartridge (that is, not in
the Game Boy itself).

In each cartridge, the required (or preferred) MBC type should be
specified in the byte at $0147 of the ROM, as described
[in the cartridge header](https://gbdev.io/pandocs/single.html#0147--cartridge-type).  Several MBC
types are available:

### MBC Timing Issues

Among Nintendo MBCs, only the MBC5 is guaranteed by Nintendo to support
the tighter timing of CGB Double Speed Mode. There have been rumours
that older MBCs (like MBC1-3) wouldn't be fast enough in that mode. If
so, it might be nevertheless possible to use Double Speed during periods
which use only code and data which is located in internal RAM. Despite the 
above, a self-made MBC1-EPROM card appears to work stable and fine even in 
Double Speed Mode.

### MBC Unmapped RAM Bank Access

In most MBCs, if an unmapped RAM bank is selected (which would be translate to an out of bounds RAM address by the MBC controller), 
the MBC will simply wrap around the internal ram address and would access a valid RAM address.

The MBC internal address being accessed can be calculated using this formula: `((address - external_ram_start_address) + (active_ram_bank * ram_bank_size)) % max_external_ram_size`.

_Fonte desta seção: [`src/MBCs.md`](https://github.com/gbdev/pandocs/blob/fe246067b695b5404a4a6a47efb4fd6d921ececb/src/MBCs.md)_


---

<!-- fonte: src/nombc.md @ fe246067b695 -->

## No MBC

(32 KiB ROM only)

Small games of not more than 32 KiB ROM do not require a MBC chip for
ROM banking. The ROM is directly mapped to memory at $0000-7FFF.
Optionally up to 8 KiB of RAM could be connected at $A000-BFFF, using
a discrete logic decoder <!--74HC138?--> in place of a full MBC chip.

_Fonte desta seção: [`src/nombc.md`](https://github.com/gbdev/pandocs/blob/fe246067b695b5404a4a6a47efb4fd6d921ececb/src/nombc.md)_


---

<!-- fonte: src/MBC1.md @ fe246067b695 -->

## MBC1

(max 2MByte ROM and/or 32 KiB RAM)

This is the first MBC chip for the Game Boy. Any newer MBC chips
work similarly, so it is relatively easy to upgrade a program from one
MBC chip to another — or to make it compatible with several
types of MBCs.

In its default configuration, MBC1 supports up to 512 KiB ROM with up to 32 KiB of banked RAM.
Some cartridges wire the MBC differently, where the 2-bit RAM banking register is wired as an extension of the ROM banking register (instead of to RAM) in order to support up to 2 MiB ROM, at the cost of only supporting a fixed 8 KiB of cartridge RAM.
All MBC1 cartridges with 1 MiB of ROM or more use this alternate wiring.
Also see the note on MBC1M multi-game compilation carts below.

Note that the memory in range 0000–7FFF is used both for reading from
ROM and writing to the MBCs Control Registers.

### Memory

#### 0000–3FFF — ROM Bank X0 \[read-only\]

This area normally contains the first 16 KiB (bank 00) of the cartridge
ROM.

In 1 MiB and larger cartridges (that use the 2-bit second banking register for extended ROM banking), entering mode 1 (see below) will allow that second banking register to apply to reads from this region in addition to the regular 4000–7FFF banked region, resulting in accessing banks $20/$40/$60 for regular large ROM carts, or banks $10/$20/$30 for an MBC1M multi-cart (see note below).

#### 4000–7FFF — ROM Bank 01-7F \[read-only\]

This area may contain any of the further 16 KiB banks of the ROM. If the main 5-bit ROM banking register is 0, it reads the bank as if it was set to 1.

For 1 MiB+ ROM, this means any bank that is possible to access via the 0000–3FFF region is not accessible in this region. i.e. banks $00/$20/$40/$60 in regular large ROM carts, or banks $00/$10/$20/$30 in MBC1M multi-game compilation carts. Instead, it automatically maps to 1 bank
higher ($01/$21/$41/$61 or $01/$11/$21/$31 respectively).

#### A000–BFFF — RAM Bank 00–03, if any

This area is used to address external RAM in the cartridge (if any). The RAM is only accessible if RAM is enabled, otherwise reads return open bus values (often $FF, but not guaranteed) and writes are ignored.

Available RAM sizes are 8 KiB (at $A000–BFFF) and 32 KiB (in form of four 8K banks at $A000–BFFF). 32 KiB is only available in cartridges with ROM <= 512 KiB.

External RAM is often battery-backed, allowing for the storage of game data while the Game Boy is turned off, or if the cartridge is removed from the Game Boy. External RAM is no slower than the Game Boy's internal RAM, so many games use part of the external RAM as extra working RAM, even if they use another part of it for battery-backed saves.

### Registers

All of the MBC1 registers default to $00 on power-up, which for the "ROM Bank Number" register is _treated as_ $01.

#### 0000–1FFF — RAM Enable (Write Only)

Before external RAM can be read or written, it must be enabled by
writing `$A` to anywhere in this address space.
Any value with `$A` in the lower 4 bits **enables** the RAM attached to the MBC, and any
other value **disables** the RAM. It is unknown why `$A` is the value used to enable RAM.

It is recommended to disable external RAM
after accessing it, in order to protect its contents from corruption during
power down of the Game Boy or removal of the cartridge. Once the cartridge has _completely_ lost power from the Game Boy, the RAM is automatically disabled to protect it.

#### 2000–3FFF — ROM Bank Number (Write Only)

This 5-bit register (range $01-$1F) selects the ROM bank number for the 4000–7FFF region. Higher
bits are discarded — writing $E1 (binary 111**00001**) to this register
would select bank $01.

If this register is set to $00, it behaves as if it is set to $01. This means you cannot duplicate bank $00 into both the 0000–3FFF and 4000–7FFF ranges by setting this register to $00.

If the ROM Bank Number is set to a higher value than the number of banks
in the cart, the bank number is masked to the required number of bits.
e.g. a 256 KiB cart only needs a 4-bit bank number to address all of its
16 banks, so this register is masked to 4 bits. The upper bit would be
ignored for bank selection.

Even with smaller ROMs that use less than 5 bits for bank selection, the full 5-bit register is still compared for the bank 00→01 translation logic. As a result if the ROM is 256 KiB or smaller, it _is_ possible to map bank 0 to the 4000–7FFF region — by setting the 5th bit to 1 it will prevent the 00→01 translation (which looks at the full 5-bit register, and sees the value $10, not $00), while the bits actually used for bank selection (4, in this example) are all 0, so bank $00 is selected.

On larger carts which need a >5 bit bank number, the secondary banking
register at 4000–5FFF is used to supply an additional 2 bits for the
effective bank number:
`Selected ROM Bank = (Secondary Bank << 5) + ROM Bank`.[^MBC1_MBC1M_banking]

These additional two bits are ignored for the bank 00→01 translation. This causes a problem — attempting to access banks $20, $40, and $60 only set bits in the upper 2-bit register, with the lower 5-bit register set to 00. As a result, any
attempt to address these ROM Banks will select Bank $21, $41 and $61
instead. The only way to access banks $20, $40 or $60 at all is to enter mode 1,
which remaps the 0000–3FFF range. This has its own problems for game
developers as that range contains interrupt handlers, so it's usually only
used in multi-game compilation carts (see below).

[^MBC1_MBC1M_banking]: MBC1M has a different formula, see below.

#### 4000–5FFF — RAM Bank Number — or — Upper Bits of ROM Bank Number (Write Only)

This second 2-bit register can be used to select a RAM Bank in range from
$00–$03 (32 KiB ram carts only), or to specify the upper two bits (bits 5-6)
of the ROM Bank number (1 MiB ROM or larger carts only). If neither ROM nor
RAM is large enough, setting this register does nothing.

In 1MB MBC1 multi-carts (see below), this 2-bit register is instead
applied to bits 4-5 of the ROM bank number and the top bit of the main
5-bit main ROM banking register is ignored.

#### 6000–7FFF — Banking Mode Select (Write Only)

This 1-bit register selects between the two MBC1 banking modes, controlling
the behaviour of the secondary 2-bit banking register (above). If the cart
is not large enough to use the 2-bit register (≤ 8 KiB RAM and ≤ 512 KiB ROM)
this mode select has no observable effect. The program may freely switch
between the two modes at any time.

**`Value written`** — layout de bits (8 bits):

| Bits | Campo |
|---|---|
| 0 | Banking mode |


The **banking mode** can be:
- `0` = *simple* (default): 0000–3FFF and A000–BFFF are locked to bank 0 of ROM and SRAM respectively.
- `1` = *advanced*: 0000–3FFF and A000-BFFF can be bank-switched via the [4000–5FFF register](https://gbdev.io/pandocs/single.html#40005fff--ram-bank-number--or--upper-bits-of-rom-bank-number-write-only).


:::tip

Technically, the MBC1 has AND gates between the both bank registers and the second-highest bit of the address. This is intended to cause accesses to the 0000–3FFF region (which has that address bit set to 0) to treat both registers as always 0, so that only bank 0 is accessible through this address.

However, when the second bank register is connected to RAM, this has the side effect of also locking RAM to bank 0, as the RAM address space (A000–BFFF) _also_ has the second-highest address bit set to 0.

Setting the mode to 1 disables these AND gates, allowing the two-bit register to switch the selected bank in both these regions.

:::

### Addressing diagrams

The following diagrams show how the address within the ROM/RAM chips are calculated from the accessed address and banking registers

#### 0000–3FFF

<div class=table-wrapper><table class=bit-descrs><thead><tr>
  <th></th><th>20</th><th>19</th><th>18</th><th>17</th><th>16</th><th>15</th><th>14</th><th>13</th><th>12</th><th>...</th><th>1</th><th>0</th>
</tr></thead><tbody><tr>
  <td><strong>Mode 0</strong></td><td colspan=2>0</td><td rowspan=2 colspan=5>0</td><td rowspan=2 colspan=5>From Game Boy address</td>
</tr><tr>
  <td><strong>Mode 1</strong></td><td colspan=2>From 4000–5FFF bank register</td>
</tr></tbody></table></div>

#### 4000–7FFF

<div class=table-wrapper><table class=bit-descrs><thead><tr>
  <th></th><th>20</th><th>19</th><th>18</th><th>17</th><th>16</th><th>15</th><th>14</th><th>13</th><th>12</th><th>...</th><th>1</th><th>0</th>
</tr></thead><tbody><tr>
  <td><strong>Mode 0 / Mode 1</strong></td><td colspan=2>From 4000–5FFF bank register</td><td colspan=5>From 2000–3FFF bank register</td><td colspan=5>From Game Boy address</td>
</tr></tbody></table></div>

:::tip

In a smaller cartridge, some of the upper bits are ignored.
(For example, a 128 KiB = 2<sup>17</sup> bytes ROM only uses bits 0–16.)

:::

#### A000–BFFF

<div class=table-wrapper><table class=bit-descrs><thead><tr>
  <th></th><th>14</th><th>13</th><th>12</th><th>...</th><th>1</th><th>0</th>
</tr></thead><tbody><tr>
  <td><strong>Mode 0</strong></td><td colspan=2>0</td><td rowspan=2 colspan=4>From Game Boy address</td>
</tr><tr>
  <td><strong>Mode 1</strong></td><td colspan=2>From 4000–5FFF bank register</td>
</tr></tbody></table></div>

### "MBC1M": 1 MiB Multi-Game Compilation Carts

Known as MBC1M, these carts have an alternative wiring, that ignores
the top bit of the main ROM banking register (making it effectively a 4-bit register for banking, though the full 5 bit register is still used for 00→01 translation)
and applies the 2-bit register to bits 4-5 of the bank number (instead of
the usual bits 5-6). This means that in mode 1 the 2-bit register selects
banks $00, $10, $20, or $30, rather than the usual $00, $20, $40 or $60.

These carts make use of the fact that mode 1 remaps the 0000–3FFF area
to switch games. The 2-bit register is used to select the game — switching
the zero bank and the region of banks that the 4000–7FFF ROM area can
access to those for the selected game and then the game only changes the
main ROM banking register. As far as the selected game knows, it's running
from a 256 KiB cart!

These carts can normally be identified by having a Nintendo copyright
header in bank $10. A badly dumped multi-cart ROM can be identified by
having duplicate content in banks $10-$1F (dupe of $00–$0F) and banks $30-$3F
(dupe of $20-$2F).
There is a known bad dump of the Mortal Kombat I & II collection around.

An "MBC1M" compilation cart ROM can be converted into a regular MBC1 ROM
by increasing the ROM size to 2 MiB and duplicating each sub-ROM — $00–$0F
duplicated into $10-$1F, the original $10-$1F placed in $20-$2F and
duplicated into $30-$3F and so on.

#### MBC1M addressing diagrams

##### 0000–3FFF

<div class=table-wrapper><table class=bit-descrs><thead><tr>
  <th></th><th>19</th><th>18</th><th>17</th><th>16</th><th>15</th><th>14</th><th>13</th><th>12</th><th>..</th><th>1</th><th>0</th>
</tr></thead><tbody><tr>
  <td><strong>Mode 0</strong></td><td colspan=2>0</td><td rowspan=2 colspan=4>0</td><td rowspan=2 colspan=5>From Game Boy address</td>
</tr><tr>
  <td><strong>Mode 1</strong></td><td colspan=2>From 4000–5FFF bank register</td>
</tr></tbody></table></div>

##### 4000–7FFF

<div class=table-wrapper><table class=bit-descrs><thead><tr>
  <th></th><th>19</th><th>18</th><th>17</th><th>16</th><th>15</th><th>14</th><th>13</th><th>12</th><th>..</th><th>1</th><th>0</th>
</tr></thead><tbody><tr>
  <td><strong>Mode 0 / Mode 1</strong></td><td colspan=2>From 4000–5FFF bank register</td><td colspan=4>From 2000–3FFF bank register (bit 4 unused)</td><td colspan=5>From Game Boy address</td>
</tr></tbody></table></div>

_Fonte desta seção: [`src/MBC1.md`](https://github.com/gbdev/pandocs/blob/fe246067b695b5404a4a6a47efb4fd6d921ececb/src/MBC1.md)_


---

<!-- fonte: src/MBC2.md @ fe246067b695 -->

## MBC2

(max 256 KiB ROM and 512×4 bits RAM)

### Memory

#### 0000–3FFF — ROM Bank 0 \[read-only\]

Contains the first 16 KiB of the ROM.

#### 4000–7FFF — ROM Bank $01-0F \[read-only\]

Same as for MBC1, but only a total of 16 ROM banks is supported.

#### A000–A1FF — Built-in RAM

The MBC2 doesn't support external RAM, instead it includes 512 half-bytes of RAM (built into the MBC2 chip itself).
It still requires an external battery to save data during power-off though.
As the data consists of 4-bit values, only the lower 4 bits of the bytes in this memory area are used.
The upper 4 bits of each byte are undefined and should not be relied upon.

#### A200–BFFF — 15 "echoes" of A000–A1FF

Only the bottom 9 bits of the address are used to index into the internal RAM, so RAM access repeats.
As with the A000–A1FF region, only the lower 4 bits of the "bytes" are used, and the upper 4 bits of each byte are undefined and should not be relied upon.

### Registers

#### 0000–3FFF — RAM Enable, ROM Bank Number \[write-only\]

This address range is responsible for both enabling/disabling the RAM and for controlling the ROM bank number.
Bit 8 of the address (the least
significant bit of the upper address byte) determines whether to control
the RAM enable flag or the ROM bank number.

##### When bit 8 is clear

When the least significant bit of the upper address byte is zero, the value that is written controls whether the RAM is enabled.
Save RAM will be enabled if and only if the lower 4 bits of the value written here are `$A`.
If any other value is written, RAM is disabled.

Examples of addresses that can control RAM: $0000–00FF, $0200–02FF, $0400–04FF, ..., $3E00–3EFF.

RAM is disabled by default.

##### When bit 8 is set

When the least significant bit of the upper address byte is one, the value that is written controls the selected ROM bank at 4000–7FFF.

Specifically, the lower 4 bits of the value written to this address range specify the ROM bank number.
If bank 0 is written, the resulting bank will be bank 1 instead.

Examples of address that can control ROM: $0100–01FF, $0300–03FF, $0500–05FF, ..., $3F00–3FFF.

The ROM bank is set to 1 by default.

_Fonte desta seção: [`src/MBC2.md`](https://github.com/gbdev/pandocs/blob/fe246067b695b5404a4a6a47efb4fd6d921ececb/src/MBC2.md)_


---

<!-- fonte: src/MBC3.md @ fe246067b695 -->

## MBC3

(max 2MByte ROM and/or 32KByte RAM and Timer)

Beside for the ability to access up to 2MB ROM (128 banks), and 32KB RAM
(4 banks), the MBC3 also includes a built-in Real Time Clock (RTC). The
RTC requires an external 32.768 kHz Quartz Oscillator, and an external
battery (if it should continue to tick when the Game Boy is turned off).

### Memory

#### 0000-3FFF - ROM Bank 00 (Read Only)

Contains the first 16 KiB of the ROM.

#### 4000-7FFF - ROM Bank 01-7F (Read Only)

Same as for MBC1, except that accessing banks $20, $40, and $60 is
supported now.

#### A000-BFFF - RAM Bank 00-07 or RTC register (Read/Write)

Depending on the current Bank Number/RTC Register selection (see below),
this memory space is used to access an 8 KiB external RAM Bank, or a
single RTC Register.

### Registers

#### A000-BFFF - RTC Register 08-0C (Read/Write)

Depending on the current Bank Number/RTC Register selection (see below),
this memory space is used to access an 8KByte external RAM Bank, or a
single RTC Register. The mapped RTC register can be read/written by 
accessing any address in that area, typically using address A000.

#### 0000-1FFF - RAM and Timer Enable (Write Only)

Mostly the same as for MBC1, a value of $0A will enable reading and
writing to external RAM - and to the RTC Registers! A value of $00 will
disable either.

#### 2000-3FFF - ROM Bank Number (Write Only)

Same as for MBC1, except that the whole 7 bits of the ROM Bank Number
are written directly to this address. As with the MBC1, writing a value
of $00 will select Bank $01 instead. All other values $01-$7F select the
corresponding ROM Banks.

#### 4000-5FFF - RAM Bank Number - or - RTC Register Select (Write Only)

Controls what is mapped into memory at A000-BFFF.

| Value   | Selection                                     |
|---------|-----------------------------------------------|
| $00-$07 | The corresponding RAM Bank.                   |
| $08-$0C | The corresponding RTC Register (see below).    |


#### 6000-7FFF - Latch Clock Data (Write Only)

When writing $00, and then $01 to this register, the current time
becomes latched into the RTC registers. The latched data will not change
until it becomes latched again, by repeating the write $00-\>$01
procedure. This provides a way to read the RTC registers while the
clock keeps ticking.

#### Clock Counter Registers
| Register | Name | Description | Range |
|----------|------|-------------|-------|
| $08 | RTC S | Seconds | 0-59 ($00-$3B) |
| $09 | RTC M | Minutes | 0-59 ($00-$3B) |
| $0A | RTC H | Hours | 0-23 ($00-$17) |
| $0B | RTC DL | Lower 8 bits of Day Counter | ($00-$FF) |
| $0C | RTC DH | Upper 1 bit of Day Counter, Carry Bit, Halt Flag. <br>Bit 0: Most significant bit (Bit 8) of Day Counter<br>Bit 6: Halt (0=Active, 1=Stop Timer)<br>Bit 7:  Day Counter Carry Bit (1=Counter Overflow) | |

The Halt Flag is supposed to be set before **writing** to the RTC
Registers.

#### The Day Counter

The total 9 bits of the Day Counter allow counting days in range from
0-511 ($000-$1FF). The Day Counter Carry Bit becomes set when this value
overflows. In that case the Carry Bit remains set until the program does
reset it. Note that you can store an offset to the Day Counter in
battery RAM. For example, every time you read a non-zero Day Counter,
add this Counter to the offset in RAM, and reset the Counter to zero.
This method allows counting any number of days, making your program
Year-10000-Proof, provided that the cartridge gets used at least every
511 days.

#### Delays

When accessing the RTC Registers, it is recommended to wait 4 µs
(4 M-cycles in Normal Speed Mode) between any separate accesses.


## MBC30

(4 MiB ROM, 64 KiB RAM, timer)

The MBC30 is practically identical to MBC3 in operation, but is capable of addressing twice as much memory for both ROM and RAM.
The only title to be shipped with the MBC30 mapper was _Pocket Monsters: Crystal Version_ in Japan, with the various worldwide versions using MBC3.

_Fonte desta seção: [`src/MBC3.md`](https://github.com/gbdev/pandocs/blob/fe246067b695b5404a4a6a47efb4fd6d921ececb/src/MBC3.md)_


---

<!-- fonte: src/MBC5.md @ fe246067b695 -->

## MBC5

It can map up to 64 Mbits (8 MiB) of ROM.

MBC5 (Memory Bank Controller 5) is the 5th generation MBC (MBC4 was not used in any released cartridges).
It is the first MBC that is guaranteed to work properly with GBC Double Speed mode.

### Memory

#### 0000-3FFF - ROM Bank 00 (Read Only)

Contains the first 16 KiB of the ROM.

#### 4000-7FFF - ROM bank 00-1FF (Read Only)

Same as for MBC1, except that accessing up to bank $1FF is supported
now. Also, bank 0 is actually bank 0.

#### A000-BFFF - RAM bank 00-0F, if any (Read/Write)

Same as for MBC1, except that RAM sizes are 8 KiB, 32 KiB and 128 KiB.

### Registers

#### 0000-1FFF - RAM Enable (Write Only)

Mostly the same as for MBC1. Writing $0A will enable reading and
writing to external RAM. Writing $00 will disable it.

Actual MBCs actually enable RAM when writing any value whose bottom 4 bits equal $A (so $0A, $1A, and so on), and disable it when writing anything else.
Relying on this behavior is not recommended for compatibility reasons.

#### 2000-2FFF - 8 least significant bits of ROM bank number (Write Only)

The 8 least significant bits of the ROM bank number go here. Writing 0 will indeed
give bank 0 on MBC5, unlike other MBCs.

#### 3000-3FFF - 9th bit of ROM bank number (Write Only)

The 9th bit of the ROM bank number goes here.

#### 4000-5FFF - RAM bank number (Write Only)

As for the MBC1s RAM Banking Mode, writing a value in the range $00-$0F
maps the corresponding external RAM bank (if any) into the memory area at
A000-BFFF.

> _(imagem omitida nesta cópia offline)_

#### Rumble

On cartridges which feature a rumble motor, bit 3 of the RAM Bank register
is connected to the Rumble circuitry instead of the RAM chip. Setting the
bit to 1 enables the rumble motor and keeps it enabled until the bit is reset again.

To control the rumble's intensity, it should be turned on and off repeatedly,
as seen with these two examples from Pokémon Pinball:

> _Diagrama `../generated/MBC5_Rumble_Mild.svg` omitido nesta cópia offline (era um SVG do Pan Docs). Ver https://gbdev.io/pandocs/single.html_

> _Diagrama `../generated/MBC5_Rumble_Strong.svg` omitido nesta cópia offline (era um SVG do Pan Docs). Ver https://gbdev.io/pandocs/single.html_

_Fonte desta seção: [`src/MBC5.md`](https://github.com/gbdev/pandocs/blob/fe246067b695b5404a4a6a47efb4fd6d921ececb/src/MBC5.md)_


---
