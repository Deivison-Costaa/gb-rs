//! ROADMAP 1.2a — decodificação de endereço e RAM interna do `Bus`.
//!
//! Spec: `docs/reference/01-memory-map.md` (Pan Docs `fe246067b695`), três
//! seções: § Memory Map (a tabela), § Echo RAM e § FEA0–FEFF range.
//!
//! O que está sendo medido aqui é **quem responde a cada endereço**, e só isso.
//! Valores iniciais são o 1.2b; contagem de M-cycles é o 1.3. `Bus::read` e
//! `Bus::write` não avançam o tempo — a R2 vive no laço que os chama, não neles.
//!
//! Três afirmações da spec merecem teste próprio porque a intuição erra as três:
//!
//! 1. o echo é **mais curto que a fonte** (espelha `$C000`–`$DDFF`, não os 8 KiB);
//! 2. a região proibida lê **`$00`** no DMG fora de bloqueio de OAM, não `$FF`;
//! 3. a HRAM tem **127** bytes — `$FFFF` é o `IE`, não o 128º.
//!
//! `unwrap`/`expect` são permitidos aqui: a R6 proíbe fora de teste.

use std::cell::RefCell;
use std::rc::Rc;

use gb_core::bus::{Bus, Region};
use gb_core::cart::{Cartridge, OPEN_BUS};

/// Conteúdo esperado de cada endereço nos testes de RAM.
///
/// O XOR com o byte alto faz `pattern` distinguir um endereço do seu vizinho
/// **e** do endereço 4 KiB adiante — que é exatamente o par que uma máscara
/// curta demais (`& 0x0FFF`, dobrando `$D000`–`$DFFF` sobre `$C000`–`$CFFF`)
/// confundiria. Padrão constante não separaria "guardou" de "aliasou".
fn pattern(addr: u16) -> u8 {
    (addr as u8) ^ ((addr >> 8) as u8)
}

// ---------------------------------------------------------------------------
// Cartucho-espião: para medir roteamento, e não conteúdo
// ---------------------------------------------------------------------------

/// O byte que o cartucho-espião devolve para qualquer leitura.
///
/// Tem de diferir de [`OPEN_BUS`]: é ele que separa "o `Bus` roteou para o
/// cartucho" de "ninguém respondeu". Com `$FF` dos dois lados, um roteamento
/// que mandasse a VRAM inteira para o cartucho passaria despercebido.
const CARTRIDGE_MARKER: u8 = 0xC7;

/// Valor arbitrário usado nas escritas de teste. Não é `$00` nem `$FF` de
/// propósito — a § Echo RAM avisa que esses dois valores não distinguem
/// espelho de RAM não inicializada.
const PROBE: u8 = 0x5A;

#[derive(Default)]
struct SpyLog {
    reads: RefCell<Vec<u16>>,
    writes: RefCell<Vec<(u16, u8)>>,
}

/// Cartucho que não guarda nada: só anota o que o `Bus` lhe manda.
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

/// Os endereços que o cartucho tem direito de ver, em ordem crescente.
fn cartridge_windows() -> Vec<u16> {
    (0x0000..=0x7FFFu16).chain(0xA000..=0xBFFF).collect()
}

// ---------------------------------------------------------------------------
// § Memory Map — a tabela, transcrita e conferida linha a linha
// ---------------------------------------------------------------------------

/// A tabela § Memory Map do Pan Docs, exatamente como ela está lá.
///
/// As duas linhas de ROM e as duas de WRAM ficam separadas de propósito, ainda
/// que o DMG as trate como faixas contíguas: quem as divide é o banking, que é
/// do mapeador (`$4000`–`$7FFF`) e do CGB (`$D000`–`$DFFF`). Fundir as linhas
/// aqui apagaria do teste a razão de a tabela ter quatro.
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

/// Confere a **transcrição**, antes de usá-la para julgar o código.
///
/// Um buraco ou uma sobreposição na tabela acima passaria despercebido no teste
/// seguinte (endereço não coberto simplesmente não seria verificado), e o
/// resultado seria um verde que não mediu o mapa inteiro. Nota 8 do `STATUS.md`:
/// guarda que só pode passar não é guarda.
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

/// O mapa inteiro, endereço por endereço.
///
/// 65536 comparações custam milissegundos e dispensam a discussão sobre quais
/// bordas "bastariam": qualquer off-by-one em qualquer fronteira dói aqui.
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

/// Os 8 KiB inteiros, um byte por endereço, sem endereço pisando no outro.
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

/// A spec diz que WRAM e HRAM são **aleatórias** ao ligar, e que os emuladores
/// escolhem uma constante:
///
/// > The console's WRAM and HRAM are random on power-up. […] Emulation of
/// > uninitialized RAM is inconsistent: some emulators fill RAM with a constant
/// > on startup (typically $00 or $FF) […]
///
/// Este projeto escolhe `$00`. O teste existe para que a escolha fique **dita**
/// em algum lugar que quebre se mudar — não porque `$00` seja o valor certo.
/// Jogo que dependa disto tem bug: a própria spec desaconselha.
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

/// Espelho nos dois sentidos: não é uma cópia, é a mesma memória.
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

/// **O espelho é mais curto que a fonte.**
///
/// `$E000`–`$FDFF` são 7680 bytes; a WRAM tem 8192. A § Memory Map escreve
/// "mirror of C000–DDFF", e não "mirror of WRAM": os últimos 512 bytes
/// (`$DE00`–`$DFFF`) **não têm echo nenhum**. Quem lembra do echo como "a WRAM
/// espelhada" inventa 512 bytes de espelho que não existem — e o sintoma
/// apareceria só no jogo que usa o topo da WRAM.
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

/// As duas pontas do espelho, ditas como endereços e não como aritmética.
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

/// **Lê `$00`, não `$FF`.** A seção inteira, no que cabe ao DMG:
///
/// > This area returns $FF when OAM is blocked, and otherwise the behavior
/// > depends on the hardware revision.
/// > - On DMG, MGB, SGB, and SGB2, reads during OAM block trigger OAM
/// >   corruption. Reads otherwise return $00.
///
/// O `$FF` está preso à condição "OAM bloqueada", que depende do modo da PPU.
/// Não há PPU (M3), logo a OAM nunca está bloqueada, logo a leitura é `$00`.
/// "Não usável" leva a mão para `$FF` — mas `$FF` aqui seria afirmar um
/// bloqueio que este emulador ainda não sabe ter.
///
/// Quando o 3.6 trouxer o bloqueio por modo, este teste passa a valer só fora
/// dele; a corrupção de OAM que a spec descreve junto é o 7.2 (`oam_bug`).
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

/// 127 bytes, não 128. A § Memory Map dá `$FF80`–`$FFFE` à HRAM e reserva
/// `$FFFF` para o `IE`; contar a HRAM até `$FFFF` cria um 128º byte que na
/// verdade é um registrador de interrupções (ROADMAP 2.2).
///
/// A afirmação sobre `$FFFF` é feita **pela região**, e não por aliasing. Uma
/// HRAM de 128 bytes que engole `$FFFF` não pisa em `$FFFE` — ela dá um índice
/// novo ao endereço anexado, e um teste que só procurasse colisão passaria
/// verde. Foi o que aconteceu na primeira execução desta suíte.
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

/// Varre os 65536 endereços e compara a lista do que o cartucho **viu** com a
/// lista do que ele deveria ver. Afirma as duas metades de uma vez: nenhuma
/// janela do cartucho ficou de fora, e nenhum endereço de fora dele entrou.
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

/// VRAM e OAM existem no mapa e **não** existem no código: os componentes que
/// respondem por elas são de itens posteriores.
///
/// Barramento aberto aqui não é uma afirmação sobre o hardware — é a ausência
/// de componente ligado. O teste fixa essa ausência para que ela seja uma
/// decisão visível, e não uma lacuna que alguém descubra depurando a PPU.
/// Entrar em pânico seria pior: `read` de emulador é o último lugar onde se
/// quer descobrir erro de roteamento (mesma decisão do `NoMbc`, 0.4).
///
/// **`$FF00`–`$FF7F` e `$FFFF` saíram desta lista no 1.2b-ii**, e é o que este
/// teste previa que aconteceria. Não saíram inteiros: os 41 endereços que a
/// tabela § Hardware registers nomeia ganharam célula e valor inicial, e os
/// outros 72 continuam sem dono. Quem mede isso agora é `bus_boot_state.rs`,
/// onde a lista tem o recorte certo — repeti-la aqui daria duas listas para
/// manter em sincronia e uma delas ficaria velha.
#[test]
fn the_regions_without_an_owner_are_open_bus_and_swallow_writes() {
    let mut bus = bus();

    let pending = [
        (0x8000u16, "VRAM", "ROADMAP 3.1"),
        (0x9FFF, "VRAM", "ROADMAP 3.1"),
        (0xFE00, "OAM", "ROADMAP 3.1"),
        (0xFE9F, "OAM", "ROADMAP 3.1"),
    ];

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
