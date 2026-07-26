//! ROADMAP 1.4a — o bloco `LD r8,r8`: `$40`–`$7F` **sem** `$76`.

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::{Cpu, Lockup};

const ENTRY: usize = 0x0100;

const SCRATCH: u16 = 0xC0A7;

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

const fn ld(dest: u8, source: u8) -> u8 {
    0b0100_0000 | (dest << 3) | source
}

fn rom_with(program: &[u8]) -> Vec<u8> {
    let mut rom = vec![0x00; NoMbc::MAX_ROM_LEN];
    rom[ENTRY..ENTRY + program.len()].copy_from_slice(program);
    rom
}

fn machine(program: &[u8]) -> (Cpu, Bus) {
    let rom = rom_with(program);
    let checksum = CartridgeHeader::parse(&rom)
        .expect("32 KiB de ROM têm cabeçalho de sobra")
        .checksum();
    let cartridge = NoMbc::new(rom).expect("32 KiB cabem num cartucho sem MBC");

    (Cpu::after_boot_rom(checksum), Bus::new(Box::new(cartridge)))
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
fn every_register_to_register_load_copies_source_into_destination() {
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

#[test]
fn opcode_76_is_halt_and_not_a_load_from_hl_into_hl() {
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

fn decoded_elsewhere(opcode: u8) -> bool {
    opcode == 0x00
        || opcode == 0xC3
        || opcode & 0b1100_0111 == 0b0000_0110
        || opcode & 0b1100_0111 == 0b0000_0010
        || opcode & 0b1100_1111 == 0b0000_0001
        || matches!(opcode, 0xE0 | 0xE2 | 0xEA | 0xF0 | 0xF2 | 0xFA)
}

#[test]
fn the_block_this_item_decodes_is_exactly_40_to_7f_without_76() {
    const ILLEGAL: [u8; 11] = [
        0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
    ];

    for opcode in 0x00..=0xFFu8 {
        let (mut cpu, mut bus) = machine(&[opcode, 0x00, 0x00]);
        seed_registers(&mut cpu);

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
        } else if !decoded_elsewhere(opcode) {
            assert_eq!(
                cpu.lockup(),
                Some(Lockup::UndecodedOpcode(opcode)),
                "${opcode:02X} é opcode legítimo fora deste sub-item: o \
                 rótulo tem de continuar dizendo `falta implementar`"
            );
        }
    }
}

#[test]
fn no_load_in_the_block_touches_the_flags() {
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
