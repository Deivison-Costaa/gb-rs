//! ROADMAP 0.4 — o trait `Cartridge` e o cartucho sem mapeador.
//!
//! Spec: `docs/reference/08-cartridges-mbc.md` § No MBC (Pan Docs
//! `fe246067b695`), que cabe em três linhas:
//!
//! > Small games of not more than 32 KiB ROM do not require a MBC chip for
//! > ROM banking. The ROM is directly mapped to memory at $0000-7FFF.
//! > Optionally up to 8 KiB of RAM could be connected at $A000-BFFF, using
//! > a discrete logic decoder in place of a full MBC chip.
//!
//! Três afirmações testáveis saem daí: **não mais que 32 KiB**, **mapeamento
//! direto** (nada de banco chaveado em `$4000`–`$7FFF`) e **RAM opcional** —
//! esta última fora do escopo do 0.4, o que os testes de `load` fixam.
//!
//! ROMs sintéticas, montadas byte a byte. Nada de ler `tests/roms/`: é
//! gitignored e baixado por script, então o teste passaria vazio na máquina de
//! quem não rodou o download — o modo de falha da nota 8 do `STATUS.md`.
//!
//! `unwrap`/`expect` são permitidos aqui: a R6 proíbe fora de teste.

use gb_core::cart::{Cartridge, CartridgeError, HeaderError, NoMbc, OPEN_BUS, load};

const KIB: usize = 1024;

/// `$0147` — o tipo do cartucho, o byte que `load` despacha.
const CARTRIDGE_TYPE: usize = 0x0147;
/// `$014D` — o checksum do cabeçalho.
const HEADER_CHECKSUM: usize = 0x014D;
/// Menor ROM com o cabeçalho inteiro (`$014F` é o último byte dele).
const MIN_ROM_LEN: usize = 0x0150;
/// `$00` — ROM ONLY, o único tipo que o 0.4 monta.
const ROM_ONLY: u8 = 0x00;

/// Conteúdo de cada posição da ROM de teste.
///
/// Não é enfeite: o padrão precisa distinguir um endereço do seu vizinho **e**
/// do endereço 16 KiB adiante, senão o teste do mapeamento direto não separa
/// "mapeou certo" de "leu o banco errado". O XOR com o byte alto garante que
/// `pattern(a)` e `pattern(a + 0x4000)` sempre difiram — é exatamente o bit
/// que um mapeador mexeria.
fn pattern(index: usize) -> u8 {
    (index as u8) ^ ((index >> 8) as u8)
}

fn patterned_rom(len: usize) -> Vec<u8> {
    (0..len).map(pattern).collect()
}

/// ROM com cabeçalho legível e o tipo de cartucho pedido em `$0147`.
///
/// O checksum fica **errado de propósito** (o padrão escreve `$014D` como
/// qualquer outro byte): `load` não é o boot ROM e não julga cabeçalho — ver
/// `load_does_not_judge_the_header_checksum`.
fn rom_of_type(code: u8, len: usize) -> Vec<u8> {
    assert!(len >= MIN_ROM_LEN, "ROM do fixture não tem cabeçalho");
    let mut rom = patterned_rom(len);
    rom[CARTRIDGE_TYPE] = code;
    rom
}

fn nombc(rom: Vec<u8>) -> NoMbc {
    NoMbc::new(rom).expect("ROM do fixture cabe num cartucho sem MBC")
}

// ---------------------------------------------------------------------------
// Mapeamento — § No MBC: "directly mapped to memory at $0000-7FFF"
// ---------------------------------------------------------------------------

/// A afirmação central da spec, endereço por endereço.
///
/// Varre os 32768 endereços em vez de amostrar as bordas: qualquer deslocamento
/// (`addr - 0x4000` no segundo banco, um off-by-one na fronteira) tem de doer
/// aqui, e um `for` sobre a janela inteira é mais barato de ler do que a lista
/// de casos que se acha que cobre o assunto.
#[test]
fn maps_the_whole_32_kib_directly() {
    let rom = patterned_rom(32 * KIB);
    let cart = nombc(rom.clone());

    for addr in 0x0000..=0x7FFFu16 {
        assert_eq!(
            cart.read(addr),
            rom[addr as usize],
            "${addr:04X} devia ler o byte de mesmo índice na ROM: \
             sem MBC não há banco a escolher"
        );
    }
}

/// 32 KiB é o teto, e o teto é inclusivo — a spec diz "not more than 32 KiB".
#[test]
fn rejects_a_rom_one_byte_past_32_kib() {
    let len = 32 * KIB + 1;
    assert_eq!(
        NoMbc::new(patterned_rom(len)).err(),
        Some(CartridgeError::RomTooLarge { len }),
        "$0000-$7FFF endereça 32 KiB; o byte 32769 não tem como ser lido \
         sem mapeador, e aceitá-lo em silêncio esconderia cartucho \
         mal-identificado"
    );
}

/// ROM menor que a janela: o que sobra não é ROM nenhuma.
///
/// A spec descreve o cartucho de 32 KiB e **não diz** o que responde acima do
/// fim de um chip menor. Espelhar (`addr % rom.len()`) seria inventar fiação;
/// barramento aberto é o que o Pan Docs documenta para leitura sem quem
/// responda (§ MBC1, `$A000`–`$BFFF`: "reads return open bus values (often
/// $FF, but not guaranteed)").
#[test]
fn reads_past_the_end_of_a_short_rom_are_open_bus() {
    let rom = patterned_rom(16 * KIB);
    let cart = nombc(rom.clone());

    assert_eq!(
        cart.read(0x3FFF),
        rom[0x3FFF],
        "$3FFF é o último byte que existe nesta ROM"
    );
    for addr in [0x4000u16, 0x5A5A, 0x7FFF] {
        assert_eq!(
            cart.read(addr),
            OPEN_BUS,
            "${addr:04X} está além do fim da ROM: não é para espelhar o começo"
        );
    }
}

/// Sem MBC não há registradores — e as janelas que o MBC1 usaria para eles são
/// justamente `$0000`–`$1FFF`, `$2000`–`$3FFF`, `$4000`–`$5FFF` e
/// `$6000`–`$7FFF` (4.2). Escrever ali num cartucho ROM ONLY não muda nada.
#[test]
fn writes_to_the_rom_area_are_ignored() {
    let rom = patterned_rom(32 * KIB);
    let mut cart = nombc(rom.clone());

    for addr in [0x0000u16, 0x2000, 0x4000, 0x6000, 0x7FFF] {
        cart.write(addr, 0x0A);
        assert_eq!(
            cart.read(addr),
            rom[addr as usize],
            "${addr:04X} é ROM: escrita não pega, nem como registrador"
        );
    }
}

// ---------------------------------------------------------------------------
// RAM externa — fora do escopo do 0.4, e o teste diz que é de propósito
// ---------------------------------------------------------------------------

/// `$A000`–`$BFFF` sem RAM conectada: ninguém responde, nada é guardado.
#[test]
fn the_external_ram_window_is_open_bus() {
    let mut cart = nombc(patterned_rom(32 * KIB));

    for addr in [0xA000u16, 0xB000, 0xBFFF] {
        assert_eq!(
            cart.read(addr),
            OPEN_BUS,
            "${addr:04X}: este cartucho não tem RAM"
        );
        cart.write(addr, 0x0A);
        assert_eq!(
            cart.read(addr),
            OPEN_BUS,
            "${addr:04X}: escrever numa RAM que não existe não a cria"
        );
    }
}

/// Endereços que o `Bus` (1.2) nunca vai rotear para cá. O contrato do trait é
/// definido só nas duas janelas do cartucho; responder barramento aberto no
/// resto é para o dia em que alguém errar o roteamento — pânico em `read` de
/// emulador é o pior lugar possível para descobrir isso.
#[test]
fn addresses_outside_the_cartridge_are_open_bus() {
    let cart = nombc(patterned_rom(32 * KIB));

    for addr in [0x8000u16, 0x9FFF, 0xC000, 0xFF44, 0xFFFF] {
        assert_eq!(
            cart.read(addr),
            OPEN_BUS,
            "${addr:04X} não pertence ao cartucho"
        );
    }
}

// ---------------------------------------------------------------------------
// Despacho por `$0147` — a primeira vez que o core **usa** o tipo do cartucho
// ---------------------------------------------------------------------------

#[test]
fn load_dispatches_rom_only_to_nombc() {
    let rom = rom_of_type(ROM_ONLY, 32 * KIB);
    let cart = load(rom.clone()).expect("$0147 = $00 é ROM ONLY");

    assert_eq!(
        cart.read(0x7FFF),
        rom[0x7FFF],
        "o cartucho montado tem de ler a ROM que recebeu"
    );
}

/// Tipo com mapeador que ainda não existe, e tipo que não existe em lugar
/// nenhum, dão o mesmo veredito: `load` não monta.
///
/// O caso `$42` importa por si: código fora da tabela do Pan Docs faz
/// `CartridgeType::name()` devolver `None`, e tratar "não sei o que é" como
/// ROM ONLY seria rodar um cartucho desconhecido como se fosse simples.
#[test]
fn load_refuses_types_it_cannot_map() {
    for code in [0x01u8, 0x03, 0x05, 0x11, 0x13, 0x19, 0x42, 0xFF] {
        let rom = rom_of_type(code, 32 * KIB);
        match load(rom) {
            Err(CartridgeError::UnsupportedType { cartridge_type }) => assert_eq!(
                cartridge_type.code(),
                code,
                "o erro tem de dizer qual tipo recusou"
            ),
            other => panic!(
                "${code:02X} não devia montar, mas load deu {:?}",
                other.map(|_| ())
            ),
        }
    }
}

/// `$08` e `$09` são cartucho sem MBC **com** RAM — a segunda frase da § No
/// MBC. Ainda assim são recusados, e o motivo está na tabela de `$0147`:
///
/// > No licensed cartridge makes use of this option. The exact behavior is
/// > unknown.
///
/// Montá-los como ROM ONLY daria um cartucho cuja RAM lê `$FF` e engole
/// escrita: o jogo perderia o save em silêncio. Recusar é a resposta honesta
/// enquanto não houver comportamento documentado para copiar.
#[test]
fn load_refuses_rom_plus_ram_because_the_spec_calls_it_unknown() {
    for code in [0x08u8, 0x09] {
        let rom = rom_of_type(code, 32 * KIB);
        assert!(
            matches!(load(rom), Err(CartridgeError::UnsupportedType { .. })),
            "${code:02X} tem RAM que este item não implementa: recusar > fingir"
        );
    }
}

#[test]
fn load_refuses_a_rom_only_larger_than_32_kib() {
    let len = 64 * KIB;
    assert_eq!(
        load(rom_of_type(ROM_ONLY, len)).err(),
        Some(CartridgeError::RomTooLarge { len }),
        "cabeçalho dizendo ROM ONLY não faz 64 KiB caberem em $0000-$7FFF"
    );
}

#[test]
fn load_propagates_the_header_error() {
    let len = MIN_ROM_LEN - 1;
    assert_eq!(
        load(patterned_rom(len)).err(),
        Some(CartridgeError::Header(HeaderError::TooShort { len })),
        "sem cabeçalho não há $0147, e sem $0147 não há despacho"
    );
}

/// Quem trava a máquina com checksum errado é o boot ROM, e este emulador o
/// pula (1.2). O mapeador não é juiz de cabeçalho: recusar aqui tornaria
/// indiagnosticável justamente a ROM corrompida que alguém quer investigar —
/// a mesma decisão que o `gb-cli info` tomou na 0006.
#[test]
fn load_does_not_judge_the_header_checksum() {
    let mut rom = rom_of_type(ROM_ONLY, 32 * KIB);
    rom[HEADER_CHECKSUM] = rom[HEADER_CHECKSUM].wrapping_add(1);

    assert!(
        load(rom).is_ok(),
        "checksum errado é diagnóstico do `info`, não erro de montagem"
    );
}

/// Estes erros vão parar na tela do usuário via `gb-cli`. Mensagem que não diz
/// qual byte ou qual tamanho causou o problema obriga quem lê a ir ao código.
#[test]
fn error_messages_carry_the_offending_value() {
    let too_large = NoMbc::new(patterned_rom(64 * KIB))
        .expect_err("64 KiB não cabe")
        .to_string();
    assert!(
        too_large.contains("65536"),
        "a mensagem devia dizer o tamanho recusado, e diz: {too_large:?}"
    );

    let unsupported = load(rom_of_type(0x13, 32 * KIB))
        .err()
        .expect("MBC3 não é suportado")
        .to_string();
    assert!(
        unsupported.contains("$13") && unsupported.contains("MBC3+RAM+BATTERY"),
        "a mensagem devia nomear o tipo recusado, e diz: {unsupported:?}"
    );
}
