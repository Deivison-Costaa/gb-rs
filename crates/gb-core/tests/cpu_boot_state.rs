//! ROADMAP 1.2b-i — os registradores da CPU no hand-off da boot ROM.
//!
//! Spec: `docs/reference/01-memory-map.md` § Console state after boot ROM
//! hand-off → *CPU registers* (Pan Docs `fe246067b695`). A tabela tem **cinco
//! colunas de modelo** e este emulador é DMG; a coluna que vale é a segunda:
//!
//! > | Register | DMG0 | **DMG** | MGB | SGB | SGB2 |
//! > | A  | $01 | **$01** | $FF | $01 | $FF |
//! > | F  | Z=0 N=0 H=0 C=0 | **Z=1 N=0 H=? C=?** | Z=1 N=0 H=? C=? | Z=0 N=0 H=0 C=0 | Z=0 N=0 H=0 C=0 |
//! > | B  | $FF | **$00** | $00 | $00 | $00 |
//! > | C  | $13 | **$13** | $13 | $14 | $14 |
//! > | D  | $00 | **$00** | $00 | $00 | $00 |
//! > | E  | $C1 | **$D8** | $D8 | $00 | $00 |
//! > | H  | $84 | **$01** | $01 | $C0 | $C0 |
//! > | L  | $03 | **$4D** | $4D | $60 | $60 |
//! > | PC | $0100 | **$0100** | $0100 | $0100 | $0100 |
//! > | SP | $FFFE | **$FFFE** | $FFFE | $FFFE | $FFFE |
//!
//! e a nota de rodapé que faz do `F` a única entrada que não é literal:
//!
//! > If the header checksum is $00, then the carry and half-carry flags are
//! > clear; otherwise, they are both set.
//!
//! Duas coisas que a tabela não diz e que os testes daqui fixam como decisão:
//! o que são os bits 3–0 de `F` (a coluna descreve quatro flags e mais nada),
//! e **qual** dos dois checksums a nota de rodapé quer — o gravado em `$014D`
//! ou o calculado a partir de `$0134`–`$014C`. Num cartucho que o boot ROM
//! deixaria rodar os dois são iguais; este emulador pula o boot ROM e monta
//! cartucho com checksum errado de propósito, então aqui eles divergem.
//!
//! ROMs sintéticas, montadas byte a byte: `tests/roms/` é gitignored e baixado
//! por script, e teste que lê de lá passa vazio na máquina de quem não rodou o
//! download — o modo de falha da nota 8 do `STATUS.md`.
//!
//! `unwrap`/`expect` são permitidos aqui: a R6 proíbe fora de teste.

use gb_core::cart::CartridgeHeader;
use gb_core::cpu::{Flag, Registers};

/// Menor ROM com o cabeçalho inteiro (`$014F` é o último byte dele).
const MIN_ROM_LEN: usize = 0x0150;
/// `$0134` — primeiro byte que entra no checksum calculado.
const CHECKSUMMED_FIRST: usize = 0x0134;
/// `$0134`–`$014C` são 25 bytes, e o `- 1` por byte da fórmula sai daqui.
const CHECKSUMMED_LEN: u8 = 0x19;
/// `$014D` — o checksum gravado, que o boot ROM confere.
const HEADER_CHECKSUM: usize = 0x014D;

/// Os oito registradores de 8 bits que a tabela dá como literal, na ordem dela.
///
/// `F` fica de fora de propósito: é a única linha que depende da ROM, e
/// misturá-la aqui esconderia justamente o que esta iteração tem de decidir.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Column {
    name: &'static str,
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
}

impl Column {
    fn of(regs: &Registers, name: &'static str) -> Self {
        Self {
            name,
            a: regs.a,
            b: regs.b,
            c: regs.c,
            d: regs.d,
            e: regs.e,
            h: regs.h,
            l: regs.l,
        }
    }

    /// Compara só os valores; o nome é rótulo de mensagem, não dado.
    fn same_values(self, other: Self) -> bool {
        (self.a, self.b, self.c, self.d, self.e, self.h, self.l)
            == (
                other.a, other.b, other.c, other.d, other.e, other.h, other.l,
            )
    }
}

/// A coluna DMG — a que este emulador tem de reproduzir.
const DMG: Column = Column {
    name: "DMG",
    a: 0x01,
    b: 0x00,
    c: 0x13,
    d: 0x00,
    e: 0xD8,
    h: 0x01,
    l: 0x4D,
};

/// As outras quatro colunas da **mesma tabela**, transcritas para servirem de
/// controle negativo.
///
/// Não é zelo excessivo: DMG0 é uma variante rara do próprio Game Boy
/// monocromático e mora na coluna vizinha. Copiar a errada dá um estado inteiro
/// plausível, que boota, e que só destoa dentro de um jogo.
const OTHER_MODELS: [Column; 4] = [
    Column {
        name: "DMG0",
        a: 0x01,
        b: 0xFF,
        c: 0x13,
        d: 0x00,
        e: 0xC1,
        h: 0x84,
        l: 0x03,
    },
    Column {
        name: "MGB",
        a: 0xFF,
        b: 0x00,
        c: 0x13,
        d: 0x00,
        e: 0xD8,
        h: 0x01,
        l: 0x4D,
    },
    Column {
        name: "SGB",
        a: 0x01,
        b: 0x00,
        c: 0x14,
        d: 0x00,
        e: 0x00,
        h: 0xC0,
        l: 0x60,
    },
    Column {
        name: "SGB2",
        a: 0xFF,
        b: 0x00,
        c: 0x14,
        d: 0x00,
        e: 0x00,
        h: 0xC0,
        l: 0x60,
    },
];

/// ROM sintética cujo cabeçalho produz exatamente o par de checksums pedido.
///
/// A fórmula do § 014D é `checksum = checksum - rom[addr] - 1` sobre 25 bytes;
/// com o resto do intervalo zerado, mexer em `$0134` já alcança qualquer alvo.
/// `$014D` está **fora** do intervalo somado, então o gravado se escolhe à
/// parte — que é o ponto de metade dos testes daqui.
///
/// O fixture se confere antes de devolver: helper que monta o cenário errado em
/// silêncio dá veredito de outro experimento (`STATUS.md`, nota 14).
fn rom_with_checksums(stored: u8, computed: u8) -> Vec<u8> {
    let mut rom = vec![0u8; MIN_ROM_LEN];
    rom[CHECKSUMMED_FIRST] = 0u8.wrapping_sub(computed).wrapping_sub(CHECKSUMMED_LEN);
    rom[HEADER_CHECKSUM] = stored;

    let checksum = CartridgeHeader::parse(&rom)
        .expect("ROM do fixture tem o cabeçalho inteiro")
        .checksum();
    assert_eq!(checksum.stored(), stored, "fixture: $014D saiu errado");
    assert_eq!(
        checksum.computed(),
        computed,
        "fixture: o checksum calculado saiu errado"
    );

    rom
}

/// Os registradores no hand-off, para um cartucho com o par de checksums dado.
fn after_boot(stored: u8, computed: u8) -> Registers {
    let rom = rom_with_checksums(stored, computed);
    let header = CartridgeHeader::parse(&rom).expect("ROM do fixture tem o cabeçalho inteiro");

    Registers::after_boot_rom(header.checksum())
}

/// Cartucho íntegro: os dois checksums batem, como num jogo de verdade.
fn after_boot_valid(checksum: u8) -> Registers {
    after_boot(checksum, checksum)
}

// ---------------------------------------------------------------------------
// Os literais da coluna DMG
// ---------------------------------------------------------------------------

#[test]
fn the_eight_bit_registers_come_from_the_dmg_column() {
    let regs = after_boot_valid(0x42);

    assert_eq!(regs.a, 0x01, "A");
    assert_eq!(regs.b, 0x00, "B");
    assert_eq!(regs.c, 0x13, "C");
    assert_eq!(regs.d, 0x00, "D");
    assert_eq!(regs.e, 0xD8, "E");
    assert_eq!(regs.h, 0x01, "H");
    assert_eq!(regs.l, 0x4D, "L");
}

#[test]
fn the_pairs_compose_the_values_the_folklore_quotes() {
    // BC=$0013, DE=$00D8, HL=$014D são os números que todo mundo repete. Eles
    // valem como conferência cruzada da montagem dos pares (1.1) contra a
    // tabela: se `bc()` trocasse as metades, sairia $1300 aqui.
    let regs = after_boot_valid(0x42);

    assert_eq!(regs.bc(), 0x0013, "BC");
    assert_eq!(regs.de(), 0x00D8, "DE");
    assert_eq!(regs.hl(), 0x014D, "HL");
}

#[test]
fn sp_and_pc_hand_off_at_the_cartridge_entry_point() {
    let regs = after_boot_valid(0x42);

    assert_eq!(
        regs.pc, 0x0100,
        "PC — é aqui que a boot ROM entrega o controle"
    );
    assert_eq!(
        regs.sp, 0xFFFE,
        "SP — topo da HRAM, e a pilha cresce para baixo"
    );
}

#[test]
fn this_is_the_dmg_column_and_not_one_of_the_other_four() {
    // Controle negativo da leitura da tabela. `assert_eq!(regs.a, 0x01)` não
    // distingue DMG de SGB, e nenhum teste de valor isolado distingue DMG de
    // DMG0 — é o conjunto que identifica a coluna.
    let regs = after_boot_valid(0x42);
    let actual = Column::of(&regs, "implementado");

    assert!(
        actual.same_values(DMG),
        "os registradores não são a coluna DMG: {actual:?}"
    );

    for other in OTHER_MODELS {
        assert!(
            !actual.same_values(other),
            "os registradores saíram iguais à coluna {}, que é outro console",
            other.name
        );
    }
}

// ---------------------------------------------------------------------------
// `F` — a única linha da tabela que depende da ROM
// ---------------------------------------------------------------------------

#[test]
fn z_is_set_and_n_is_clear_whatever_the_checksum() {
    // A nota de rodapé fala **só** de carry e half-carry. Z e N são literais da
    // coluna e não podem passar a depender da ROM junto com os outros dois.
    for checksum in [0x00, 0x01, 0x7F, 0x80, 0xFF] {
        let regs = after_boot_valid(checksum);

        assert!(regs.flag(Flag::Z), "Z com checksum ${checksum:02X}");
        assert!(!regs.flag(Flag::N), "N com checksum ${checksum:02X}");
    }
}

#[test]
fn half_carry_and_carry_are_clear_when_the_header_checksum_is_zero() {
    let regs = after_boot_valid(0x00);

    assert!(!regs.flag(Flag::H), "H com checksum $00");
    assert!(!regs.flag(Flag::C), "C com checksum $00");
}

#[test]
fn half_carry_and_carry_are_set_for_every_other_header_checksum() {
    // "otherwise, they are both set" — os 255 outros valores, não só alguns.
    // Uma implementação que testasse `!= 0xFF`, ou que olhasse um bit em vez do
    // byte inteiro, passa numa amostra e falha aqui.
    for checksum in 0x01..=0xFF {
        let regs = after_boot_valid(checksum);

        assert!(regs.flag(Flag::H), "H com checksum ${checksum:02X}");
        assert!(regs.flag(Flag::C), "C com checksum ${checksum:02X}");
    }
}

#[test]
fn the_flags_follow_the_stored_byte_and_not_the_computed_one() {
    // **A decisão desta iteração.** A nota de rodapé diz "if the header
    // checksum is $00" e liga a palavra à § 014D — Header checksum, cuja
    // primeira frase é "This byte contains an 8-bit checksum": o sujeito é o
    // byte gravado em `$014D`. O valor calculado a partir de `$0134`–`$014C` é
    // a variável `checksum` do trecho em C daquela seção, não "the header
    // checksum".
    //
    // Num cartucho íntegro os dois coincidem e a escolha é invisível. Ela só
    // aparece em ROM corrompida — que este emulador monta de propósito, porque
    // `cart::load` não julga o cabeçalho e a ROM que trava o boot ROM é
    // justamente a que alguém quer diagnosticar.
    let stored_zero = after_boot(0x00, 0xE7);
    assert!(
        !stored_zero.flag(Flag::H) && !stored_zero.flag(Flag::C),
        "com $014D = $00, H e C são limpos mesmo que o calculado não seja zero"
    );

    let computed_zero = after_boot(0x5A, 0x00);
    assert!(
        computed_zero.flag(Flag::H) && computed_zero.flag(Flag::C),
        "com $014D = $5A, H e C são setados mesmo que o calculado seja zero"
    );
}

#[test]
fn f_is_b0_on_a_valid_cartridge_and_80_on_the_zero_checksum() {
    // Os dois únicos valores possíveis de `F`, escritos por extenso. $B0 é o
    // número que o folclore repete (`AF = $01B0`) — e ele só vale para a ROM
    // cujo `$014D` não é zero, que é o caso comum e por isso o que a memória
    // guardou como se fosse constante.
    assert_eq!(after_boot_valid(0x42).f, 0b1011_0000, "Z=1 N=0 H=1 C=1");
    assert_eq!(after_boot_valid(0x00).f, 0b1000_0000, "Z=1 N=0 H=0 C=0");

    assert_eq!(after_boot_valid(0x42).af(), 0x01B0);
    assert_eq!(after_boot_valid(0x00).af(), 0x0180);
}

#[test]
fn the_low_nibble_of_f_has_no_value_in_the_spec_and_comes_out_zero() {
    // Decisão registrada, na linha da 0009. A coluna descreve `F` por quatro
    // flags e nada mais; sobre os bits 3–0 o Pan Docs do commit fixado não tem
    // uma frase. Zero aqui não é máscara aplicada — é o que sobra ao montar `F`
    // a partir dos únicos quatro bits que a tabela nomeia.
    //
    // O teste existe para que isso continue sendo consequência de uma decisão e
    // não vire um `& 0xF0` escondido em algum lugar do caminho.
    for checksum in 0x00..=0xFF {
        let regs = after_boot_valid(checksum);

        assert_eq!(
            regs.f & 0b0000_1111,
            0,
            "nibble baixo de F com checksum ${checksum:02X}"
        );
    }
}

// ---------------------------------------------------------------------------
// A fronteira com o 1.1
// ---------------------------------------------------------------------------

#[test]
fn the_boot_state_is_a_constructor_and_not_the_default() {
    // `Registers::default()` continua sendo tudo zero: ausência de decisão. O
    // estado pós-boot é uma decisão, depende da ROM e por isso não cabe num
    // `Default`. Se alguém fundir os dois, este teste avisa.
    let regs = after_boot_valid(0x42);

    assert_ne!(regs, Registers::default());
    assert_eq!(
        Registers::default().pc,
        0x0000,
        "o default não pula a boot ROM"
    );
}
