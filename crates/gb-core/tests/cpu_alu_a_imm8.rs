//! ROADMAP 1.6d — `alu a,imm8`: o bloco `11 ooo 110`, `$C6 $CE $D6 $DE $E6 $EE
//! $F6 $FE`.
//!
//! As mesmas oito operações dos três sub-itens do 1.6 (`AluOp` já existe
//! inteiro desde a 0024) — a novidade é só a origem do operando: o `PC`, não
//! um `r8`. Duas M-cycles, `fetch → read(u8)`, como qualquer imediato de 8
//! bits do 1.4b. Ao contrário do `(HL)` do 1.6a (nota 45 — sem testemunha
//! entre os passos), aqui o `PC` **é** testemunha (nota 43): se o operando
//! fosse lido dentro do fetch, o `PC` já estaria em `entry+2` depois do
//! primeiro `step`, não em `entry+1`.

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::{Cpu, Flag, Lockup, Registers};

const ENTRY: usize = 0x0100;
const ENTRY_ADDRESS: u16 = 0x0100;

const SEED_A: u8 = 0x21;
const SEED: [u8; 6] = [0x31, 0x42, 0x53, 0x64, 0xC7, 0xA0];

const ADD_A_IMM8: u8 = 0xC6;
const ADC_A_IMM8: u8 = 0xCE;
const SUB_A_IMM8: u8 = 0xD6;
const SBC_A_IMM8: u8 = 0xDE;
const AND_A_IMM8: u8 = 0xE6;
const XOR_A_IMM8: u8 = 0xEE;
const OR_A_IMM8: u8 = 0xF6;
const CP_A_IMM8: u8 = 0xFE;

const ALL_EIGHT: [u8; 8] = [
    ADD_A_IMM8, ADC_A_IMM8, SUB_A_IMM8, SBC_A_IMM8, AND_A_IMM8, XOR_A_IMM8, OR_A_IMM8, CP_A_IMM8,
];

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

fn expected_result(opcode: u8, a: u8, operand: u8, carry_in: bool) -> u8 {
    let carry_in = u8::from(carry_in);
    match opcode {
        ADD_A_IMM8 => a.wrapping_add(operand),
        ADC_A_IMM8 => a.wrapping_add(operand).wrapping_add(carry_in),
        SUB_A_IMM8 | CP_A_IMM8 => a.wrapping_sub(operand),
        SBC_A_IMM8 => a.wrapping_sub(operand).wrapping_sub(carry_in),
        AND_A_IMM8 => a & operand,
        XOR_A_IMM8 => a ^ operand,
        _ => a | operand,
    }
}

#[test]
fn the_block_2_layout_gives_the_eight_opcodes_gbops_lists() {
    assert_eq!(
        ALL_EIGHT,
        (0..8u8)
            .map(|op| 0b1100_0110 | (op << 3))
            .collect::<Vec<u8>>()
            .as_slice(),
        "`11 ooo 110` do `02-cpu.md`: a operação em 5-3, os bits 2-0 fixos em \
         `110` (não é um `r8` — o imediato substitui o campo inteiro). `000` é \
         `ADD`, `111` é `CP`, na mesma ordem do `10 ooo rrr` do 1.6a/1.6b/1.6c"
    );
}

// 0x2A partilha bits com SEED_A (0x21) sem ser um subconjunto nem um
// superconjunto: AND, XOR e OR dão três resultados distintos entre si (0x20,
// 0x0B, 0x2B) e nenhum coincide com SEED_A — um operando que não distinguisse
// as três deixaria uma troca AND↔XOR↔OR muda.
const DISTINGUISHING_OPERAND: u8 = 0x2A;

#[test]
fn each_operation_applies_the_immediate_operand_into_a() {
    for &opcode in &ALL_EIGHT {
        let (mut cpu, mut bus) = machine(&[opcode, DISTINGUISHING_OPERAND]);
        seed(&mut cpu.registers);

        let expected = expected_result(opcode, SEED_A, DISTINGUISHING_OPERAND, false);

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        let expected_a = if opcode == CP_A_IMM8 {
            SEED_A
        } else {
            expected
        };
        assert_eq!(
            cpu.registers.a, expected_a,
            "${opcode:02X}: o operando vem do byte que segue o opcode, não de um \
             `r8`. `CP` não escreve em `A` (1.6b) — o resultado da conta é \
             {expected:#04X}, mas fica só nas flags"
        );
    }
}

#[test]
fn adc_and_sbc_consume_the_incoming_carry() {
    for opcode in [ADC_A_IMM8, SBC_A_IMM8] {
        let (mut cpu, mut bus) = machine(&[opcode, 0x0C]);
        seed(&mut cpu.registers);
        cpu.registers.set_flag(Flag::C, true);

        let expected = expected_result(opcode, SEED_A, 0x0C, true);

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(
            cpu.registers.a, expected,
            "${opcode:02X}: mesma letra do `r8` — `ADC`/`SBC` usam o `C` de \
             entrada, e sem ele o resultado seria um a menos/a mais"
        );
    }
}

// Controle do teste acima: sem ele, uma ALU que sempre soma/subtrai o carry
// passaria nos dois lados.
#[test]
fn add_and_sub_ignore_the_incoming_carry() {
    for opcode in [ADD_A_IMM8, SUB_A_IMM8] {
        let (mut cpu, mut bus) = machine(&[opcode, 0x0C]);
        seed(&mut cpu.registers);
        cpu.registers.set_flag(Flag::C, true);

        let expected = expected_result(opcode, SEED_A, 0x0C, false);

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(
            cpu.registers.a, expected,
            "${opcode:02X}: `ADD`/`SUB` não usam o `C` de entrada — só `ADC`/\
             `SBC` (bit 3 do campo `ooo`) o fazem"
        );
    }
}

#[test]
fn the_flag_column_matches_the_r8_form_of_the_same_operation() {
    // (opcode, A, operando, Z, N, H, C) — os cantos que cada coluna do
    // 02-cpu.md já fixou para add/adc/sub/sbc/and/xor/or/cp a,r8 (1.6a/b/c).
    const CASES: [(u8, u8, u8, bool, bool, bool, bool); 8] = [
        (ADD_A_IMM8, 0x0F, 0x01, false, false, true, false),
        (ADC_A_IMM8, 0xFF, 0x00, true, false, true, true),
        (SUB_A_IMM8, 0x00, 0x01, false, true, true, true),
        (SBC_A_IMM8, 0x00, 0x00, false, true, true, true),
        (AND_A_IMM8, 0x00, 0x00, true, false, true, false),
        (XOR_A_IMM8, 0xFF, 0xFF, true, false, false, false),
        (OR_A_IMM8, 0x00, 0x00, true, false, false, false),
        (CP_A_IMM8, 0x05, 0x05, true, true, false, false),
    ];

    for (opcode, a, operand, zero, subtract, half, carry) in CASES {
        let (mut cpu, mut bus) = machine(&[opcode, operand]);
        cpu.registers.a = a;
        cpu.registers.f = if opcode == ADC_A_IMM8 || opcode == SBC_A_IMM8 {
            0x10
        } else {
            0x00
        };

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(cpu.registers.flag(Flag::Z), zero, "${opcode:02X}: Z");
        assert_eq!(cpu.registers.flag(Flag::N), subtract, "${opcode:02X}: N");
        assert_eq!(cpu.registers.flag(Flag::H), half, "${opcode:02X}: H");
        assert_eq!(cpu.registers.flag(Flag::C), carry, "${opcode:02X}: C");
    }
}

// A nota 43: ao contrário do `(HL)` do 1.6a (nota 45, sem testemunha), o
// operando aqui vem do `PC`, e o `PC` **é** testemunha entre os M-cycles.
#[test]
fn the_immediate_is_read_in_the_second_m_cycle_and_the_pc_proves_it() {
    for &opcode in &ALL_EIGHT {
        let (mut cpu, mut bus) = machine(&[opcode, 0x0C]);
        seed(&mut cpu.registers);
        let a_before = cpu.registers.a;
        let f_before = cpu.registers.f;

        cpu.step(&mut bus);

        assert_eq!(
            cpu.registers.pc,
            ENTRY_ADDRESS + 1,
            "${opcode:02X}: o M1 é o fetch — lê só o opcode. Ler o imediato \
             junto (dois acessos no mesmo `step`) violaria a invariante do 1.3 \
             e deixaria o `PC` em entry+2 aqui, não em entry+1"
        );
        assert_eq!(
            cpu.registers.a, a_before,
            "${opcode:02X}: o M1 não aplica a operação"
        );
        assert_eq!(
            cpu.registers.f, f_before,
            "${opcode:02X}: nem mexe nas flags"
        );
        assert!(
            !cpu.is_between_instructions(),
            "${opcode:02X} tem 8 T-cycles: dois M-cycles"
        );

        cpu.step(&mut bus);

        assert_eq!(
            cpu.registers.pc,
            ENTRY_ADDRESS + 2,
            "${opcode:02X}: o M2 lê o imediato e aplica — o `PC` anda para o \
             byte seguinte"
        );
        assert!(
            cpu.is_between_instructions(),
            "${opcode:02X}: dois M-cycles e acabou"
        );
    }
}

#[test]
fn an_alu_imm8_op_touches_only_a_the_flags_and_the_program_counter() {
    for &opcode in &ALL_EIGHT {
        let (mut cpu, mut bus) = machine(&[opcode, 0x0C]);
        seed(&mut cpu.registers);

        let mut expected = cpu.registers;

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        expected.pc = ENTRY_ADDRESS + 2;
        expected.a = cpu.registers.a;
        expected.f = cpu.registers.f;

        assert_eq!(
            cpu.registers, expected,
            "${opcode:02X}: `A`, `F` e `PC` são os únicos destinos — nenhum \
             outro registrador muda"
        );
    }
}

#[test]
fn no_opcode_of_this_item_reads_or_writes_the_bus_beyond_the_immediate() {
    const OPERAND_SLOT: u16 = 0xC7A0;

    for &opcode in &ALL_EIGHT {
        let (mut cpu, mut bus) = machine(&[opcode, 0x0C]);
        seed(&mut cpu.registers);
        bus.write(OPERAND_SLOT, 0xAA);

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(
            bus.read(OPERAND_SLOT),
            0xAA,
            "${opcode:02X}: o único endereço tocado é o `PC` — nada escreve em \
             memória, e nada lê de `(HL)`"
        );
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
        || (0xA0..=0xB7).contains(&opcode)
        || (0xB8..=0xBF).contains(&opcode)
        || matches!(
            opcode,
            0x08 | 0xE0 | 0xE2 | 0xEA | 0xF0 | 0xF2 | 0xF9 | 0xFA
        )
}

#[test]
fn the_block_this_item_decodes_is_exactly_the_eight_opcodes_of_11_ooo_110() {
    const ILLEGAL: [u8; 11] = [
        0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
    ];

    for opcode in 0x00..=0xFFu8 {
        let (mut cpu, mut bus) = machine(&[opcode, 0x00, 0x00, 0x00, 0x00]);
        seed(&mut cpu.registers);

        for _ in 0..5 {
            cpu.step(&mut bus);
        }

        let in_block = ALL_EIGHT.contains(&opcode);

        if in_block {
            assert_eq!(
                cpu.lockup(),
                None,
                "${opcode:02X} é um dos oito `alu a,imm8` e o 1.6d o decodifica"
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
                 tem de continuar dizendo `falta implementar`. `INC`/`DEC r8` \
                 (`00 ddd 100`/`00 ddd 101`) é o 1.6e"
            );
        }
    }
}
