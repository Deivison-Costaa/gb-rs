//! ROADMAP 0.4 — o trait `Cartridge` e o cartucho sem mapeador.

use gb_core::cart::{Cartridge, CartridgeError, HeaderError, NoMbc, OPEN_BUS, load};

const KIB: usize = 1024;

const CARTRIDGE_TYPE: usize = 0x0147;
const HEADER_CHECKSUM: usize = 0x014D;
const MIN_ROM_LEN: usize = 0x0150;
const ROM_ONLY: u8 = 0x00;

fn pattern(index: usize) -> u8 {
    (index as u8) ^ ((index >> 8) as u8)
}

fn patterned_rom(len: usize) -> Vec<u8> {
    (0..len).map(pattern).collect()
}

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

#[test]
fn load_refuses_types_it_cannot_map() {
    for code in [0x19u8, 0x42, 0xFF] {
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

#[test]
fn load_does_not_judge_the_header_checksum() {
    let mut rom = rom_of_type(ROM_ONLY, 32 * KIB);
    rom[HEADER_CHECKSUM] = rom[HEADER_CHECKSUM].wrapping_add(1);

    assert!(
        load(rom).is_ok(),
        "checksum errado é diagnóstico do `info`, não erro de montagem"
    );
}

#[test]
fn error_messages_carry_the_offending_value() {
    let too_large = NoMbc::new(patterned_rom(64 * KIB))
        .expect_err("64 KiB não cabe")
        .to_string();
    assert!(
        too_large.contains("65536"),
        "a mensagem devia dizer o tamanho recusado, e diz: {too_large:?}"
    );

    let unsupported = load(rom_of_type(0x19, 32 * KIB))
        .err()
        .expect("MBC5 não é suportado")
        .to_string();
    assert!(
        unsupported.contains("$19") && unsupported.contains("MBC5"),
        "a mensagem devia nomear o tipo recusado, e diz: {unsupported:?}"
    );
}
