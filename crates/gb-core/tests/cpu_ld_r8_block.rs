//! ROADMAP 1.4a — o bloco `LD r8,r8`: `$40`–`$7F` **sem** `$76`.
//!
//! Spec: `docs/reference/03-opcodes.md`, linhas `40`–`7F` (gbops
//! `90b9bf296aed`), e `docs/reference/02-cpu.md` § Block 1: 8-bit
//! register-to-register loads (Pan Docs `fe246067b695`).
//!
//! # Uma regra, três formas de M-cycle
//!
//! A § Block 1 dá a codificação inteira do bloco em quatro linhas:
//!
//! ```text
//! Bits | Campo
//!    7 | 0
//!    6 | 1
//!  5-3 | Dest (r8)
//!  2-0 | Source (r8)
//! ```
//!
//! e o `r8` é `b c d e h l [hl] a` nos índices 0 a 7. Sessenta e quatro
//! combinações, uma regra só — e é por isso que este item não testa opcode por
//! opcode à mão: os testes varrem as 7×7 combinações de registrador e as duas
//! famílias que tocam a memória, gerando o opcode pela mesma fórmula que a spec
//! dá. Transcrever 63 linhas seria transcrever a fórmula 63 vezes.
//!
//! O que **não** é uniforme é o timing, e a tabela de gbops o separa em três:
//!
//! | Forma | T-cycles | M-cycles (passo a passo) |
//! |---|---|---|
//! | `LD r,r'` | 4 | `fetch` |
//! | `LD r,(HL)` | 8 | `fetch → read((HL)->r)` |
//! | `LD (HL),r` | 8 | `fetch → write(r->(HL))` |
//!
//! As duas últimas são **1 byte** e **2 M-cycles**: o segundo M-cycle não busca
//! operando nenhum, ele é o acesso à memória apontada por `HL`. Um `PC` que
//! ande dois bytes aqui leu o opcode seguinte como se fosse dado.
//!
//! # A exceção, que é o teste mais importante do arquivo
//!
//! `$76` seria `LD (HL),(HL)` pela fórmula, e não é: a § Block 1 o marca como
//! **exceção** e o manda para `HALT`, que é o ROADMAP 2.3. Um decodificador
//! escrito direto dos bits acerta 63 opcodes e transforma o 64º numa leitura e
//! numa escrita no mesmo endereço — plausível, silenciosa, e capaz de fazer
//! qualquer ROM que use `HALT` girar para sempre sem travar.
//!
//! `unwrap`/`expect` são permitidos aqui: a R6 proíbe fora de teste.

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::{Cpu, Lockup};

/// `$0100` — o endereço em que a boot ROM entrega o controle ao cartucho.
const ENTRY: usize = 0x0100;

/// Um endereço de WRAM, para os testes que precisam de memória que aceite
/// escrita. `HL` no hand-off vale `$014D`, que é **ROM**: um `LD (HL),r`
/// apontado para lá é engolido pelo cartucho, e o teste passaria a medir o
/// `NoMbc` em vez do opcode.
///
/// Os **dois** bytes são diferentes de zero de propósito. Com `$C000` — o
/// começo redondo da WRAM, que é o endereço que se escreve sem pensar — o
/// byte baixo é `$00`, e a WRAM também começa zerada: `storing_l_to_hl_…`
/// passava sem que nada tivesse sido escrito. É a nota 8 do `STATUS.md` na
/// forma mais barata de cair, e só apareceu porque o teste foi rodado contra
/// a implementação que ainda não decodificava o bloco.
const SCRATCH: u16 = 0xC0A7;

/// Os oito valores de `r8`, na ordem dos índices 0 a 7 que a § Block 1 lista.
///
/// O índice 6 é `[hl]` e não é registrador: ele é a memória apontada por `HL`,
/// e é o que separa as três formas de M-cycle da tabela.
const R8: [Operand; 8] = [
    Operand::Register("B"),
    Operand::Register("C"),
    Operand::Register("D"),
    Operand::Register("E"),
    Operand::Register("H"),
    Operand::Register("L"),
    Operand::Memory,
    Operand::Register("A"),
];

/// Índice de `[hl]` dentro de `r8`. É o valor que, repetido nos dois campos,
/// dá `$76` — o `HALT` da exceção.
const HL_INDEX: u8 = 6;

/// Um operando `r8`: ou um dos sete registradores, ou a memória em `HL`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Operand {
    Register(&'static str),
    Memory,
}

/// O opcode de `LD dest,source`, montado pela fórmula da § Block 1:
/// `0b01_ddd_sss`.
const fn ld(dest: u8, source: u8) -> u8 {
    0b0100_0000 | (dest << 3) | source
}

/// Uma ROM de 32 KiB com `program` em `$0100`. O resto é `$00` (`NOP`), para
/// que uma CPU que escape do programa continue andando em vez de travar por
/// acaso — travar por acaso parece exatamente o que os testes de `$76`
/// procuram.
fn rom_with(program: &[u8]) -> Vec<u8> {
    let mut rom = vec![0x00; NoMbc::MAX_ROM_LEN];
    rom[ENTRY..ENTRY + program.len()].copy_from_slice(program);
    rom
}

/// Uma CPU no hand-off da boot ROM e um barramento com `program` em `$0100`.
fn machine(program: &[u8]) -> (Cpu, Bus) {
    let rom = rom_with(program);
    let checksum = CartridgeHeader::parse(&rom)
        .expect("32 KiB de ROM têm cabeçalho de sobra")
        .checksum();
    let cartridge = NoMbc::new(rom).expect("32 KiB cabem num cartucho sem MBC");

    (Cpu::after_boot_rom(checksum), Bus::new(Box::new(cartridge)))
}

/// Sentinelas distintas para os sete registradores, para que copiar o errado
/// nunca produza o valor certo por acaso. `HL` fica valendo [`SCRATCH`] porque
/// metade dos testes deste arquivo o usa como endereço.
fn seed_registers(cpu: &mut Cpu) {
    cpu.registers.b = 0xB1;
    cpu.registers.c = 0xC2;
    cpu.registers.d = 0xD3;
    cpu.registers.e = 0xE4;
    cpu.registers.a = 0xA5;
    cpu.registers.set_hl(SCRATCH);
}

/// Lê um dos sete registradores pelo nome que a tabela `r8` lhe dá.
fn read_register(cpu: &Cpu, name: &str) -> u8 {
    match name {
        "B" => cpu.registers.b,
        "C" => cpu.registers.c,
        "D" => cpu.registers.d,
        "E" => cpu.registers.e,
        "H" => cpu.registers.h,
        "L" => cpu.registers.l,
        "A" => cpu.registers.a,
        other => unreachable!("{other} não é um registrador da lista r8"),
    }
}

// ---------------------------------------------------------------------------
// `LD r,r'` — 49 opcodes, 1 M-cycle
// ---------------------------------------------------------------------------

#[test]
fn every_register_to_register_load_copies_source_into_destination() {
    // As 7×7 combinações em que nenhum dos dois operandos é `[hl]`. É a matriz
    // inteira menos a linha e a coluna do índice 6 — 49 dos 63 opcodes.
    for (dest_index, dest) in R8.iter().enumerate() {
        for (source_index, source) in R8.iter().enumerate() {
            let (Operand::Register(dest), Operand::Register(source)) = (dest, source) else {
                continue;
            };
            #[expect(clippy::cast_possible_truncation, reason = "índices de 0 a 7")]
            let opcode = ld(dest_index as u8, source_index as u8);

            let (mut cpu, mut bus) = machine(&[opcode]);
            seed_registers(&mut cpu);
            let expected = read_register(&cpu, source);

            cpu.step(&mut bus);

            assert_eq!(
                read_register(&cpu, dest),
                expected,
                "${opcode:02X} é `LD {dest},{source}`"
            );
        }
    }
}

#[test]
fn a_register_to_register_load_is_one_m_cycle_and_one_byte() {
    // A coluna de gbops é `fetch`, e só: 4 T-cycles, 1 byte. O M-cycle da
    // instrução **é** o fetch, então uma chamada de `step` a completa.
    for (dest_index, dest) in R8.iter().enumerate() {
        for (source_index, source) in R8.iter().enumerate() {
            let (Operand::Register(dest), Operand::Register(source)) = (dest, source) else {
                continue;
            };
            #[expect(clippy::cast_possible_truncation, reason = "índices de 0 a 7")]
            let opcode = ld(dest_index as u8, source_index as u8);

            let (mut cpu, mut bus) = machine(&[opcode]);
            seed_registers(&mut cpu);

            cpu.step(&mut bus);

            assert!(
                cpu.is_between_instructions(),
                "${opcode:02X} (`LD {dest},{source}`) tem 4 T-cycles: um \
                 M-cycle, que é o próprio fetch"
            );
            assert_eq!(
                cpu.registers.pc, 0x0101,
                "${opcode:02X} (`LD {dest},{source}`) tem 1 byte"
            );
        }
    }
}

#[test]
fn a_register_to_register_load_touches_nothing_but_the_two_registers() {
    // `LD D,B` (`$50`). Um `assert_eq!` na struct inteira pega o que uma lista
    // de asserções por registrador não pega: o decodificador que acerta o
    // destino e escreve em mais alguém de lambuja.
    let (mut cpu, mut bus) = machine(&[0x50]);
    seed_registers(&mut cpu);

    let mut expected = cpu.registers;
    expected.d = expected.b;
    expected.pc = 0x0101;

    cpu.step(&mut bus);

    assert_eq!(
        cpu.registers, expected,
        "`LD D,B` tem `-` nas quatro colunas de flag e mexe em um registrador só"
    );
}

// ---------------------------------------------------------------------------
// `LD r,(HL)` — 7 opcodes, 2 M-cycles
// ---------------------------------------------------------------------------

#[test]
fn every_load_from_hl_reads_the_byte_hl_points_at() {
    for (dest_index, dest) in R8.iter().enumerate() {
        let Operand::Register(dest) = dest else {
            continue;
        };
        #[expect(clippy::cast_possible_truncation, reason = "índices de 0 a 7")]
        let opcode = ld(dest_index as u8, HL_INDEX);

        let (mut cpu, mut bus) = machine(&[opcode]);
        seed_registers(&mut cpu);
        bus.write(SCRATCH, 0x5A);

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(
            read_register(&cpu, dest),
            0x5A,
            "${opcode:02X} é `LD {dest},(HL)`"
        );
    }
}

#[test]
fn loading_from_hl_takes_two_m_cycles_and_the_register_changes_on_the_second() {
    // `LD B,(HL)` (`$46`): `fetch → read((HL)->B)`. O destino recebe o byte no
    // **segundo** M-cycle, junto com a leitura — não antes (seria 1 M-cycle) e
    // não num terceiro `internal` (que a coluna não tem, e os 8 T-cycles
    // desmentem). Observar o meio da instrução é a R2; ver o
    // `cpu_mcycle_loop.rs`, onde `JP u16` faz o mesmo papel.
    let (mut cpu, mut bus) = machine(&[0x46]);
    seed_registers(&mut cpu);
    bus.write(SCRATCH, 0x5A);
    let before = cpu.registers.b;

    cpu.step(&mut bus);
    assert_eq!(
        cpu.registers.b, before,
        "no M1 só o opcode chegou; o barramento ainda não visitou (HL)"
    );
    assert!(
        !cpu.is_between_instructions(),
        "`LD B,(HL)` tem 8 T-cycles: uma implementação que já terminou no \
         fetch é instruction-stepped, e a R2 proíbe"
    );

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.b, 0x5A, "o M2 é o `read((HL)->B)`");
    assert!(
        cpu.is_between_instructions(),
        "dois M-cycles e acabou: não há terceiro"
    );
}

#[test]
fn loading_from_hl_is_one_byte_and_does_not_eat_the_next_opcode() {
    // `LD B,(HL)` seguido de `$D3`, que é opcode inexistente. Se o M2 buscasse
    // um operando no fluxo de instruções em vez de ler `(HL)`, o `PC` andaria
    // dois e o `$D3` viraria dado — o `assert_eq!` no `PC` sozinho pegaria
    // isso, mas o lockup diz *qual* byte foi consumido a mais.
    let (mut cpu, mut bus) = machine(&[0x46, 0xD3]);
    seed_registers(&mut cpu);
    bus.write(SCRATCH, 0x5A);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        cpu.registers.pc, 0x0101,
        "`LD B,(HL)` tem 1 byte: o operando é `HL`, que já está na CPU"
    );

    cpu.step(&mut bus);
    assert_eq!(
        cpu.lockup(),
        Some(Lockup::IllegalOpcode(0xD3)),
        "o byte seguinte é opcode, não dado: quem o consumiu como operando \
         não trava aqui"
    );
}

#[test]
fn loading_h_from_hl_uses_the_address_hl_had_before_the_write() {
    // `LD H,(HL)` (`$66`) muda o byte alto de `HL` com o valor lido através do
    // próprio `HL`. O endereço tem de ser o de antes.
    let (mut cpu, mut bus) = machine(&[0x66]);
    seed_registers(&mut cpu);
    bus.write(SCRATCH, 0x5A);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(cpu.registers.h, 0x5A, "`LD H,(HL)` escreve em `H`");
    assert_eq!(
        cpu.registers.l,
        (SCRATCH & 0xFF) as u8,
        "e não encosta em `L`: o `HL` que sobra é o novo `H` com o `L` antigo"
    );
}

// ---------------------------------------------------------------------------
// `LD (HL),r` — 7 opcodes, 2 M-cycles
// ---------------------------------------------------------------------------

#[test]
fn every_store_to_hl_writes_the_register_where_hl_points() {
    for (source_index, source) in R8.iter().enumerate() {
        let Operand::Register(source) = source else {
            continue;
        };
        #[expect(clippy::cast_possible_truncation, reason = "índices de 0 a 7")]
        let opcode = ld(HL_INDEX, source_index as u8);

        let (mut cpu, mut bus) = machine(&[opcode]);
        seed_registers(&mut cpu);
        let expected = read_register(&cpu, source);

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(
            bus.read(SCRATCH),
            expected,
            "${opcode:02X} é `LD (HL),{source}`"
        );
    }
}

#[test]
fn storing_to_hl_takes_two_m_cycles_and_memory_changes_on_the_second() {
    // `LD (HL),B` (`$70`): `fetch → write(B->(HL))`. A escrita é do M2.
    let (mut cpu, mut bus) = machine(&[0x70]);
    seed_registers(&mut cpu);
    bus.write(SCRATCH, 0x00);

    cpu.step(&mut bus);
    assert_eq!(
        bus.read(SCRATCH),
        0x00,
        "no M1 só o opcode chegou; nada foi escrito ainda"
    );
    assert!(
        !cpu.is_between_instructions(),
        "`LD (HL),B` tem 8 T-cycles: dois M-cycles"
    );

    cpu.step(&mut bus);
    assert_eq!(bus.read(SCRATCH), 0xB1, "o M2 é o `write(B->(HL))`");
    assert!(
        cpu.is_between_instructions(),
        "dois M-cycles e acabou: não há terceiro"
    );
}

#[test]
fn storing_l_to_hl_writes_the_low_byte_of_the_address_it_wrote_to() {
    // `LD (HL),L` (`$75`) é o caso em que fonte e endereço se sobrepõem. Nada
    // aqui muda `HL`, então o valor escrito é o `L` de sempre — o teste existe
    // para fixar que não há ordem de operações a inventar.
    let (mut cpu, mut bus) = machine(&[0x75]);
    seed_registers(&mut cpu);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        bus.read(SCRATCH),
        (SCRATCH & 0xFF) as u8,
        "`LD (HL),L` grava `L` em `(HL)`"
    );
    assert_eq!(
        cpu.registers.hl(),
        SCRATCH,
        "e `HL` não se move: quem mexe em `HL` é o 1.4c, não este bloco"
    );
}

#[test]
fn storing_to_hl_is_one_byte_and_does_not_eat_the_next_opcode() {
    let (mut cpu, mut bus) = machine(&[0x70, 0xD3]);
    seed_registers(&mut cpu);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        cpu.registers.pc, 0x0101,
        "`LD (HL),B` tem 1 byte: o endereço é `HL`, que já está na CPU"
    );

    cpu.step(&mut bus);
    assert_eq!(
        cpu.lockup(),
        Some(Lockup::IllegalOpcode(0xD3)),
        "o byte seguinte é opcode, não dado"
    );
}

// ---------------------------------------------------------------------------
// A exceção: `$76` não é `LD (HL),(HL)`
// ---------------------------------------------------------------------------

#[test]
fn opcode_76_is_halt_and_not_a_load_from_hl_into_hl() {
    // A § Block 1 do `02-cpu.md`:
    //
    // > **Exception**: trying to encode `ld [hl], [hl]` instead yields the
    // > `halt` instruction
    //
    // `HALT` é o ROADMAP 2.3 e ainda não existe, então o veredito correto hoje
    // é `UndecodedOpcode` — o rótulo que diz "falta implementar", e não
    // `IllegalOpcode`, que diria que a ROM executou lixo. Um decodificador
    // escrito direto dos bits acerta os outros 63 e transforma este numa
    // leitura seguida de uma escrita no mesmo endereço: sem efeito visível,
    // sem travar, e uma ROM que use `HALT` gira para sempre.
    assert_eq!(
        ld(HL_INDEX, HL_INDEX),
        0x76,
        "a fórmula `01 ddd sss` com os dois campos em `[hl]` dá $76"
    );

    let (mut cpu, mut bus) = machine(&[0x76]);
    seed_registers(&mut cpu);
    bus.write(SCRATCH, 0x5A);

    cpu.step(&mut bus);

    assert_eq!(
        cpu.lockup(),
        Some(Lockup::UndecodedOpcode(0x76)),
        "$76 é `HALT`, que é o 2.3 — não é um load deste bloco"
    );
}

// ---------------------------------------------------------------------------
// Controle negativo: exatamente $40–$7F, menos $76
// ---------------------------------------------------------------------------

#[test]
fn the_block_this_item_decodes_is_exactly_40_to_7f_without_76() {
    // Nota 25 do `STATUS.md`: teste que afirma *pertinência* ("estes N são
    // loads") não pega *excesso* ("e mais um"). Onde a spec dá um bloco
    // fechado, o teste varre o complemento — os 256 opcodes, afirmando dos
    // dois lados. Sem isto, decodificar `$76` junto, ou passar do `$7F`,
    // continuaria verde.
    const ILLEGAL: [u8; 11] = [
        0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
    ];

    for opcode in 0x00..=0xFFu8 {
        let (mut cpu, mut bus) = machine(&[opcode, 0x00, 0x00]);
        seed_registers(&mut cpu);

        // Dois M-cycles: o bastante para as formas mais longas deste bloco
        // acabarem, e de menos para `JP u16`, que só desvia no M4.
        cpu.step(&mut bus);
        cpu.step(&mut bus);

        let in_block = (0x40..=0x7F).contains(&opcode) && opcode != 0x76;

        if in_block {
            assert_eq!(
                cpu.lockup(),
                None,
                "${opcode:02X} está no bloco `01 ddd sss` e o 1.4a o decodifica"
            );
        } else if ILLEGAL.contains(&opcode) {
            assert_eq!(
                cpu.lockup(),
                Some(Lockup::IllegalOpcode(opcode)),
                "${opcode:02X} continua sendo um dos onze que não existem"
            );
        } else if opcode != 0x00 && opcode != 0xC3 {
            // `$00` (`NOP`) e `$C3` (`JP u16`) saíram no 1.3.
            assert_eq!(
                cpu.lockup(),
                Some(Lockup::UndecodedOpcode(opcode)),
                "${opcode:02X} é opcode legítimo fora deste sub-item: o \
                 rótulo tem de continuar dizendo `falta implementar`"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Flags: as quatro colunas são `-` nas 63 linhas
// ---------------------------------------------------------------------------

#[test]
fn no_load_in_the_block_touches_the_flags() {
    // Um `F` com os quatro flags ligados e o nibble baixo sujo. O nibble baixo
    // está aqui de propósito: o 1.1 decidiu **não** mascará-lo (ver a
    // invariante no `STATUS.md`), e um `LD` que o limpasse de passagem seria a
    // máscara entrando pela porta dos fundos.
    const DIRTY_F: u8 = 0b1111_1010;

    for opcode in 0x40..=0x7Fu8 {
        if opcode == 0x76 {
            continue;
        }

        let (mut cpu, mut bus) = machine(&[opcode]);
        seed_registers(&mut cpu);
        cpu.registers.f = DIRTY_F;

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(
            cpu.registers.f, DIRTY_F,
            "${opcode:02X} tem `-` nas quatro colunas de flag da tabela"
        );
    }
}
