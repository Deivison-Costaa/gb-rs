//! ROADMAP 1.6b — `SUB a,r8`, `SBC a,r8` e `CP a,r8`: os blocos `10 010 rrr`,
//! `10 011 rrr` e `10 111 rrr`.
//!
//! Mesma letra do 1.6a, grandeza invertida: `H` é *empréstimo* do nibble baixo
//! (não carry) e `C` é o resultado ficar *"lower than zero"* (§ The Carry
//! Flag) — não "maior que $FF". `N` é `1` **literal** nas 24 linhas, ao
//! contrário do 1.6a. `CP` é a única das oito operações da ALU que não
//! escreve em `A`: a coluna de flags dele é idêntica à do `SUB`, e só a
//! ausência da escrita separa os dois — por isso `CP A,A` e `SUB A,A` têm o
//! mesmo `Z=1` e só um teste que olhe `A` depois separa quem escreveu de quem
//! só comparou.

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::{Cpu, Flag, Lockup, Registers};

mod support;
use support::decoded_elsewhere;

const ENTRY: usize = 0x0100;
const ENTRY_ADDRESS: u16 = 0x0100;

const OPERAND_SLOT: u16 = 0xC7A0;

const SEED_A: u8 = 0x21;
const SEED: [u8; 6] = [0x31, 0x42, 0x53, 0x64, 0xC7, 0xA0];

const R8_NAMES: [&str; 8] = ["B", "C", "D", "E", "H", "L", "(HL)", "A"];

const fn sub_a(r8: u8) -> u8 {
    0b1001_0000 | r8
}

const fn sbc_a(r8: u8) -> u8 {
    0b1001_1000 | r8
}

const fn cp_a(r8: u8) -> u8 {
    0b1011_1000 | r8
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
        .map(sub_a)
        .chain((0..8).map(sbc_a))
        .chain((0..8).map(cp_a))
        .collect();

    let mut expected: Vec<u8> = (0x90..=0x9Fu8).collect();
    expected.extend(0xB8..=0xBFu8);

    assert_eq!(
        opcodes, expected,
        "§ Block 2 do `02-cpu.md`: `10 ooo rrr`. `010` é `SUB` ($90), `011` é \
         `SBC` ($98) e `111` é `CP` ($B8) — os três únicos ainda não decodificados \
         do bloco. `100`/`101`/`110` (`AND`/`XOR`/`OR`, $A0–$B7) são o 1.6c"
    );
}

#[test]
fn sub_subtracts_the_operand_from_a_for_every_r8_register() {
    for r8 in 0..8u8 {
        let opcode = sub_a(r8);
        let (mut cpu, mut bus) = machine(&[opcode]);
        seed(&mut cpu.registers);
        bus.write(OPERAND_SLOT, 0x0C);

        let expected = SEED_A.wrapping_sub(operand_of(&cpu.registers, r8, 0x0C));

        for _ in 0..m_cycles_of(opcode) {
            cpu.step(&mut bus);
        }

        assert_eq!(
            cpu.registers.a, expected,
            "${opcode:02X} é `SUB A,{}`: o operando sai do acumulador. `$97` \
             (`SUB A,A`) zera — o mesmo registrador é minuendo e subtraendo",
            R8_NAMES[r8 as usize]
        );
    }
}

#[test]
fn sbc_subtracts_the_operand_and_the_incoming_borrow_from_a() {
    for r8 in 0..8u8 {
        let opcode = sbc_a(r8);
        let (mut cpu, mut bus) = machine(&[opcode]);
        seed(&mut cpu.registers);
        cpu.registers.set_flag(Flag::C, true);
        bus.write(OPERAND_SLOT, 0x0C);

        let expected = SEED_A
            .wrapping_sub(operand_of(&cpu.registers, r8, 0x0C))
            .wrapping_sub(1);

        for _ in 0..m_cycles_of(opcode) {
            cpu.step(&mut bus);
        }

        assert_eq!(
            cpu.registers.a, expected,
            "${opcode:02X} é `SBC A,{}`: a § The Carry Flag lista `SBC` entre as \
             instruções que **usam** o `C`. Sem o empréstimo de entrada o \
             resultado seria um a mais — invisível com `C` de entrada limpo",
            R8_NAMES[r8 as usize]
        );
    }
}

#[test]
fn sub_ignores_the_incoming_borrow() {
    for r8 in 0..8u8 {
        let opcode = sub_a(r8);
        let (mut cpu, mut bus) = machine(&[opcode]);
        seed(&mut cpu.registers);
        cpu.registers.set_flag(Flag::C, true);
        bus.write(OPERAND_SLOT, 0x0C);

        let expected = SEED_A.wrapping_sub(operand_of(&cpu.registers, r8, 0x0C));

        for _ in 0..m_cycles_of(opcode) {
            cpu.step(&mut bus);
        }

        assert_eq!(
            cpu.registers.a, expected,
            "${opcode:02X} é `SUB`, não `SBC`: o que separa os dois é o bit 3. \
             Controle do teste do `SBC` — sem ele, uma ALU que sempre subtrai o \
             empréstimo passa nos dois"
        );
    }
}

#[test]
fn n_is_literal_one_and_h_c_are_borrows_not_carries() {
    // (A, operando, Z, H, C) — H é empréstimo do nibble baixo, C do byte.
    const CASES: [(u8, u8, bool, bool, bool); 5] = [
        (0x10, 0x01, false, true, false),
        (0x00, 0x01, false, true, true),
        (0xFF, 0xFF, true, false, false),
        (0x08, 0x07, false, false, false),
        (0x00, 0x80, false, false, true),
    ];

    for (a, operand, zero, half, carry) in CASES {
        let (mut cpu, mut bus) = machine(&[sub_a(0)]);
        cpu.registers.a = a;
        cpu.registers.b = operand;
        cpu.registers.f = 0x00;

        cpu.step(&mut bus);

        assert!(
            cpu.registers.flag(Flag::N),
            "`SUB A,B`: `N` é `1` literal na coluna das 24 linhas, não conta — \
             ao contrário do `ADD`/`ADC` do 1.6a, que têm `0` no mesmo lugar"
        );
        assert_eq!(
            cpu.registers.flag(Flag::H),
            half,
            "`SUB A,B` com A={a:#04X} e B={operand:#04X}: `H` é o empréstimo do \
             nibble baixo — grandeza invertida do carry do 1.6a, mesma letra"
        );
        assert_eq!(
            cpu.registers.flag(Flag::C),
            carry,
            "A={a:#04X} - B={operand:#04X}: `C` é o resultado ficar *lower than \
             zero* (§ The Carry Flag), não *higher than $FF*"
        );
        assert_eq!(
            cpu.registers.flag(Flag::Z),
            zero,
            "A={a:#04X} - B={operand:#04X}: `Z` é sobre o byte que sobra depois \
             do `wrapping_sub`, não sobre um resultado negativo hipotético"
        );
    }
}

#[test]
fn the_incoming_borrow_of_sbc_counts_for_the_half_borrow_too() {
    for (carry_in, half, result) in [(false, false, 0x10u8), (true, true, 0x0F)] {
        let (mut cpu, mut bus) = machine(&[sbc_a(0)]);
        cpu.registers.a = 0x10;
        cpu.registers.b = 0x00;
        cpu.registers.set_flag(Flag::C, carry_in);

        cpu.step(&mut bus);

        assert_eq!(
            cpu.registers.a,
            result,
            "A=$10 - B=$00 - C={}",
            u8::from(carry_in)
        );
        assert_eq!(
            cpu.registers.flag(Flag::H),
            half,
            "`SBC A,B` com A=$10, B=$00 e C={}: o empréstimo de entrada faz parte \
             do nibble baixo do resultado. Uma ALU que só empreste do byte total \
             acerta `A` e erra `H` — $10-$00 não pede nibble nenhum, mas \
             $10-$00-1 pede",
            u8::from(carry_in)
        );
    }
}

#[test]
fn cp_computes_the_same_flags_as_sub_but_does_not_write_a() {
    for r8 in 0..8u8 {
        let sub_opcode = sub_a(r8);
        let (mut sub_cpu, mut sub_bus) = machine(&[sub_opcode]);
        seed(&mut sub_cpu.registers);
        sub_bus.write(OPERAND_SLOT, 0x0C);
        for _ in 0..m_cycles_of(sub_opcode) {
            sub_cpu.step(&mut sub_bus);
        }

        let cp_opcode = cp_a(r8);
        let (mut cp_cpu, mut cp_bus) = machine(&[cp_opcode]);
        seed(&mut cp_cpu.registers);
        cp_bus.write(OPERAND_SLOT, 0x0C);
        for _ in 0..m_cycles_of(cp_opcode) {
            cp_cpu.step(&mut cp_bus);
        }

        assert_eq!(
            cp_cpu.registers.f, sub_cpu.registers.f,
            "${cp_opcode:02X} (`CP A,{}`) tem a mesma coluna de flags que \
             ${sub_opcode:02X} (`SUB A,{0}`), letra por letra",
            R8_NAMES[r8 as usize]
        );
        assert_eq!(
            cp_cpu.registers.a, SEED_A,
            "${cp_opcode:02X}: `CP` só produz flags — o acumulador não muda, ao \
             contrário do `SUB` que acabou de zerá-lo ou não neste mesmo caso"
        );
    }
}

#[test]
fn cp_a_a_and_sub_a_a_agree_on_zero_but_disagree_on_writing_a() {
    let (mut cp_cpu, mut cp_bus) = machine(&[cp_a(7)]);
    cp_cpu.registers.a = SEED_A;

    cp_cpu.step(&mut cp_bus);

    assert!(
        cp_cpu.registers.flag(Flag::Z),
        "`CP A,A` é sempre `Z=1`, igual `SUB A,A` — o par que separa \"escreveu \
         o resultado\" de \"só comparou\", e sozinho o `Z` não separa nada"
    );
    assert_eq!(
        cp_cpu.registers.a, SEED_A,
        "`CP A,A` não escreve: `A` continua {SEED_A:#04X}. Um `apply` que \
         sempre escreva passaria despercebido no `Z`, porque `SUB A,A` também \
         dá `Z=1` — só olhar `A` depois separa os dois"
    );
}

#[test]
fn every_subtraction_of_the_two_writing_operations_matches_the_flag_definitions() {
    for (base, uses_borrow_in) in [(0x90u8, false), (0x98, true)] {
        let (mut cpu, mut bus) = machine(&[base]);

        for a in 0..=0xFFu8 {
            for operand in 0..=0xFFu8 {
                for carry_in in [false, true] {
                    cpu.registers.f = if carry_in { 0xFF } else { 0xEF };
                    cpu.registers.pc = ENTRY_ADDRESS;
                    cpu.registers.a = a;
                    cpu.registers.b = operand;

                    cpu.step(&mut bus);

                    let subtrahend = u16::from(operand) + u16::from(uses_borrow_in && carry_in);
                    let borrowed = u16::from(a) < subtrahend;
                    let result = cpu.registers.a;

                    let half = (a ^ operand ^ result) & 0x10 != 0;

                    let expected = u8::from(result == 0) << 7
                        | 1 << 6
                        | u8::from(half) << 5
                        | u8::from(borrowed) << 4
                        | 0x0F;

                    assert_eq!(
                        u16::from(result),
                        (u16::from(a).wrapping_sub(subtrahend)) & 0xFF,
                        "${base:02X}: A={a:#04X}, B={operand:#04X}, C de entrada={}",
                        u8::from(carry_in)
                    );
                    assert_eq!(
                        cpu.registers.f,
                        expected,
                        "${base:02X} com A={a:#04X}, B={operand:#04X} e C de \
                         entrada={}: varredura dos 65536 pares. `N` é `1` literal \
                         e entrou `0`; `H`/`C` são empréstimo, não carry; o \
                         nibble baixo de `F` entrou `1111` e a ALU não o mascara",
                        u8::from(carry_in)
                    );
                }
            }
        }
    }
}

#[test]
fn the_bus_access_of_96_9e_be_happens_in_the_m2_and_not_during_the_fetch() {
    const BEFORE: u8 = 0x11;
    const AFTER: u8 = 0x22;

    for opcode in [0x96u8, 0x9E, 0xBE] {
        let (mut cpu, mut bus) = machine(&[opcode]);
        cpu.registers.a = 0x33;
        cpu.registers.set_hl(OPERAND_SLOT);
        cpu.registers.f = 0x00;
        bus.write(OPERAND_SLOT, BEFORE);

        cpu.step(&mut bus);
        bus.write(OPERAND_SLOT, AFTER);
        cpu.step(&mut bus);

        let expected = if opcode == 0xBE {
            0x33
        } else {
            0x33u8.wrapping_sub(AFTER)
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
fn a_sub_or_sbc_touches_only_a_the_flags_and_the_program_counter() {
    for r8 in 0..8u8 {
        for opcode in [sub_a(r8), sbc_a(r8)] {
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
                "${opcode:02X}: `A` é o único destino da coluna, como no 1.6a"
            );
        }
    }
}

#[test]
fn cp_touches_only_the_flags_and_the_program_counter() {
    for r8 in 0..8u8 {
        let opcode = cp_a(r8);
        let (mut cpu, mut bus) = machine(&[opcode]);
        seed(&mut cpu.registers);
        bus.write(OPERAND_SLOT, 0x0C);

        let mut expected = cpu.registers;

        for _ in 0..m_cycles_of(opcode) {
            cpu.step(&mut bus);
        }

        expected.pc = 0x0101;
        expected.f = cpu.registers.f;

        assert_eq!(
            cpu.registers, expected,
            "${opcode:02X}: `CP` não escreve nem em `A` — só produz flags"
        );
    }
}

#[test]
fn no_opcode_of_this_item_writes_to_memory() {
    for r8 in 0..8u8 {
        for opcode in [sub_a(r8), sbc_a(r8), cp_a(r8)] {
            let (mut cpu, mut bus) = machine(&[opcode]);
            seed(&mut cpu.registers);
            bus.write(OPERAND_SLOT, 0x0C);

            for _ in 0..m_cycles_of(opcode) {
                cpu.step(&mut bus);
            }

            assert_eq!(
                bus.read(OPERAND_SLOT),
                0x0C,
                "${opcode:02X}: os M-cycles do `$96`/`$9E`/`$BE` são fetch e \
                 leitura, nunca escrita"
            );
        }
    }
}

#[test]
fn the_blocks_this_item_decodes_are_exactly_the_twenty_four_opcodes_of_1001_and_1011() {
    const ILLEGAL: [u8; 11] = [
        0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
    ];

    for opcode in 0x00..=0xFFu8 {
        let (mut cpu, mut bus) = machine(&[opcode, 0x00, 0x00, 0x00, 0x00]);
        seed(&mut cpu.registers);

        for _ in 0..5 {
            cpu.step(&mut bus);
        }

        let in_block = (0x90..=0x9F).contains(&opcode) || (0xB8..=0xBF).contains(&opcode);

        if in_block {
            assert_eq!(
                cpu.lockup(),
                None,
                "${opcode:02X} é um dos vinte e quatro `SUB`/`SBC`/`CP A,r8` e o \
                 1.6b o decodifica"
            );
        } else if ILLEGAL.contains(&opcode) {
            assert_eq!(
                cpu.lockup(),
                Some(Lockup::IllegalOpcode(opcode)),
                "${opcode:02X} continua sendo um dos onze que não existem"
            );
        } else if decoded_elsewhere(opcode) {
            assert_eq!(
                cpu.lockup(),
                None,
                "${opcode:02X}: decodificado por outro sub-item"
            );
        } else {
            assert_eq!(
                cpu.lockup(),
                Some(Lockup::UndecodedOpcode(opcode)),
                "${opcode:02X} é opcode legítimo fora deste sub-item: o rótulo \
                 tem de continuar dizendo `falta implementar`. `AND`/`XOR`/`OR` \
                 ($A0–$B7) são o 1.6c"
            );
        }
    }
}
