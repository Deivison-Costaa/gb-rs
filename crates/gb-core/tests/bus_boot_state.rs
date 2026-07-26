//! ROADMAP 1.2b-ii — os registradores de hardware no hand-off da boot ROM.

use gb_core::bus::Bus;
use gb_core::cart::{Cartridge, OPEN_BUS};

const PROBE: u8 = 0x5A;

struct SilentCartridge;

impl Cartridge for SilentCartridge {
    fn read(&self, _addr: u16) -> u8 {
        OPEN_BUS
    }

    fn write(&mut self, _addr: u16, _value: u8) {}
}

fn bus() -> Bus {
    Bus::new(Box::new(SilentCartridge))
}

type Cell = Option<u8>;

const DMG_COLUMN: &[(u16, &str, Cell)] = &[
    (0xFF00, "P1", Some(0xCF)),
    (0xFF01, "SB", Some(0x00)),
    (0xFF02, "SC", Some(0x7E)),
    (0xFF04, "DIV", Some(0xAB)),
    (0xFF05, "TIMA", Some(0x00)),
    (0xFF06, "TMA", Some(0x00)),
    (0xFF07, "TAC", Some(0xF8)),
    (0xFF0F, "IF", Some(0xE1)),
    (0xFF10, "NR10", Some(0x80)),
    (0xFF11, "NR11", Some(0xBF)),
    (0xFF12, "NR12", Some(0xF3)),
    (0xFF13, "NR13", Some(0xFF)),
    (0xFF14, "NR14", Some(0xBF)),
    (0xFF16, "NR21", Some(0x3F)),
    (0xFF17, "NR22", Some(0x00)),
    (0xFF18, "NR23", Some(0xFF)),
    (0xFF19, "NR24", Some(0xBF)),
    (0xFF1A, "NR30", Some(0x7F)),
    (0xFF1B, "NR31", Some(0xFF)),
    (0xFF1C, "NR32", Some(0x9F)),
    (0xFF1D, "NR33", Some(0xFF)),
    (0xFF1E, "NR34", Some(0xBF)),
    (0xFF20, "NR41", Some(0xFF)),
    (0xFF21, "NR42", Some(0x00)),
    (0xFF22, "NR43", Some(0x00)),
    (0xFF23, "NR44", Some(0xBF)),
    (0xFF24, "NR50", Some(0x77)),
    (0xFF25, "NR51", Some(0xF3)),
    (0xFF26, "NR52", Some(0xF1)),
    (0xFF40, "LCDC", Some(0x91)),
    (0xFF41, "STAT", Some(0x85)),
    (0xFF42, "SCY", Some(0x00)),
    (0xFF43, "SCX", Some(0x00)),
    (0xFF44, "LY", Some(0x00)),
    (0xFF45, "LYC", Some(0x00)),
    (0xFF46, "DMA", Some(0xFF)),
    (0xFF47, "BGP", Some(0xFC)),
    (0xFF48, "OBP0", None),
    (0xFF49, "OBP1", None),
    (0xFF4A, "WY", Some(0x00)),
    (0xFF4B, "WX", Some(0x00)),
    (0xFFFF, "IE", Some(0x00)),
];

const CGB_ONLY: &[(u16, &str)] = &[
    (0xFF4C, "KEY0"),
    (0xFF4D, "KEY1"),
    (0xFF4F, "VBK"),
    (0xFF50, "BANK"),
    (0xFF51, "HDMA1"),
    (0xFF52, "HDMA2"),
    (0xFF53, "HDMA3"),
    (0xFF54, "HDMA4"),
    (0xFF55, "HDMA5"),
    (0xFF56, "RP"),
    (0xFF68, "BGPI"),
    (0xFF69, "BGPD"),
    (0xFF6A, "OGPI"),
    (0xFF6B, "OGPD"),
    (0xFF70, "SVBK"),
];

fn unnamed_io_addresses() -> Vec<u16> {
    let named: Vec<u16> = DMG_COLUMN
        .iter()
        .map(|&(addr, _, _)| addr)
        .chain(CGB_ONLY.iter().map(|&(addr, _)| addr))
        .collect();

    (0xFF00..=0xFF7Fu16)
        .filter(|addr| !named.contains(addr))
        .collect()
}

#[test]
fn the_transcribed_column_partitions_the_io_range_exactly_once() {
    let mut seen: Vec<u16> = DMG_COLUMN
        .iter()
        .map(|&(addr, _, _)| addr)
        .filter(|&addr| addr != 0xFFFF)
        .chain(CGB_ONLY.iter().map(|&(addr, _)| addr))
        .chain(unnamed_io_addresses())
        .collect();

    let total = seen.len();
    seen.sort_unstable();
    seen.dedup();

    assert_eq!(total, seen.len(), "a transcrição repete algum endereço");
    assert_eq!(
        seen,
        (0xFF00..=0xFF7Fu16).collect::<Vec<_>>(),
        "as três listas juntas têm de dar exatamente $FF00–$FF7F"
    );
}

#[test]
fn every_named_register_holds_the_dmg_column_value_at_hand_off() {
    let bus = bus();

    for &(addr, name, cell) in DMG_COLUMN {
        let Some(expected) = cell else { continue };

        assert_eq!(
            bus.read(addr),
            expected,
            "${addr:04X} ({name}) devia ser ${expected:02X} no hand-off da boot ROM"
        );
    }
}

#[test]
fn this_is_the_dmg_mgb_column_and_not_one_of_the_other_three() {
    let bus = bus();

    let wrong_column = [
        (0xFF04u16, "DIV", 0x18u8, "DMG0"),
        (0xFF41, "STAT", 0x81, "DMG0"),
        (0xFF44, "LY", 0x91, "DMG0"),
        (0xFF26, "NR52", 0xF0, "SGB / SGB2"),
        (0xFF02, "SC", 0x7F, "CGB / AGB"),
        (0xFF46, "DMA", 0x00, "CGB / AGB"),
    ];

    for (addr, name, value, column) in wrong_column {
        assert_ne!(
            bus.read(addr),
            value,
            "${addr:04X} ({name}) saiu com ${value:02X}, que é o valor da coluna \
             {column} — outro console"
        );
    }
}

#[test]
fn obp0_and_obp1_are_uninitialized_in_the_spec_and_zero_by_choice_here() {
    let mut bus = bus();

    for addr in [0xFF48u16, 0xFF49] {
        assert_eq!(
            bus.read(addr),
            0x00,
            "${addr:04X}: a tabela marca `??`; a constante é escolha deste \
             emulador, não valor de hardware"
        );
    }

    for addr in [0xFF48u16, 0xFF49] {
        bus.write(addr, PROBE);
        assert_eq!(
            bus.read(addr),
            PROBE,
            "${addr:04X} é registrador de verdade: não ter valor inicial na spec \
             não quer dizer não ter célula"
        );
    }
}

#[test]
fn the_cgb_only_registers_do_not_exist_on_this_console() {
    let mut bus = bus();

    for &(addr, name) in CGB_ONLY {
        assert_eq!(
            bus.read(addr),
            OPEN_BUS,
            "${addr:04X} ({name}) é `---` na coluna DMG: o registrador não \
             existe neste console, e `---` não é $00"
        );

        bus.write(addr, PROBE);
        assert_eq!(
            bus.read(addr),
            OPEN_BUS,
            "${addr:04X} ({name}) guardou uma escrita sem existir"
        );
    }
}

#[test]
fn the_addresses_the_table_never_mentions_have_no_owner_yet() {
    let mut bus = bus();
    let unnamed = unnamed_io_addresses();

    assert_eq!(
        unnamed.len(),
        72,
        "dos 128 endereços de I/O a tabela dá valor a 41 e marca 15 como `---`; \
         sobre os outros 72 ela não diz nada"
    );

    for addr in unnamed {
        assert_eq!(
            bus.read(addr),
            OPEN_BUS,
            "${addr:04X} não está na tabela § Hardware registers"
        );

        bus.write(addr, PROBE);
        assert_eq!(
            bus.read(addr),
            OPEN_BUS,
            "${addr:04X} guardou uma escrita, mas ninguém respondia por ele"
        );
    }
}

#[test]
fn ie_starts_at_zero_and_is_a_register_of_its_own() {
    let mut bus = bus();

    assert_eq!(
        bus.read(0xFFFF),
        0x00,
        "$FFFF (IE) é a última linha da tabela, e a coluna DMG dá $00"
    );

    bus.write(0xFFFF, PROBE);
    assert_eq!(
        bus.read(0xFFFF),
        PROBE,
        "$FFFF tem célula própria: é registrador, e o despacho de interrupções \
         (2.2) vai lê-lo e escrevê-lo"
    );
    assert_eq!(
        bus.read(0xFFFE),
        0x00,
        "escrever no IE mexeu no último byte da HRAM: $FFFF não é o 128º byte \
         dela"
    );
}

#[test]
fn the_named_registers_have_storage_and_no_read_semantics_yet() {
    let mut bus = bus();

    for (addr, name) in [(0xFF04u16, "DIV"), (0xFF44, "LY"), (0xFF07, "TAC")] {
        bus.write(addr, PROBE);
        assert_eq!(
            bus.read(addr),
            PROBE,
            "${addr:04X} ({name}) hoje é um byte cru: sem máscara, sem read-only, \
             sem efeito colateral. Se o componente dono chegou, este teste é que \
             está velho"
        );
    }
}

#[test]
fn the_hand_off_state_is_what_bus_new_gives_because_the_boot_rom_is_skipped() {
    let bus = bus();

    assert_ne!(
        bus.read(0xFF40),
        0x00,
        "LCDC = $91 no hand-off: um `Bus::new` que zerasse a região de I/O \
         entregaria a tela desligada ao cartucho"
    );
}
