//! ROADMAP 0.3a — o cabeçalho do cartucho, em `$0100`–`$014F`.
//!
//! Spec: `docs/reference/08-cartridges-mbc.md` § The Cartridge Header
//! (Pan Docs `fe246067b695`). Os números aqui saíram de lá e não da memória
//! do agente — ver o doc da iteração 0005 para onde as duas divergiram.
//!
//! As ROMs destes testes são **sintéticas**, montadas byte a byte. Nada de ler
//! `tests/roms/`: aquilo é gitignored e baixado por script, então um teste que
//! dependesse dele passaria vazio na máquina de quem não rodou o download —
//! que é o modo de falha da nota 8 do `STATUS.md`.
//!
//! `unwrap`/`expect` são permitidos aqui: a R6 proíbe fora de teste.

use gb_core::cart::{CartridgeHeader, HeaderError};

/// Menor ROM que contém o cabeçalho inteiro: `$014F` é o último byte dele.
const ROM_LEN: usize = 0x0150;

const TITLE: usize = 0x0134;
const CARTRIDGE_TYPE: usize = 0x0147;
const ROM_SIZE: usize = 0x0148;
const RAM_SIZE: usize = 0x0149;
const HEADER_CHECKSUM: usize = 0x014D;

/// ROM zerada do tamanho mínimo. Cada teste escreve só o campo que lhe importa.
fn blank_rom() -> Vec<u8> {
    vec![0x00; ROM_LEN]
}

/// O checksum como o boot ROM o calcula, transcrito da spec:
///
/// ```c
/// uint8_t checksum = 0;
/// for (uint16_t address = 0x0134; address <= 0x014C; address++) {
///     checksum = checksum - rom[address] - 1;
/// }
/// ```
///
/// Este é o oráculo dos testes de checksum: veio da spec, não da implementação.
/// Se a implementação copiasse este código e o teste comparasse os dois, o
/// teste não mediria nada — por isso a forma aqui é a do laço em C, e a de lá
/// é a que der na telha de quem implementa.
fn boot_rom_checksum(rom: &[u8]) -> u8 {
    let mut checksum: u8 = 0;
    for &byte in &rom[0x0134..=0x014C] {
        checksum = checksum.wrapping_sub(byte).wrapping_sub(1);
    }
    checksum
}

/// Grava em `$014D` o checksum que o boot ROM aceitaria.
fn seal(rom: &mut [u8]) {
    rom[HEADER_CHECKSUM] = boot_rom_checksum(rom);
}

fn parse(rom: &[u8]) -> CartridgeHeader {
    CartridgeHeader::parse(rom).expect("cabeçalho sintético deveria ser parseável")
}

// ---------------------------------------------------------------------------
// Limites da fatia
// ---------------------------------------------------------------------------

#[test]
fn rejects_rom_that_ends_inside_the_header() {
    let rom = vec![0x00; ROM_LEN - 1];
    assert_eq!(
        CartridgeHeader::parse(&rom),
        Err(HeaderError::TooShort { len: ROM_LEN - 1 }),
        "ROM que acaba antes de $014F não tem cabeçalho completo"
    );
}

#[test]
fn accepts_rom_of_exactly_the_header_length() {
    assert!(CartridgeHeader::parse(&blank_rom()).is_ok());
}

#[test]
fn ignores_everything_after_the_header() {
    let mut short = blank_rom();
    short[TITLE..TITLE + 5].copy_from_slice(b"ALEPH");
    seal(&mut short);

    let mut long = short.clone();
    long.resize(32 * 1024, 0xFF);

    assert_eq!(
        CartridgeHeader::parse(&short),
        CartridgeHeader::parse(&long),
        "o cabeçalho é $0100–$014F; o resto da ROM não entra nele"
    );
}

// ---------------------------------------------------------------------------
// $0134–$0143 — título
// ---------------------------------------------------------------------------

#[test]
fn title_stops_at_the_zero_padding() {
    let mut rom = blank_rom();
    rom[TITLE..TITLE + 6].copy_from_slice(b"TETRIS");

    assert_eq!(parse(&rom).title(), "TETRIS");
}

#[test]
fn title_may_fill_all_sixteen_bytes() {
    let mut rom = blank_rom();
    rom[TITLE..TITLE + 16].copy_from_slice(b"SUPER MARIOLAND2");

    assert_eq!(parse(&rom).title(), "SUPER MARIOLAND2");
}

/// Em cartuchos novos `$0143` é o CGB flag e `$013F`–`$0142` o código do
/// fabricante — bytes que **não** são título e que nem ASCII são. Sem parar no
/// primeiro byte não imprimível, um título de 15 caracteres arrastaria o
/// `$80` junto.
#[test]
fn title_stops_at_the_cgb_flag() {
    let mut rom = blank_rom();
    rom[TITLE..TITLE + 15].copy_from_slice(b"POKEMON RED VER");
    rom[0x0143] = 0x80;

    assert_eq!(parse(&rom).title(), "POKEMON RED VER");
}

/// O caso que separa "parar no primeiro byte não imprimível" de "filtrar os
/// bytes não imprimíveis": em cartucho novo, um título curto é seguido de `$00`
/// e **depois** do código do fabricante, que é ASCII imprimível. Quem filtra em
/// vez de parar cola o código do fabricante no fim do título.
#[test]
fn title_stops_before_the_manufacturer_code() {
    let mut rom = blank_rom();
    rom[TITLE..TITLE + 5].copy_from_slice(b"MARIO");
    rom[0x013F..0x0143].copy_from_slice(b"AXYE"); // código do fabricante
    rom[0x0143] = 0x80; // CGB flag

    assert_eq!(parse(&rom).title(), "MARIO");
}

#[test]
fn title_drops_trailing_spaces() {
    let mut rom = blank_rom();
    rom[TITLE..TITLE + 8].copy_from_slice(b"ZELDA   ");

    assert_eq!(parse(&rom).title(), "ZELDA");
}

#[test]
fn title_of_a_blank_rom_is_empty() {
    assert_eq!(parse(&blank_rom()).title(), "");
}

// ---------------------------------------------------------------------------
// $0147 — tipo de cartucho
// ---------------------------------------------------------------------------

#[test]
fn cartridge_type_names_the_codes_from_the_table() {
    for (code, name) in [
        (0x00, "ROM ONLY"),
        (0x01, "MBC1"),
        (0x03, "MBC1+RAM+BATTERY"),
        (0x06, "MBC2+BATTERY"),
        (0x13, "MBC3+RAM+BATTERY"),
        (0x1B, "MBC5+RAM+BATTERY"),
        (0xFF, "HuC1+RAM+BATTERY"),
    ] {
        let mut rom = blank_rom();
        rom[CARTRIDGE_TYPE] = code;
        let kind = parse(&rom).cartridge_type();

        assert_eq!(kind.code(), code);
        assert_eq!(kind.name(), Some(name), "tipo ${code:02X}");
    }
}

/// `$04` não está na tabela do Pan Docs. Não vira erro de parse: um `info` que
/// se recusa a falar sobre a ROM esquisita é justamente o que não serve para
/// diagnosticar ROM esquisita.
#[test]
fn cartridge_type_keeps_the_raw_code_when_it_is_not_in_the_table() {
    let mut rom = blank_rom();
    rom[CARTRIDGE_TYPE] = 0x04;
    let kind = parse(&rom).cartridge_type();

    assert_eq!(kind.code(), 0x04);
    assert_eq!(kind.name(), None);
}

// ---------------------------------------------------------------------------
// $0148 — tamanho da ROM
// ---------------------------------------------------------------------------

#[test]
fn rom_size_is_32_kib_shifted_by_the_code() {
    for (code, bytes, banks) in [
        (0x00, 32 * 1024, 2),
        (0x01, 64 * 1024, 4),
        (0x04, 512 * 1024, 32),
        (0x05, 1024 * 1024, 64),
        (0x08, 8 * 1024 * 1024, 512),
    ] {
        let mut rom = blank_rom();
        rom[ROM_SIZE] = code;
        let size = parse(&rom).rom_size();

        assert_eq!(size.bytes(), Some(bytes), "tamanho de ROM ${code:02X}");
        assert_eq!(size.banks(), Some(banks), "bancos de ROM ${code:02X}");
    }
}

/// `$52`/`$53`/`$54` aparecem "only in unofficial docs", sem nenhum cartucho ou
/// arquivo conhecido, e a própria spec diz que provavelmente estão errados.
/// Mapeá-los seria plantar 1.1 MiB inventado num campo que o 0.3b vai imprimir.
#[test]
fn rom_size_refuses_the_unattested_codes() {
    for code in [0x52, 0x53, 0x54, 0x09, 0xFF] {
        let mut rom = blank_rom();
        rom[ROM_SIZE] = code;
        let size = parse(&rom).rom_size();

        assert_eq!(size.code(), code);
        assert_eq!(size.bytes(), None, "tamanho de ROM ${code:02X}");
        assert_eq!(size.banks(), None, "bancos de ROM ${code:02X}");
    }
}

// ---------------------------------------------------------------------------
// $0149 — tamanho da RAM
// ---------------------------------------------------------------------------

/// A RAM **não** segue fórmula: `$04` são 128 KiB e `$05` são 64 KiB. Qualquer
/// `32 KiB << n` acerta parte da tabela e erra estes dois — que é o jeito mais
/// fácil de ter um teste verde e um MBC1 quebrado no marco M4.
#[test]
fn ram_size_is_a_table_and_it_is_not_monotonic() {
    for (code, bytes) in [
        (0x00, 0),
        (0x02, 8 * 1024),
        (0x03, 32 * 1024),
        (0x04, 128 * 1024),
        (0x05, 64 * 1024),
    ] {
        let mut rom = blank_rom();
        rom[RAM_SIZE] = code;

        assert_eq!(
            parse(&rom).ram_size().bytes(),
            Some(bytes),
            "tamanho de RAM ${code:02X}"
        );
    }
}

/// O erro que a spec pegou: `$01` é largamente documentado por aí como 2 KiB,
/// e o Pan Docs diz "Unused" — chip de 2 KiB nunca foi usado em cartucho e a
/// origem do valor é desconhecida. `None` é a resposta honesta; `Some(2048)`
/// seria um número inventado com cara de medição.
#[test]
fn ram_size_code_one_is_unattested_not_two_kib() {
    let mut rom = blank_rom();
    rom[RAM_SIZE] = 0x01;
    let size = parse(&rom).ram_size();

    assert_eq!(size.code(), 0x01);
    assert_eq!(size.bytes(), None);
}

#[test]
fn ram_size_refuses_codes_outside_the_table() {
    let mut rom = blank_rom();
    rom[RAM_SIZE] = 0x06;

    assert_eq!(parse(&rom).ram_size().bytes(), None);
}

// ---------------------------------------------------------------------------
// $014D — checksum do cabeçalho
// ---------------------------------------------------------------------------

#[test]
fn checksum_follows_the_boot_rom_formula() {
    let mut rom = blank_rom();
    rom[TITLE..TITLE + 6].copy_from_slice(b"TETRIS");
    rom[CARTRIDGE_TYPE] = 0x00;
    rom[ROM_SIZE] = 0x00;
    rom[RAM_SIZE] = 0x00;
    seal(&mut rom);

    let checksum = parse(&rom).checksum();

    assert_eq!(checksum.computed(), boot_rom_checksum(&rom));
    assert_eq!(checksum.stored(), rom[HEADER_CHECKSUM]);
    assert!(checksum.is_valid());
}

#[test]
fn checksum_of_a_sealed_blank_rom_is_valid() {
    let mut rom = blank_rom();
    seal(&mut rom);

    assert!(parse(&rom).checksum().is_valid());
}

/// O boot ROM trava a máquina se `$014D` não bater. Um bit trocado em qualquer
/// byte de `$0134`–`$014C` tem de aparecer.
#[test]
fn checksum_catches_a_single_flipped_bit() {
    let mut rom = blank_rom();
    rom[TITLE..TITLE + 6].copy_from_slice(b"TETRIS");
    seal(&mut rom);

    for address in [0x0134, 0x0140, 0x0147, 0x014C] {
        let mut corrupted = rom.clone();
        corrupted[address] ^= 0x01;
        let checksum = parse(&corrupted).checksum();

        assert!(
            !checksum.is_valid(),
            "bit trocado em ${address:04X} passou despercebido"
        );
        assert_eq!(
            checksum.stored(),
            rom[HEADER_CHECKSUM],
            "o armazenado é o byte de $014D, não o recalculado"
        );
    }
}

/// A outra ponta do intervalo: `$0133` (fim do logo) e `$014D`–`$014F` (o
/// próprio checksum e o global) estão **fora** da soma. Um `..=0x014D` ou um
/// `0x0133..` compilam igual e passam nos testes acima; só este os pega.
#[test]
fn checksum_ignores_bytes_outside_0134_014c() {
    let mut rom = blank_rom();
    rom[TITLE..TITLE + 6].copy_from_slice(b"TETRIS");
    seal(&mut rom);

    for address in [0x0100, 0x0133, 0x014E, 0x014F] {
        let mut touched = rom.clone();
        touched[address] ^= 0xFF;

        assert!(
            parse(&touched).checksum().is_valid(),
            "${address:04X} está fora de $0134–$014C e não deveria entrar na soma"
        );
    }
}
