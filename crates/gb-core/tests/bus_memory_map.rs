//! ROADMAP 1.2a — decodificação de endereço e RAM interna do `Bus`.

use std::cell::RefCell;
use std::rc::Rc;

use gb_core::bus::{Bus, Region};
use gb_core::cart::{Cartridge, OPEN_BUS};

fn pattern(addr: u16) -> u8 {
    (addr as u8) ^ ((addr >> 8) as u8)
}

// ---------------------------------------------------------------------------
// Cartucho-espião: para medir roteamento, e não conteúdo
// ---------------------------------------------------------------------------

const CARTRIDGE_MARKER: u8 = 0xC7;

const PROBE: u8 = 0x5A;

#[derive(Default)]
struct SpyLog {
    reads: RefCell<Vec<u16>>,
    writes: RefCell<Vec<(u16, u8)>>,
}

struct SpyCartridge(Rc<SpyLog>);

impl Cartridge for SpyCartridge {
    fn read(&self, addr: u16) -> u8 {
        self.0.reads.borrow_mut().push(addr);
        CARTRIDGE_MARKER
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.0.writes.borrow_mut().push((addr, value));
    }
}

fn spied_bus() -> (Bus, Rc<SpyLog>) {
    let log = Rc::new(SpyLog::default());
    (Bus::new(Box::new(SpyCartridge(Rc::clone(&log)))), log)
}

fn bus() -> Bus {
    spied_bus().0
}

fn cartridge_windows() -> Vec<u16> {
    (0x0000..=0x7FFFu16).chain(0xA000..=0xBFFF).collect()
}

// ---------------------------------------------------------------------------
// § Memory Map — a tabela, transcrita e conferida linha a linha
// ---------------------------------------------------------------------------

const MEMORY_MAP: &[(u16, u16, Region)] = &[
    (0x0000, 0x3FFF, Region::CartridgeRom),
    (0x4000, 0x7FFF, Region::CartridgeRom),
    (0x8000, 0x9FFF, Region::VideoRam),
    (0xA000, 0xBFFF, Region::ExternalRam),
    (0xC000, 0xCFFF, Region::WorkRam),
    (0xD000, 0xDFFF, Region::WorkRam),
    (0xE000, 0xFDFF, Region::EchoRam),
    (0xFE00, 0xFE9F, Region::ObjectAttributeMemory),
    (0xFEA0, 0xFEFF, Region::NotUsable),
    (0xFF00, 0xFF7F, Region::IoRegisters),
    (0xFF80, 0xFFFE, Region::HighRam),
    (0xFFFF, 0xFFFF, Region::InterruptEnable),
];

#[test]
fn the_transcribed_table_covers_the_address_space_exactly_once() {
    let mut next = 0u32;

    for &(start, end, region) in MEMORY_MAP {
        assert!(start <= end, "{region:?}: faixa invertida");
        assert_eq!(
            u32::from(start),
            next,
            "{region:?} começa em ${start:04X}, mas a faixa anterior terminou \
             em ${:04X}: a transcrição tem buraco ou sobreposição",
            next.saturating_sub(1)
        );
        next = u32::from(end) + 1;
    }

    assert_eq!(
        next, 0x1_0000,
        "a tabela tem de terminar em $FFFF — o barramento de endereços do \
         Game Boy tem 16 bits e não sobra endereço sem dono"
    );
}

#[test]
fn every_address_decodes_to_the_region_the_pandocs_table_gives_it() {
    for &(start, end, expected) in MEMORY_MAP {
        for addr in start..=end {
            assert_eq!(
                Region::of(addr),
                expected,
                "${addr:04X} está em ${start:04X}–${end:04X}, que a § Memory Map \
                 atribui a {expected:?}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// WRAM — `$C000`–`$DFFF`, 8 KiB, sem banco no DMG
// ---------------------------------------------------------------------------

#[test]
fn work_ram_holds_eight_kib_with_no_aliasing() {
    let mut bus = bus();

    for addr in 0xC000..=0xDFFFu16 {
        bus.write(addr, pattern(addr));
    }
    for addr in 0xC000..=0xDFFFu16 {
        assert_eq!(
            bus.read(addr),
            pattern(addr),
            "${addr:04X} devia guardar o próprio byte: a WRAM do DMG são 8 KiB \
             contíguos, e o banco chaveado de $D000–$DFFF é só CGB"
        );
    }
}

#[test]
fn work_ram_and_high_ram_start_zeroed_by_an_explicit_choice() {
    let bus = bus();

    for addr in [0xC000u16, 0xD000, 0xDFFF, 0xFF80, 0xFFFE] {
        assert_eq!(
            bus.read(addr),
            0x00,
            "${addr:04X}: RAM interna começa zerada neste emulador"
        );
    }
}

// ---------------------------------------------------------------------------
// § Echo RAM — "the same effect as reads and writes to C000-DDFF"
// ---------------------------------------------------------------------------

#[test]
fn echo_ram_is_work_ram_seen_from_e000() {
    let mut bus = bus();

    for addr in 0xE000..=0xFDFFu16 {
        let source = addr - 0x2000;
        bus.write(source, pattern(addr));
        assert_eq!(
            bus.read(addr),
            pattern(addr),
            "${addr:04X} devia ler o que foi escrito em ${source:04X}"
        );
    }

    for addr in 0xE000..=0xFDFFu16 {
        let source = addr - 0x2000;
        bus.write(addr, pattern(source));
        assert_eq!(
            bus.read(source),
            pattern(source),
            "escrever em ${addr:04X} devia pegar em ${source:04X}: o echo não é \
             cópia, é a mesma célula vista por outro endereço"
        );
    }
}

#[test]
fn the_top_of_work_ram_has_no_echo_at_all() {
    let mut bus = bus();

    const MIRRORED: u8 = 0x11;
    const BEYOND_THE_MIRROR: u8 = 0x22;

    for addr in 0xC000..=0xDDFFu16 {
        bus.write(addr, MIRRORED);
    }
    for addr in 0xDE00..=0xDFFFu16 {
        bus.write(addr, BEYOND_THE_MIRROR);
    }

    for addr in 0xE000..=0xFDFFu16 {
        assert_eq!(
            bus.read(addr),
            MIRRORED,
            "${addr:04X} leu o topo da WRAM: nenhum endereço do echo alcança \
             $DE00–$DFFF"
        );
    }

    for addr in 0xE000..=0xFDFFu16 {
        bus.write(addr, MIRRORED);
    }
    for addr in 0xDE00..=0xDFFFu16 {
        assert_eq!(
            bus.read(addr),
            BEYOND_THE_MIRROR,
            "${addr:04X} foi alterado por uma escrita no echo, e não devia: \
             o espelho termina em $DDFF"
        );
    }
}

#[test]
fn the_echo_window_starts_at_c000_and_ends_at_ddff() {
    let mut bus = bus();

    bus.write(0xC000, 0x01);
    bus.write(0xDDFF, 0x02);

    assert_eq!(bus.read(0xE000), 0x01, "$E000 é o espelho de $C000");
    assert_eq!(bus.read(0xFDFF), 0x02, "$FDFF é o espelho de $DDFF");
}

// ---------------------------------------------------------------------------
// § FEA0–FEFF range — a região proibida
// ---------------------------------------------------------------------------

#[test]
fn the_not_usable_range_reads_zero_while_there_is_no_oam_blocking() {
    let mut bus = bus();

    for addr in 0xFEA0..=0xFEFFu16 {
        assert_eq!(
            bus.read(addr),
            0x00,
            "${addr:04X}: sem PPU a OAM nunca está bloqueada, e no DMG a \
             leitura fora de bloqueio é $00"
        );
    }

    for addr in 0xFEA0..=0xFEFFu16 {
        bus.write(addr, PROBE);
        assert_eq!(
            bus.read(addr),
            0x00,
            "${addr:04X} guardou uma escrita: a spec descreve a leitura como \
             constante, então não há célula aqui para escrever"
        );
    }
}

// ---------------------------------------------------------------------------
// HRAM — `$FF80`–`$FFFE`, e o `$FFFF` que não é dela
// ---------------------------------------------------------------------------

#[test]
fn high_ram_is_127_bytes_and_stops_before_the_ie_register() {
    let mut bus = bus();

    for addr in 0xFF80..=0xFFFEu16 {
        bus.write(addr, pattern(addr));
    }
    for addr in 0xFF80..=0xFFFEu16 {
        assert_eq!(
            bus.read(addr),
            pattern(addr),
            "${addr:04X} é HRAM e devia guardar o próprio byte"
        );
    }

    assert_eq!(
        Region::of(0xFFFF),
        Region::InterruptEnable,
        "$FFFF é o registrador IE, e anexá-lo à HRAM daria a ela um 128º byte \
         que o mapa de memória não tem"
    );

    bus.write(0xFFFF, PROBE);
    assert_eq!(
        bus.read(0xFFFE),
        pattern(0xFFFE),
        "escrever em $FFFF mexeu no último byte da HRAM: a HRAM termina em $FFFE"
    );
}

// ---------------------------------------------------------------------------
// Roteamento para o cartucho — duas janelas, e **só** elas
// ---------------------------------------------------------------------------

#[test]
fn the_cartridge_windows_answer_with_what_the_cartridge_says() {
    let (bus, _log) = spied_bus();

    for addr in [0x0000u16, 0x3FFF, 0x4000, 0x7FFF, 0xA000, 0xBFFF] {
        assert_eq!(
            bus.read(addr),
            CARTRIDGE_MARKER,
            "${addr:04X} pertence ao cartucho: quem responde é ele"
        );
    }
}

#[test]
fn only_the_two_cartridge_windows_reach_the_cartridge_on_reads() {
    let (bus, log) = spied_bus();

    for addr in 0x0000..=0xFFFFu16 {
        let _ = bus.read(addr);
    }

    assert_eq!(
        *log.reads.borrow(),
        cartridge_windows(),
        "o cartucho só responde por $0000–$7FFF e $A000–$BFFF; qualquer outro \
         endereço que chegue nele é erro de roteamento"
    );
}

#[test]
fn only_the_two_cartridge_windows_reach_the_cartridge_on_writes() {
    let (mut bus, log) = spied_bus();

    for addr in 0x0000..=0xFFFFu16 {
        bus.write(addr, PROBE);
    }

    let expected: Vec<(u16, u8)> = cartridge_windows()
        .into_iter()
        .map(|a| (a, PROBE))
        .collect();
    assert_eq!(
        *log.writes.borrow(),
        expected,
        "escrita em ROM é como se fala com o mapeador (4.2): o `Bus` tem de \
         entregá-la, e entregar só a das janelas do cartucho"
    );
}

// ---------------------------------------------------------------------------
// O que ainda não tem dono
// ---------------------------------------------------------------------------

#[test]
fn the_regions_without_an_owner_are_open_bus_and_swallow_writes() {
    let mut bus = bus();

    let pending: [(u16, &str, &str); 0] = [];

    for (addr, name, item) in pending {
        assert_eq!(
            bus.read(addr),
            OPEN_BUS,
            "${addr:04X} ({name}) ainda não tem componente: quem o liga é o {item}"
        );
        bus.write(addr, PROBE);
        assert_eq!(
            bus.read(addr),
            OPEN_BUS,
            "${addr:04X} ({name}) guardou uma escrita sem ter memória: se o \
             {item} chegou, este teste é que está velho"
        );
    }
}
