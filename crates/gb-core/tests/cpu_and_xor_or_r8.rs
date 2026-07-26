//! ROADMAP 1.6c — `AND a,r8`, `XOR a,r8` e `OR a,r8`: os blocos `10 100 rrr`,
//! `10 101 rrr` e `10 110 rrr`.
//!
//! A diferença de natureza em relação ao 1.6a/1.6b: aqui `H` e `C` são
//! **constantes na coluna**, não resultado de conta nenhuma. `AND` tem `H=1`,
//! `C=0`; `XOR` e `OR` têm `H=0`, `C=0`. Nenhuma das duas colunas é
//! carry/empréstimo de verdade — um half-carry "calculado genericamente" a
//! partir da soma ou da subtração erra as três.

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::{Cpu, Flag, Lockup, Registers};

const ENTRY: usize = 0x0100;
const ENTRY_ADDRESS: u16 = 0x0100;

const OPERAND_SLOT: u16 = 0xC7A0;

const SEED_A: u8 = 0x21;
const SEED: [u8; 6] = [0x31, 0x42, 0x53, 0x64, 0xC7, 0xA0];

const R8_NAMES: [&str; 8] = ["B", "C", "D", "E", "H", "L", "(HL)", "A"];

const fn and_a(r8: u8) -> u8 {
    0b1010_0000 | r8
}

const fn xor_a(r8: u8) -> u8 {
    0b1010_1000 | r8
}

const fn or_a(r8: u8) -> u8 {
    0b1011_0000 | r8
}

fn machine(program: &[u8]) -> (Cpu, Bus) {
    let mut rom = vec![0x00; NoMbc::MAX_ROM_LEN];
    rom[ENTRY..ENTRY + program.len()].copy_from_slice(program);

    let checksum = CartridgeHeader::parse(&rom)
        .expect("32 KiB de ROM têm cabeçalho de sobra")
        .checksum();
    let cartridge = NoMbc::new(rom).expect("32 KiB cabem num cartucho sem MBC");

    (Cpu::after_boot_rom(checksum), Bus::new(Box::new(cartridge)))
}

fn seed(registers: &mut Registers) {
    registers.a = SEED_A;
    registers.b = SEED[0];
    registers.c = SEED[1];
    registers.d = SEED[2];
    registers.e = SEED[3];
    registers.h = SEED[4];
    registers.l = SEED[5];
}

fn operand_of(registers: &Registers, r8: u8, from_memory: u8) -> u8 {
    match r8 {
        0 => registers.b,
        1 => registers.c,
        2 => registers.d,
        3 => registers.e,
        4 => registers.h,
        5 => registers.l,
        6 => from_memory,
        _ => registers.a,
    }
}

fn m_cycles_of(opcode: u8) -> usize {
    if opcode & 0b111 == 6 { 2 } else { 1 }
}

#[test]
fn the_block_2_layout_gives_the_twenty_four_opcodes_gbops_lists() {
    let opcodes: Vec<u8> = (0..8)
        .map(and_a)
        .chain((0..8).map(xor_a))
        .chain((0..8).map(or_a))
        .collect();

    assert_eq!(
        opcodes,
        (0xA0..=0xB7u8).collect::<Vec<u8>>(),
        "§ Block 2 do `02-cpu.md`: `10 ooo rrr`. `100` é `AND` ($A0), `101` é \
         `XOR` ($A8) e `110` é `OR` ($B0) — os três últimos ainda não \
         decodificados do bloco. `111` (`CP`, $B8) já saiu no 1.6b"
    );
}

#[test]
fn and_computes_the_bitwise_and_of_a_and_the_operand() {
    for r8 in 0..8u8 {
        let opcode = and_a(r8);
        let (mut cpu, mut bus) = machine(&[opcode]);
        seed(&mut cpu.registers);
        bus.write(OPERAND_SLOT, 0x0C);

        let expected = SEED_A & operand_of(&cpu.registers, r8, 0x0C);

        for _ in 0..m_cycles_of(opcode) {
            cpu.step(&mut bus);
        }

        assert_eq!(
            cpu.registers.a, expected,
            "${opcode:02X} é `AND A,{}`: `E` do § Block 2 é `a &= r8`",
            R8_NAMES[r8 as usize]
        );
    }
}

#[test]
fn xor_computes_the_bitwise_xor_of_a_and_the_operand() {
    for r8 in 0..8u8 {
        let opcode = xor_a(r8);
        let (mut cpu, mut bus) = machine(&[opcode]);
        seed(&mut cpu.registers);
        bus.write(OPERAND_SLOT, 0x0C);

        let expected = SEED_A ^ operand_of(&cpu.registers, r8, 0x0C);

        for _ in 0..m_cycles_of(opcode) {
            cpu.step(&mut bus);
        }

        assert_eq!(
            cpu.registers.a, expected,
            "${opcode:02X} é `XOR A,{}`: `$AF` (`XOR A,A`) sempre zera — o \
             registrador contra si mesmo",
            R8_NAMES[r8 as usize]
        );
    }
}

#[test]
fn or_computes_the_bitwise_or_of_a_and_the_operand() {
    for r8 in 0..8u8 {
        let opcode = or_a(r8);
        let (mut cpu, mut bus) = machine(&[opcode]);
        seed(&mut cpu.registers);
        bus.write(OPERAND_SLOT, 0x0C);

        let expected = SEED_A | operand_of(&cpu.registers, r8, 0x0C);

        for _ in 0..m_cycles_of(opcode) {
            cpu.step(&mut bus);
        }

        assert_eq!(
            cpu.registers.a, expected,
            "${opcode:02X} é `OR A,{}`",
            R8_NAMES[r8 as usize]
        );
    }
}

#[test]
fn and_sets_h_and_clears_c_no_matter_the_operands() {
    // A=$00, B=$00: a soma de nibbles é $00+$00=$00 (sem estouro) e o
    // empréstimo $00<$00 é falso — um H "genérico" copiado do 1.6a/1.6b daria
    // 0 aqui. A coluna do `AND` é `1` literal, então o resultado certo é 1.
    let (mut cpu, mut bus) = machine(&[and_a(0)]);
    cpu.registers.a = 0x00;
    cpu.registers.b = 0x00;
    cpu.registers.f = 0x00;

    cpu.step(&mut bus);

    assert!(
        cpu.registers.flag(Flag::H),
        "`AND A,B` com A=$00 e B=$00: `H` é `1` literal na coluna, não \
         resultado de carry de nibble nenhum — que aqui daria 0"
    );
    assert!(
        !cpu.registers.flag(Flag::C),
        "`AND A,B`: `C` é `0` literal na coluna"
    );
}

#[test]
fn xor_and_or_clear_h_and_c_no_matter_the_operands() {
    // A=$00, B=$0F: o empréstimo de nibble ($00 & $F) < ($0F & $F) é
    // verdadeiro — um H "genérico" copiado do SUB/SBC (1.6b) daria 1 aqui. A
    // coluna do `XOR`/`OR` é `0` literal, então o resultado certo é 0.
    for opcode_of in [xor_a, or_a] {
        let (mut cpu, mut bus) = machine(&[opcode_of(0)]);
        cpu.registers.a = 0x00;
        cpu.registers.b = 0x0F;
        cpu.registers.f = 0xFF;

        cpu.step(&mut bus);

        assert!(
            !cpu.registers.flag(Flag::H),
            "${:02X}: `H` é `0` literal na coluna, não empréstimo de nibble \
             nenhum — que aqui daria 1",
            opcode_of(0)
        );
        assert!(
            !cpu.registers.flag(Flag::C),
            "${:02X}: `C` é `0` literal na coluna",
            opcode_of(0)
        );
    }
}

type BitwiseOp = fn(u8, u8) -> u8;

#[test]
fn every_bitwise_op_matches_its_flag_definition_and_does_not_mask_the_low_nibble() {
    const CASES: [(u8, BitwiseOp, bool); 3] = [
        (0xA0, |a, b| a & b, true),
        (0xA8, |a, b| a ^ b, false),
        (0xB0, |a, b| a | b, false),
    ];

    for (base, combine, half) in CASES {
        let (mut cpu, mut bus) = machine(&[base]);

        for a in 0..=0xFFu8 {
            for operand in 0..=0xFFu8 {
                cpu.registers.f = 0xFF;
                cpu.registers.pc = ENTRY_ADDRESS;
                cpu.registers.a = a;
                cpu.registers.b = operand;

                cpu.step(&mut bus);

                let result = combine(a, operand);
                let expected = u8::from(result == 0) << 7 | u8::from(half) << 5 | 0x0F;

                assert_eq!(
                    cpu.registers.a, result,
                    "${base:02X}: A={a:#04X}, B={operand:#04X}"
                );
                assert_eq!(
                    cpu.registers.f, expected,
                    "${base:02X} com A={a:#04X}, B={operand:#04X}: varredura dos \
                     65536 pares. `N` e `C` são `0` literal e entraram `1`; `H` \
                     é a constante da coluna, não carry nenhum; o nibble baixo \
                     de `F` entrou `1111` e a ALU não o mascara"
                );
            }
        }
    }
}

#[test]
fn the_bus_access_of_a6_ae_b6_happens_in_the_m2_and_not_during_the_fetch() {
    const ACCUMULATOR: u8 = 0x3C;
    const BEFORE: u8 = 0x0F;
    const AFTER: u8 = 0xF0;

    for opcode in [0xA6u8, 0xAE, 0xB6] {
        let (mut cpu, mut bus) = machine(&[opcode]);
        cpu.registers.a = ACCUMULATOR;
        cpu.registers.set_hl(OPERAND_SLOT);
        cpu.registers.f = 0x00;
        bus.write(OPERAND_SLOT, BEFORE);

        cpu.step(&mut bus);
        bus.write(OPERAND_SLOT, AFTER);
        cpu.step(&mut bus);

        let expected = match opcode {
            0xA6 => ACCUMULATOR & AFTER,
            0xAE => ACCUMULATOR ^ AFTER,
            _ => ACCUMULATOR | AFTER,
        };

        assert_eq!(
            cpu.registers.a, expected,
            "${opcode:02X}: o acesso mora no M2 — o M1 já gastou o dele lendo o \
             opcode (invariante do 1.3). Ler `(HL)` junto do fetch usaria \
             {BEFORE:#04X} em vez de {AFTER:#04X}"
        );
    }
}

#[test]
fn a_bitwise_op_touches_only_a_the_flags_and_the_program_counter() {
    for r8 in 0..8u8 {
        for opcode in [and_a(r8), xor_a(r8), or_a(r8)] {
            let (mut cpu, mut bus) = machine(&[opcode]);
            seed(&mut cpu.registers);
            bus.write(OPERAND_SLOT, 0x0C);

            let mut expected = cpu.registers;

            for _ in 0..m_cycles_of(opcode) {
                cpu.step(&mut bus);
            }

            expected.pc = 0x0101;
            expected.a = cpu.registers.a;
            expected.f = cpu.registers.f;

            assert_eq!(
                cpu.registers, expected,
                "${opcode:02X}: `A` é o único destino da coluna"
            );
        }
    }
}

#[test]
fn no_opcode_of_this_item_writes_to_memory() {
    for r8 in 0..8u8 {
        for opcode in [and_a(r8), xor_a(r8), or_a(r8)] {
            let (mut cpu, mut bus) = machine(&[opcode]);
            seed(&mut cpu.registers);
            bus.write(OPERAND_SLOT, 0x0C);

            for _ in 0..m_cycles_of(opcode) {
                cpu.step(&mut bus);
            }

            assert_eq!(
                bus.read(OPERAND_SLOT),
                0x0C,
                "${opcode:02X}: os M-cycles do `$A6`/`$AE`/`$B6` são fetch e \
                 leitura, nunca escrita"
            );
        }
    }
}

fn decoded_elsewhere(opcode: u8) -> bool {
    opcode == 0x00
        || opcode == 0xC3
        || ((0x40..=0x7F).contains(&opcode) && opcode != 0x76)
        || opcode & 0b1100_0111 == 0b0000_0110
        || opcode & 0b1100_0111 == 0b0000_0010
        || opcode & 0b1100_1111 == 0b0000_0001
        || opcode & 0b1100_1111 == 0b1100_0101
        || opcode & 0b1100_1111 == 0b1100_0001
        || (0x80..=0x9F).contains(&opcode)
        || (0xB8..=0xBF).contains(&opcode)
        || matches!(
            opcode,
            0x08 | 0xE0 | 0xE2 | 0xEA | 0xF0 | 0xF2 | 0xF9 | 0xFA
        )
        || matches!(
            opcode,
            0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE
        )
}

#[test]
fn the_blocks_this_item_decodes_are_exactly_the_twenty_four_opcodes_of_100_101_and_110() {
    const ILLEGAL: [u8; 11] = [
        0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
    ];

    for opcode in 0x00..=0xFFu8 {
        let (mut cpu, mut bus) = machine(&[opcode, 0x00, 0x00, 0x00, 0x00]);
        seed(&mut cpu.registers);

        for _ in 0..5 {
            cpu.step(&mut bus);
        }

        let in_block = (0xA0..=0xB7).contains(&opcode);

        if in_block {
            assert_eq!(
                cpu.lockup(),
                None,
                "${opcode:02X} é um dos vinte e quatro `AND`/`XOR`/`OR A,r8` e o \
                 1.6c o decodifica"
            );
        } else if ILLEGAL.contains(&opcode) {
            assert_eq!(
                cpu.lockup(),
                Some(Lockup::IllegalOpcode(opcode)),
                "${opcode:02X} continua sendo um dos onze que não existem"
            );
        } else if !decoded_elsewhere(opcode) {
            assert_eq!(
                cpu.lockup(),
                Some(Lockup::UndecodedOpcode(opcode)),
                "${opcode:02X} é opcode legítimo fora deste sub-item: o rótulo \
                 tem de continuar dizendo `falta implementar`. `alu a,imm8` \
                 ($C6-$FE) é o 1.6d"
            );
        }
    }
}
