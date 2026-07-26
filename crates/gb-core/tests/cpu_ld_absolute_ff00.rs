//! ROADMAP 1.4d — endereço absoluto e a página `$FF00`: `LD (u16),A`,

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::{Cpu, Lockup};

const ENTRY: usize = 0x0100;

const LDH_C_A: u8 = 0b1110_0010;
const LDH_IMM8_A: u8 = 0b1110_0000;
const LD_IMM16_A: u8 = 0b1110_1010;
const LDH_A_C: u8 = 0b1111_0010;
const LDH_A_IMM8: u8 = 0b1111_0000;
const LD_A_IMM16: u8 = 0b1111_1010;

const THIS_ITEM: [u8; 6] = [
    LDH_C_A, LDH_IMM8_A, LD_IMM16_A, LDH_A_C, LDH_A_IMM8, LD_A_IMM16,
];

const OFFSET_IMM8: u8 = 0x85;
const VIA_IMM8: u16 = 0xFF00 | OFFSET_IMM8 as u16;

const OFFSET_IN_C: u8 = 0x8A;
const VIA_C: u16 = 0xFF00 | OFFSET_IN_C as u16;

const DECOY_IN_B: u8 = 0x12;

const ABSOLUTE: u16 = 0xC3D7;

const ABSOLUTE_SWAPPED: u16 = 0xD7C3;

const STORED: u8 = 0x5A;

const UNTOUCHED: u8 = 0x6D;

const AT_TARGET: u8 = 0xE7;

const AT_NEIGHBOUR: u8 = 0x3C;

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
    cpu.registers.a = STORED;
    cpu.registers.b = DECOY_IN_B;
    cpu.registers.c = OFFSET_IN_C;
}

fn seed_memory_around(bus: &mut Bus, address: u16) {
    bus.write(address.wrapping_sub(1), AT_NEIGHBOUR);
    bus.write(address, AT_TARGET);
    bus.write(address.wrapping_add(1), AT_NEIGHBOUR);
}

#[test]
fn the_six_bit_layouts_of_block_3_give_the_six_opcodes_gbops_lists() {
    assert_eq!(
        THIS_ITEM,
        [0xE2, 0xE0, 0xEA, 0xF2, 0xF0, 0xFA],
        "os seis layouts, na ordem em que a § Block 3 os empilha"
    );
}

#[test]
fn each_pair_differs_only_in_bit_4() {
    for (store, load) in [
        (LDH_C_A, LDH_A_C),
        (LDH_IMM8_A, LDH_A_IMM8),
        (LD_IMM16_A, LD_A_IMM16),
    ] {
        assert_eq!(
            store ^ load,
            0b0001_0000,
            "${store:02X} e ${load:02X} diferem no bit 4 e em mais nada"
        );
        assert_eq!(store & 0b0001_0000, 0, "${store:02X} é o `$Ex`: escreve");
        assert_eq!(load & 0b0001_0000, 0b0001_0000, "${load:02X} é o `$Fx`: lê");
    }
}

#[test]
fn store_through_c_writes_a_at_ff00_plus_c() {
    let (mut cpu, mut bus) = machine(&[LDH_C_A]);
    seed_registers(&mut cpu);
    bus.write(VIA_C, UNTOUCHED);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(bus.read(VIA_C), STORED, "`$E2` é `LD (FF00+C),A`");
    assert_eq!(
        cpu.registers.c, OFFSET_IN_C,
        "`C` é o índice, e a instrução não mexe nele"
    );
}

#[test]
fn load_through_c_reads_into_a_from_ff00_plus_c() {
    let (mut cpu, mut bus) = machine(&[LDH_A_C]);
    seed_registers(&mut cpu);
    seed_memory_around(&mut bus, VIA_C);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(cpu.registers.a, AT_TARGET, "`$F2` é `LD A,(FF00+C)`");
    assert_eq!(cpu.registers.c, OFFSET_IN_C, "e `C` continua o índice");
}

#[test]
fn the_c_indexed_pair_is_one_byte_and_the_next_opcode_follows_immediately() {
    for opcode in [LDH_C_A, LDH_A_C] {
        let (mut cpu, mut bus) = machine(&[opcode, 0x46, 0x00]);
        seed_registers(&mut cpu);
        seed_memory_around(&mut bus, VIA_C);
        cpu.registers.set_hl(ABSOLUTE);
        bus.write(ABSOLUTE, AT_TARGET);

        cpu.step(&mut bus);
        assert_eq!(
            cpu.registers.pc, 0x0101,
            "${opcode:02X} tem 1 byte: o fetch anda um, e `C` já é o operando"
        );

        cpu.step(&mut bus);
        assert!(
            cpu.is_between_instructions(),
            "${opcode:02X} tem 8 T-cycles: acabou no M2"
        );

        cpu.step(&mut bus);
        cpu.step(&mut bus);
        assert_eq!(
            cpu.registers.b, AT_TARGET,
            "${opcode:02X}: a instrução seguinte é a de `$0101`, e não a de \
             `$0102` — um byte a mais no `PC` desalinha o fluxo inteiro"
        );
    }
}

#[test]
fn the_c_indexed_store_writes_on_the_second_m_cycle_and_not_on_the_fetch() {
    let (mut cpu, mut bus) = machine(&[LDH_C_A]);
    seed_registers(&mut cpu);
    bus.write(VIA_C, UNTOUCHED);

    cpu.step(&mut bus);
    assert_eq!(
        bus.read(VIA_C),
        UNTOUCHED,
        "M1: só o opcode chegou; o alvo está intacto"
    );
    assert!(!cpu.is_between_instructions(), "M1: falta o M2");

    cpu.step(&mut bus);
    assert_eq!(bus.read(VIA_C), STORED, "M2: `write(A->(FF00+C))`");
    assert!(cpu.is_between_instructions(), "M2: e acabou");
}

#[test]
fn the_c_indexed_load_writes_a_on_the_second_m_cycle_and_not_on_the_fetch() {
    let (mut cpu, mut bus) = machine(&[LDH_A_C]);
    seed_registers(&mut cpu);
    seed_memory_around(&mut bus, VIA_C);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, STORED, "M1: `A` ainda é o de antes");
    assert!(!cpu.is_between_instructions(), "M1: falta o M2");

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, AT_TARGET, "M2: `read((FF00+C)->A)`");
    assert!(cpu.is_between_instructions(), "M2: e acabou");
}

#[test]
fn store_through_an_immediate_offset_writes_a_at_ff00_plus_u8() {
    let (mut cpu, mut bus) = machine(&[LDH_IMM8_A, OFFSET_IMM8]);
    seed_registers(&mut cpu);
    bus.write(VIA_IMM8, UNTOUCHED);

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(bus.read(VIA_IMM8), STORED, "`$E0` é `LD (FF00+u8),A`");
    assert_eq!(cpu.registers.pc, 0x0102, "`$E0` tem 2 bytes");
}

#[test]
fn load_through_an_immediate_offset_reads_into_a_from_ff00_plus_u8() {
    let (mut cpu, mut bus) = machine(&[LDH_A_IMM8, OFFSET_IMM8]);
    seed_registers(&mut cpu);
    seed_memory_around(&mut bus, VIA_IMM8);

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(cpu.registers.a, AT_TARGET, "`$F0` é `LD A,(FF00+u8)`");
    assert_eq!(cpu.registers.pc, 0x0102, "`$F0` tem 2 bytes");
}

#[test]
fn the_immediate_offset_store_is_three_m_cycles_with_the_write_last() {
    let (mut cpu, mut bus) = machine(&[LDH_IMM8_A, OFFSET_IMM8]);
    seed_registers(&mut cpu);
    bus.write(VIA_IMM8, UNTOUCHED);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x0101, "M1: o fetch do opcode");
    assert_eq!(bus.read(VIA_IMM8), UNTOUCHED, "M1: nada escrito");

    cpu.step(&mut bus);
    assert_eq!(
        cpu.registers.pc, 0x0102,
        "M2: `read(u8)`, e o `PC` passa por ele"
    );
    assert_eq!(
        bus.read(VIA_IMM8),
        UNTOUCHED,
        "M2: o deslocamento chegou, mas a escrita é do M3"
    );
    assert!(!cpu.is_between_instructions(), "M2: falta o M3");

    cpu.step(&mut bus);
    assert_eq!(bus.read(VIA_IMM8), STORED, "M3: `write(A->(FF00+u8))`");
    assert!(cpu.is_between_instructions(), "M3: e acabou");
}

#[test]
fn the_immediate_offset_load_is_three_m_cycles_with_a_changing_last() {
    let (mut cpu, mut bus) = machine(&[LDH_A_IMM8, OFFSET_IMM8]);
    seed_registers(&mut cpu);
    seed_memory_around(&mut bus, VIA_IMM8);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x0101, "M1: o fetch do opcode");
    assert_eq!(cpu.registers.a, STORED, "M1: `A` intacto");

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x0102, "M2: `read(u8)`");
    assert_eq!(
        cpu.registers.a, STORED,
        "M2: o deslocamento chegou, mas `A` só muda no M3"
    );
    assert!(!cpu.is_between_instructions(), "M2: falta o M3");

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, AT_TARGET, "M3: `read((FF00+u8)->A)`");
    assert!(cpu.is_between_instructions(), "M3: e acabou");
}

#[test]
fn the_immediate_offset_reaches_the_whole_high_page_including_ie() {
    let (mut cpu, mut bus) = machine(&[LDH_IMM8_A, 0xFF]);
    seed_registers(&mut cpu);

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        bus.read(0xFFFF),
        STORED,
        "`LD (FF00+$FF),A` escreve no `IE`, e não dá a volta para `$00FF`"
    );
}

#[test]
fn store_to_an_absolute_address_writes_a_there() {
    let (mut cpu, mut bus) = machine(&[LD_IMM16_A, 0xD7, 0xC3]);
    seed_registers(&mut cpu);
    bus.write(ABSOLUTE, UNTOUCHED);
    bus.write(ABSOLUTE_SWAPPED, UNTOUCHED);

    for _ in 0..4 {
        cpu.step(&mut bus);
    }

    assert_eq!(bus.read(ABSOLUTE), STORED, "`$EA` é `LD (u16),A`");
    assert_eq!(
        bus.read(ABSOLUTE_SWAPPED),
        UNTOUCHED,
        "o operando é little-endian: `$D7 $C3` é `$C3D7`, e não `$D7C3` — que \
         também é WRAM e aceitaria a escrita sem reclamar"
    );
    assert_eq!(cpu.registers.pc, 0x0103, "`$EA` tem 3 bytes");
}

#[test]
fn load_from_an_absolute_address_reads_into_a() {
    let (mut cpu, mut bus) = machine(&[LD_A_IMM16, 0xD7, 0xC3]);
    seed_registers(&mut cpu);
    seed_memory_around(&mut bus, ABSOLUTE);
    bus.write(ABSOLUTE_SWAPPED, AT_NEIGHBOUR);

    for _ in 0..4 {
        cpu.step(&mut bus);
    }

    assert_eq!(cpu.registers.a, AT_TARGET, "`$FA` é `LD A,(u16)`");
    assert_eq!(cpu.registers.pc, 0x0103, "`$FA` tem 3 bytes");
}

#[test]
fn the_absolute_store_is_four_m_cycles_with_the_write_last() {
    let (mut cpu, mut bus) = machine(&[LD_IMM16_A, 0xD7, 0xC3]);
    seed_registers(&mut cpu);
    bus.write(ABSOLUTE, UNTOUCHED);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x0101, "M1: o fetch do opcode");
    assert_eq!(bus.read(ABSOLUTE), UNTOUCHED, "M1: nada escrito");

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x0102, "M2: `read(u16:lower)`");
    assert_eq!(
        bus.read(ABSOLUTE),
        UNTOUCHED,
        "M2: metade do endereço, e só"
    );

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x0103, "M3: `read(u16:upper)`");
    assert_eq!(
        bus.read(ABSOLUTE),
        UNTOUCHED,
        "M3: o endereço está inteiro dentro da CPU e a escrita ainda não \
         aconteceu — ela é o M4"
    );
    assert!(!cpu.is_between_instructions(), "M3: falta o M4");

    cpu.step(&mut bus);
    assert_eq!(bus.read(ABSOLUTE), STORED, "M4: `write(A->(u16))`");
    assert!(cpu.is_between_instructions(), "M4: e acabou");
}

#[test]
fn the_absolute_load_is_four_m_cycles_with_a_changing_last() {
    let (mut cpu, mut bus) = machine(&[LD_A_IMM16, 0xD7, 0xC3]);
    seed_registers(&mut cpu);
    seed_memory_around(&mut bus, ABSOLUTE);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x0101, "M1: o fetch do opcode");
    assert_eq!(cpu.registers.a, STORED, "M1: `A` intacto");

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x0102, "M2: `read(u16:lower)`");
    assert_eq!(cpu.registers.a, STORED, "M2: `A` intacto");

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x0103, "M3: `read(u16:upper)`");
    assert_eq!(
        cpu.registers.a, STORED,
        "M3: o endereço está inteiro e `A` ainda não mudou — a leitura é o M4"
    );
    assert!(!cpu.is_between_instructions(), "M3: falta o M4");

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, AT_TARGET, "M4: `read((u16)->A)`");
    assert!(cpu.is_between_instructions(), "M4: e acabou");
}

#[test]
fn none_of_the_six_touches_a_register_the_column_does_not_name() {
    for (opcode, operand, target, loads) in [
        (LDH_C_A, [0x00, 0x00], VIA_C, false),
        (LDH_A_C, [0x00, 0x00], VIA_C, true),
        (LDH_IMM8_A, [OFFSET_IMM8, 0x00], VIA_IMM8, false),
        (LDH_A_IMM8, [OFFSET_IMM8, 0x00], VIA_IMM8, true),
        (LD_IMM16_A, [0xD7, 0xC3], ABSOLUTE, false),
        (LD_A_IMM16, [0xD7, 0xC3], ABSOLUTE, true),
    ] {
        let (mut cpu, mut bus) = machine(&[opcode, operand[0], operand[1]]);
        seed_registers(&mut cpu);
        seed_memory_around(&mut bus, target);

        let mut expected = cpu.registers;
        expected.pc = 0x0100 + u16::from(bytes_of(opcode));
        if loads {
            expected.a = AT_TARGET;
        }

        for _ in 0..m_cycles_of(opcode) {
            cpu.step(&mut bus);
        }

        assert_eq!(
            cpu.registers, expected,
            "${opcode:02X} mexe só no que a coluna diz"
        );
    }
}

const fn bytes_of(opcode: u8) -> u8 {
    match opcode {
        LDH_C_A | LDH_A_C => 1,
        LDH_IMM8_A | LDH_A_IMM8 => 2,
        _ => 3,
    }
}

const fn m_cycles_of(opcode: u8) -> u8 {
    match opcode {
        LDH_C_A | LDH_A_C => 2,
        LDH_IMM8_A | LDH_A_IMM8 => 3,
        _ => 4,
    }
}

#[test]
fn none_of_the_six_touches_the_flags() {
    const DIRTY_F: u8 = 0b1111_1010;

    for opcode in THIS_ITEM {
        let (mut cpu, mut bus) = machine(&[opcode, OFFSET_IMM8, 0xC3]);
        seed_registers(&mut cpu);
        cpu.registers.f = DIRTY_F;

        for _ in 0..m_cycles_of(opcode) {
            cpu.step(&mut bus);
        }

        assert_eq!(
            cpu.registers.f, DIRTY_F,
            "${opcode:02X} tem `-` nas quatro colunas de flag da tabela"
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
        || matches!(opcode, 0x08 | 0xF9)
        || (0x80..=0x8F).contains(&opcode)
}

#[test]
fn the_opcodes_this_item_decodes_are_exactly_the_six_of_block_3() {
    const ILLEGAL: [u8; 11] = [
        0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
    ];

    for opcode in 0x00..=0xFFu8 {
        let (mut cpu, mut bus) = machine(&[opcode, 0x00, 0x00, 0x00]);
        seed_registers(&mut cpu);

        for _ in 0..4 {
            cpu.step(&mut bus);
        }

        if THIS_ITEM.contains(&opcode) {
            assert_eq!(
                cpu.lockup(),
                None,
                "${opcode:02X} é um dos seis do 1.4d e este item o decodifica"
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
                 tem de continuar dizendo `falta implementar`"
            );
        }
    }
}
