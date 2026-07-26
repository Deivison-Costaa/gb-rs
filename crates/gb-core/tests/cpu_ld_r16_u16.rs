//! ROADMAP 1.5a — `LD r16,u16`: o bloco `00 rr 0001`.

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::{Cpu, Lockup};

const ENTRY: usize = 0x0100;

const IMMEDIATE: u16 = 0x9C57;
const IMMEDIATE_BYTES: [u8; 2] = IMMEDIATE.to_le_bytes();
const IMMEDIATE_SWAPPED: u16 = IMMEDIATE.swap_bytes();

const SEED: u16 = 0x2413;
const HALF_SEEDED: u16 = (SEED & 0xFF00) | IMMEDIATE & 0x00FF;

const R16: [Pair; 4] = [Pair::Bc, Pair::De, Pair::Hl, Pair::Sp];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pair {
    Bc,
    De,
    Hl,
    Sp,
}

impl Pair {
    const fn name(self) -> &'static str {
        match self {
            Self::Bc => "BC",
            Self::De => "DE",
            Self::Hl => "HL",
            Self::Sp => "SP",
        }
    }

    const fn read(self, cpu: &Cpu) -> u16 {
        match self {
            Self::Bc => cpu.registers.bc(),
            Self::De => cpu.registers.de(),
            Self::Hl => cpu.registers.hl(),
            Self::Sp => cpu.registers.sp,
        }
    }
}

const fn load_pair(rr: u8) -> u8 {
    0b0000_0001 | (rr << 4)
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
    cpu.registers.set_bc(SEED);
    cpu.registers.set_de(SEED);
    cpu.registers.set_hl(SEED);
    cpu.registers.sp = SEED;
}

fn program_for(rr: u8) -> [u8; 3] {
    [load_pair(rr), IMMEDIATE_BYTES[0], IMMEDIATE_BYTES[1]]
}

#[test]
fn the_block_0_layout_gives_the_four_opcodes_gbops_lists() {
    assert_eq!(
        [load_pair(0), load_pair(1), load_pair(2), load_pair(3)],
        [0x01, 0x11, 0x21, 0x31],
        "`LD BC,u16` `LD DE,u16` `LD HL,u16` `LD SP,u16`"
    );
}

#[test]
fn every_pair_receives_the_two_operand_bytes_in_little_endian_order() {
    for (index, pair) in R16.iter().enumerate() {
        #[expect(clippy::cast_possible_truncation, reason = "índices de 0 a 3")]
        let rr = index as u8;
        let opcode = load_pair(rr);

        let (mut cpu, mut bus) = machine(&program_for(rr));
        seed_registers(&mut cpu);

        cpu.step(&mut bus);
        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(
            pair.read(&cpu),
            IMMEDIATE,
            "${opcode:02X} é `LD {},u16`: o byte de baixo vem primeiro no fluxo \
             de instruções e vai para a metade baixa do par — {IMMEDIATE_SWAPPED:#06X} \
             seria a leitura trocada",
            pair.name()
        );
        assert_eq!(cpu.registers.pc, 0x0103, "${opcode:02X} tem 3 bytes");
    }
}

#[test]
fn each_half_of_the_pair_lands_on_its_own_m_cycle() {
    for (index, pair) in R16.iter().enumerate() {
        #[expect(clippy::cast_possible_truncation, reason = "índices de 0 a 3")]
        let rr = index as u8;
        let opcode = load_pair(rr);
        let name = pair.name();

        let (mut cpu, mut bus) = machine(&program_for(rr));
        seed_registers(&mut cpu);

        cpu.step(&mut bus);
        assert_eq!(
            pair.read(&cpu),
            SEED,
            "${opcode:02X}: no M1 só o opcode chegou; `{name}` ainda é o de antes"
        );
        assert_eq!(cpu.registers.pc, 0x0101, "${opcode:02X}: o M1 é o fetch");
        assert!(
            !cpu.is_between_instructions(),
            "${opcode:02X} tem 12 T-cycles: três M-cycles"
        );

        cpu.step(&mut bus);
        assert_eq!(
            pair.read(&cpu),
            HALF_SEEDED,
            "${opcode:02X}: a coluna é `read(u16:lower->{name}:lower)` — a metade \
             baixa já está no par depois do M2, e a alta ainda não. Latchar os \
             dois bytes e escrever o par inteiro no M3 dá o mesmo estado final \
             e os mesmos 12 T-cycles"
        );
        assert_eq!(cpu.registers.pc, 0x0102, "${opcode:02X}: o M2 leu um byte");
        assert!(
            !cpu.is_between_instructions(),
            "${opcode:02X}: ainda falta o M3"
        );

        cpu.step(&mut bus);
        assert_eq!(
            pair.read(&cpu),
            IMMEDIATE,
            "${opcode:02X}: o M3 é o `read(u16:upper->{name}:upper)`"
        );
        assert_eq!(
            cpu.registers.pc, 0x0103,
            "${opcode:02X}: e o `PC` andou três"
        );
        assert!(
            cpu.is_between_instructions(),
            "${opcode:02X}: três M-cycles e acabou"
        );
    }
}

#[test]
fn the_fourth_pair_of_r16_is_sp_and_not_af() {
    let (mut cpu, mut bus) = machine(&program_for(3));
    seed_registers(&mut cpu);
    let af_before = cpu.registers.af();

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        cpu.registers.sp, IMMEDIATE,
        "$31 é `LD SP,u16`: o índice 3 do placeholder `r16` é `sp`"
    );
    assert_eq!(
        cpu.registers.af(),
        af_before,
        "`af` é o índice 3 do `r16stk` (o do `PUSH`/`POP`), que é outra tabela"
    );
}

#[test]
fn an_immediate_pair_load_changes_only_its_own_pair() {
    for (index, pair) in R16.iter().enumerate() {
        #[expect(clippy::cast_possible_truncation, reason = "índices de 0 a 3")]
        let rr = index as u8;
        let opcode = load_pair(rr);

        let (mut cpu, mut bus) = machine(&program_for(rr));
        seed_registers(&mut cpu);

        let mut expected = cpu.registers;
        expected.pc = 0x0103;
        match pair {
            Pair::Bc => expected.set_bc(IMMEDIATE),
            Pair::De => expected.set_de(IMMEDIATE),
            Pair::Hl => expected.set_hl(IMMEDIATE),
            Pair::Sp => expected.sp = IMMEDIATE,
        }

        cpu.step(&mut bus);
        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(
            cpu.registers,
            expected,
            "${opcode:02X} mexe em `{}` e no `PC`, e em mais nada",
            pair.name()
        );
    }
}

#[test]
fn no_16_bit_immediate_load_touches_the_flags() {
    const DIRTY_F: u8 = 0b1111_1010;

    for rr in 0..4u8 {
        let opcode = load_pair(rr);
        let (mut cpu, mut bus) = machine(&program_for(rr));
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
fn an_immediate_pair_load_reads_both_operand_bytes_across_the_program_counter_wrap() {
    let mut rom = rom_with(&[]);
    rom[0x0000] = IMMEDIATE_BYTES[0];
    rom[0x0001] = IMMEDIATE_BYTES[1];
    let (mut cpu, mut bus) = machine_with_rom(rom);
    seed_registers(&mut cpu);

    bus.write(0xFFFF, load_pair(0));
    cpu.registers.pc = 0xFFFF;

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        cpu.registers.bc(),
        IMMEDIATE,
        "os dois bytes de `LD BC,u16` são os seguintes ao opcode, e depois de \
         `$FFFF` vêm `$0000` e `$0001`"
    );
    assert_eq!(cpu.registers.pc, 0x0002, "e o `PC` continua andando dali");
}

fn decoded_elsewhere(opcode: u8) -> bool {
    opcode == 0x00
        || opcode == 0xC3
        || ((0x40..=0x7F).contains(&opcode) && opcode != 0x76)
        || opcode & 0b1100_0111 == 0b0000_0110
        || opcode & 0b1100_0111 == 0b0000_0010
        || opcode & 0b1100_1111 == 0b1100_0101
        || opcode & 0b1100_1111 == 0b1100_0001
        || matches!(
            opcode,
            0x08 | 0xE0 | 0xE2 | 0xEA | 0xF0 | 0xF2 | 0xF9 | 0xFA
        )
}

#[test]
fn the_block_this_item_decodes_is_exactly_the_four_opcodes_of_00_rr_0001() {
    const ILLEGAL: [u8; 11] = [
        0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
    ];

    for opcode in 0x00..=0xFFu8 {
        let (mut cpu, mut bus) = machine(&[opcode, 0x00, 0x00]);
        seed_registers(&mut cpu);

        cpu.step(&mut bus);
        cpu.step(&mut bus);
        cpu.step(&mut bus);

        let in_block = opcode & 0b1100_1111 == 0b0000_0001;

        if in_block {
            assert_eq!(
                cpu.lockup(),
                None,
                "${opcode:02X} é um dos quatro `LD r16,u16` e o 1.5a o decodifica"
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
                 tem de continuar dizendo `falta implementar`. Uma máscara \
                 frouxa aqui leva junto `INC r16` (`00 rr 0011`), \
                 `ADD HL,r16` (`00 rr 1001`) e `DEC r16` (`00 rr 1011`)"
            );
        }
    }
}
