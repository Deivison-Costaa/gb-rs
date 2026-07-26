//! ROADMAP 1.4b — os imediatos de 8 bits: `LD r8,u8` e `LD (HL),u8`.

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::{Cpu, Lockup};

const ENTRY: usize = 0x0100;

const SCRATCH: u16 = 0xC0A7;

const UNTOUCHED: u8 = 0xE7;

const IMMEDIATE: u8 = 0x5A;

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

const HL_INDEX: u8 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Operand {
    Register(&'static str),
    Memory,
}

const fn ld_imm(dest: u8) -> u8 {
    0b0000_0110 | (dest << 3)
}

fn rom_with(program: &[u8]) -> Vec<u8> {
    let mut rom = vec![0x00; NoMbc::MAX_ROM_LEN];
    rom[ENTRY..ENTRY + program.len()].copy_from_slice(program);
    rom
}

fn machine_with_rom(rom: Vec<u8>) -> (Cpu, Bus) {
    let checksum = CartridgeHeader::parse(&rom)
        .expect("32 KiB de ROM têm cabeçalho de sobra")
        .checksum();
    let cartridge = NoMbc::new(rom).expect("32 KiB cabem num cartucho sem MBC");

    (Cpu::after_boot_rom(checksum), Bus::new(Box::new(cartridge)))
}

fn machine(program: &[u8]) -> (Cpu, Bus) {
    machine_with_rom(rom_with(program))
}

fn seed_registers(cpu: &mut Cpu) {
    cpu.registers.b = 0xB1;
    cpu.registers.c = 0xC2;
    cpu.registers.d = 0xD3;
    cpu.registers.e = 0xE4;
    cpu.registers.a = 0xA5;
    cpu.registers.set_hl(SCRATCH);
}

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

#[test]
fn every_immediate_load_into_a_register_takes_the_byte_after_the_opcode() {
    for (dest_index, dest) in R8.iter().enumerate() {
        let Operand::Register(dest) = dest else {
            continue;
        };
        #[expect(clippy::cast_possible_truncation, reason = "índices de 0 a 7")]
        let opcode = ld_imm(dest_index as u8);

        let (mut cpu, mut bus) = machine(&[opcode, IMMEDIATE]);
        seed_registers(&mut cpu);

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(
            read_register(&cpu, dest),
            IMMEDIATE,
            "${opcode:02X} é `LD {dest},u8`"
        );
    }
}

#[test]
fn an_immediate_load_into_a_register_is_two_m_cycles_and_the_register_changes_on_the_second() {
    let (mut cpu, mut bus) = machine(&[0x06, IMMEDIATE]);
    seed_registers(&mut cpu);
    let before = cpu.registers.b;

    cpu.step(&mut bus);
    assert_eq!(
        cpu.registers.b, before,
        "no M1 só o opcode chegou; o operando ainda está na ROM"
    );
    assert_eq!(cpu.registers.pc, 0x0101, "o M1 passou pelo opcode, e só");
    assert!(
        !cpu.is_between_instructions(),
        "`LD B,u8` tem 8 T-cycles: uma implementação que já terminou no fetch \
         leu dois bytes num M-cycle"
    );

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.b, IMMEDIATE, "o M2 é o `read(u8->B)`");
    assert_eq!(cpu.registers.pc, 0x0102, "e é ele que passa pelo operando");
    assert!(
        cpu.is_between_instructions(),
        "dois M-cycles e acabou: não há terceiro"
    );
}

#[test]
fn an_immediate_load_does_not_execute_its_operand() {
    let (mut cpu, mut bus) = machine(&[0x06, 0xD3]);
    seed_registers(&mut cpu);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        cpu.registers.b, 0xD3,
        "`LD B,$D3` carrega o byte: aqui ele é dado, não instrução"
    );
    assert_eq!(cpu.lockup(), None, "e ninguém tentou decodificá-lo");
    assert_eq!(cpu.registers.pc, 0x0102, "`LD B,u8` tem 2 bytes");
}

#[test]
fn an_immediate_load_touches_nothing_but_the_destination() {
    let (mut cpu, mut bus) = machine(&[0x16, IMMEDIATE]);
    seed_registers(&mut cpu);

    let mut expected = cpu.registers;
    expected.d = IMMEDIATE;
    expected.pc = 0x0102;

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        cpu.registers, expected,
        "`LD D,u8` tem `-` nas quatro colunas de flag e mexe em um registrador só"
    );
}

#[test]
fn an_immediate_load_reads_its_operand_across_the_program_counter_wrap() {
    let mut rom = rom_with(&[]);
    rom[0x0000] = IMMEDIATE;
    let (mut cpu, mut bus) = machine_with_rom(rom);
    seed_registers(&mut cpu);

    bus.write(0xFFFF, 0x06);
    cpu.registers.pc = 0xFFFF;

    cpu.step(&mut bus);
    assert_eq!(
        cpu.registers.pc, 0x0000,
        "o `PC` é de 16 bits e dá a volta: depois de `$FFFF` vem `$0000`"
    );

    cpu.step(&mut bus);
    assert_eq!(
        cpu.registers.b, IMMEDIATE,
        "o operando de `LD B,u8` é o byte seguinte, e `$0000` é o byte \
         seguinte a `$FFFF`"
    );
    assert_eq!(cpu.registers.pc, 0x0001, "e o `PC` continua andando dali");
}

#[test]
fn storing_an_immediate_at_hl_writes_the_byte_after_the_opcode() {
    assert_eq!(
        ld_imm(HL_INDEX),
        0x36,
        "a fórmula `00 ddd 110` com o destino em `[hl]` dá $36"
    );

    let (mut cpu, mut bus) = machine(&[0x36, IMMEDIATE]);
    seed_registers(&mut cpu);
    bus.write(SCRATCH, UNTOUCHED);

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        bus.read(SCRATCH),
        IMMEDIATE,
        "`LD (HL),u8` grava o imediato onde `HL` aponta"
    );
    assert_eq!(cpu.registers.pc, 0x0102, "`LD (HL),u8` tem 2 bytes");
    assert_eq!(
        cpu.registers.hl(),
        SCRATCH,
        "e `HL` não se move: quem mexe em `HL` é o 1.4c"
    );
}

#[test]
fn storing_an_immediate_at_hl_is_three_m_cycles_and_the_write_is_the_third() {
    let (mut cpu, mut bus) = machine(&[0x36, IMMEDIATE]);
    seed_registers(&mut cpu);
    bus.write(SCRATCH, UNTOUCHED);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x0101, "o M1 é o fetch do opcode");
    assert_eq!(
        bus.read(SCRATCH),
        UNTOUCHED,
        "no M1 só o opcode chegou; nada foi escrito"
    );
    assert!(
        !cpu.is_between_instructions(),
        "`LD (HL),u8` tem 12 T-cycles: três M-cycles"
    );

    cpu.step(&mut bus);
    assert_eq!(
        cpu.registers.pc, 0x0102,
        "o M2 é o `read(u8)`: o `PC` passa pelo operando"
    );
    assert_eq!(
        bus.read(SCRATCH),
        UNTOUCHED,
        "e a escrita ainda **não** aconteceu — ela é o M3, não este M-cycle \
         com um `internal` depois"
    );
    assert!(
        !cpu.is_between_instructions(),
        "ainda falta o `write((HL))`"
    );

    cpu.step(&mut bus);
    assert_eq!(bus.read(SCRATCH), IMMEDIATE, "o M3 é o `write((HL))`");
    assert!(
        cpu.is_between_instructions(),
        "três M-cycles e acabou: não há quarto"
    );
}

#[test]
fn storing_an_immediate_at_hl_does_not_execute_its_operand() {
    let (mut cpu, mut bus) = machine(&[0x36, 0xD3]);
    seed_registers(&mut cpu);

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        bus.read(SCRATCH),
        0xD3,
        "`LD (HL),$D3` grava o byte: aqui ele é dado, não instrução"
    );
    assert_eq!(cpu.lockup(), None, "e ninguém tentou decodificá-lo");
}

#[test]
fn no_immediate_load_touches_the_flags() {
    const DIRTY_F: u8 = 0b1111_1010;

    for dest_index in 0..8u8 {
        let opcode = ld_imm(dest_index);

        let (mut cpu, mut bus) = machine(&[opcode, IMMEDIATE]);
        seed_registers(&mut cpu);
        cpu.registers.f = DIRTY_F;

        cpu.step(&mut bus);
        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(
            cpu.registers.f, DIRTY_F,
            "${opcode:02X} tem `-` nas quatro colunas de flag da tabela"
        );
    }
}

#[test]
fn the_block_this_item_decodes_is_exactly_the_eight_opcodes_of_00_ddd_110() {
    const ILLEGAL: [u8; 11] = [
        0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
    ];

    for opcode in 0x00..=0xFFu8 {
        let (mut cpu, mut bus) = machine(&[opcode, 0x00, 0x00]);
        seed_registers(&mut cpu);

        cpu.step(&mut bus);
        cpu.step(&mut bus);
        cpu.step(&mut bus);

        let previously_decoded = opcode == 0x00
            || opcode == 0xC3
            || ((0x40..=0x7F).contains(&opcode) && opcode != 0x76)
            || opcode & 0b1100_0111 == 0b0000_0010
            || matches!(opcode, 0xE0 | 0xE2 | 0xEA | 0xF0 | 0xF2 | 0xFA);
        let in_block = opcode & 0b1100_0111 == 0b0000_0110;

        if in_block {
            assert_eq!(
                cpu.lockup(),
                None,
                "${opcode:02X} está no bloco `00 ddd 110` e o 1.4b o decodifica"
            );
        } else if ILLEGAL.contains(&opcode) {
            assert_eq!(
                cpu.lockup(),
                Some(Lockup::IllegalOpcode(opcode)),
                "${opcode:02X} continua sendo um dos onze que não existem"
            );
        } else if !previously_decoded {
            assert_eq!(
                cpu.lockup(),
                Some(Lockup::UndecodedOpcode(opcode)),
                "${opcode:02X} é opcode legítimo fora deste sub-item: o rótulo \
                 tem de continuar dizendo `falta implementar`"
            );
        }
    }
}
