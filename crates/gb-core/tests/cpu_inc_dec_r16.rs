//! ROADMAP 1.7a — `INC r16` e `DEC r16`: os blocos `00 rr 0011` e `00 rr 1011`,
//! 8 opcodes.
//!
//! Primeira vez que uma ALU deixa as **quatro** colunas de flag intocadas —
//! o 1.6e fez isso só para `C`. `fetch(escreve a metade baixa) →
//! internal(escreve a metade alta)`: a coluna de gbops anota "Probably writes
//! to X here" em cada M-cycle, e não uma seta, mas o instante é o mesmo
//! princípio do 1.5a (`each_half_of_the_pair_lands_on_its_own_m_cycle`).

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::{Cpu, Lockup};

mod support;
use support::decoded_elsewhere;

const ENTRY: usize = 0x0100;

const SEED: u16 = 0x2413;
const DIRTY_F: u8 = 0b1010_0101;

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

    const fn low_half(self, cpu: &Cpu) -> u8 {
        match self {
            Self::Bc => cpu.registers.c,
            Self::De => cpu.registers.e,
            Self::Hl => cpu.registers.l,
            Self::Sp => cpu.registers.sp as u8,
        }
    }

    const fn set(self, cpu: &mut Cpu, value: u16) {
        match self {
            Self::Bc => cpu.registers.set_bc(value),
            Self::De => cpu.registers.set_de(value),
            Self::Hl => cpu.registers.set_hl(value),
            Self::Sp => cpu.registers.sp = value,
        }
    }
}

const fn inc_r16(rr: u8) -> u8 {
    0b0000_0011 | (rr << 4)
}

const fn dec_r16(rr: u8) -> u8 {
    0b0000_1011 | (rr << 4)
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

fn seed_registers(cpu: &mut Cpu) {
    cpu.registers.set_bc(SEED);
    cpu.registers.set_de(SEED);
    cpu.registers.set_hl(SEED);
    cpu.registers.sp = SEED;
}

#[test]
fn the_block_0_layout_gives_the_eight_opcodes_gbops_lists() {
    let inc: Vec<u8> = (0..4).map(inc_r16).collect();
    let dec: Vec<u8> = (0..4).map(dec_r16).collect();

    assert_eq!(
        inc,
        vec![0x03, 0x13, 0x23, 0x33],
        "`00 rr 0011`: `INC BC/DE/HL/SP`"
    );
    assert_eq!(
        dec,
        vec![0x0B, 0x1B, 0x2B, 0x3B],
        "`00 rr 1011`: `DEC BC/DE/HL/SP`, mesmo campo `rr`"
    );
}

#[test]
fn inc_adds_one_wrapping_to_the_pair() {
    for (rr, pair) in R16.iter().enumerate() {
        #[expect(clippy::cast_possible_truncation, reason = "índices de 0 a 3")]
        let opcode = inc_r16(rr as u8);
        let (mut cpu, mut bus) = machine(&[opcode]);
        seed_registers(&mut cpu);
        pair.set(&mut cpu, 0x1234);

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(
            pair.read(&cpu),
            0x1235,
            "${opcode:02X} é `INC {}`: 0x1234 + 1",
            pair.name()
        );
    }
}

#[test]
fn inc_wraps_from_ffff_to_0000() {
    for (rr, pair) in R16.iter().enumerate() {
        #[expect(clippy::cast_possible_truncation, reason = "índices de 0 a 3")]
        let opcode = inc_r16(rr as u8);
        let (mut cpu, mut bus) = machine(&[opcode]);
        seed_registers(&mut cpu);
        pair.set(&mut cpu, 0xFFFF);

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(
            pair.read(&cpu),
            0x0000,
            "${opcode:02X}: `INC {}` em 0xFFFF dá a volta para 0x0000",
            pair.name()
        );
    }
}

#[test]
fn dec_subtracts_one_wrapping_to_the_pair() {
    for (rr, pair) in R16.iter().enumerate() {
        #[expect(clippy::cast_possible_truncation, reason = "índices de 0 a 3")]
        let opcode = dec_r16(rr as u8);
        let (mut cpu, mut bus) = machine(&[opcode]);
        seed_registers(&mut cpu);
        pair.set(&mut cpu, 0x1235);

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(
            pair.read(&cpu),
            0x1234,
            "${opcode:02X} é `DEC {}`: 0x1235 - 1",
            pair.name()
        );
    }
}

#[test]
fn dec_wraps_from_0000_to_ffff() {
    for (rr, pair) in R16.iter().enumerate() {
        #[expect(clippy::cast_possible_truncation, reason = "índices de 0 a 3")]
        let opcode = dec_r16(rr as u8);
        let (mut cpu, mut bus) = machine(&[opcode]);
        seed_registers(&mut cpu);
        pair.set(&mut cpu, 0x0000);

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(
            pair.read(&cpu),
            0xFFFF,
            "${opcode:02X}: `DEC {}` em 0x0000 dá a volta para 0xFFFF",
            pair.name()
        );
    }
}

// O ponto inteiro do 1.7a: as quatro colunas são `-`. Os dois casos por
// direção distinguem "intocado" de "calculado como se fosse ADD/SUB de
// verdade": o par inteiro estourando (um F "genérico" mexeria) e o par sem
// estouro nenhum, com F sujo dos dois lados.
#[test]
fn inc_and_dec_of_a_pair_leave_every_flag_untouched() {
    for (rr, pair) in R16.iter().enumerate() {
        #[expect(clippy::cast_possible_truncation, reason = "índices de 0 a 3")]
        let rr = rr as u8;

        for (opcode, seed_value) in [(inc_r16(rr), 0xFFFFu16), (inc_r16(rr), 0x0001)] {
            let (mut cpu, mut bus) = machine(&[opcode]);
            seed_registers(&mut cpu);
            cpu.registers.f = DIRTY_F;
            pair.set(&mut cpu, seed_value);

            cpu.step(&mut bus);
            cpu.step(&mut bus);

            assert_eq!(
                cpu.registers.f,
                DIRTY_F,
                "${opcode:02X}: `INC {}` de {seed_value:#06X} não toca em `F`",
                pair.name()
            );
        }

        for (opcode, seed_value) in [(dec_r16(rr), 0x0000u16), (dec_r16(rr), 0xFFFE)] {
            let (mut cpu, mut bus) = machine(&[opcode]);
            seed_registers(&mut cpu);
            cpu.registers.f = DIRTY_F;
            pair.set(&mut cpu, seed_value);

            cpu.step(&mut bus);
            cpu.step(&mut bus);

            assert_eq!(
                cpu.registers.f,
                DIRTY_F,
                "${opcode:02X}: `DEC {}` de {seed_value:#06X} não toca em `F`",
                pair.name()
            );
        }
    }
}

#[test]
fn each_half_of_the_pair_lands_on_its_own_m_cycle() {
    for (rr, pair) in R16.iter().enumerate() {
        #[expect(clippy::cast_possible_truncation, reason = "índices de 0 a 3")]
        let opcode = inc_r16(rr as u8);
        let name = pair.name();

        let (mut cpu, mut bus) = machine(&[opcode]);
        seed_registers(&mut cpu);
        pair.set(&mut cpu, 0x00FF);

        cpu.step(&mut bus);
        assert_eq!(
            pair.low_half(&cpu),
            0x00,
            "${opcode:02X}: a coluna anota \"Probably writes to the low half \
             here\" no fetch — `{name}` de baixo já é 0x00"
        );
        assert_eq!(
            pair.read(&cpu),
            0x0000,
            "${opcode:02X}: a metade alta ainda não foi escrita no fetch — se \
             o par inteiro já lesse 0x0100 aqui, o M2 (`internal`) não faria \
             nada e o teste de cima não pegaria, porque a metade alta de \
             0x00FF já é 0x00 por acidente do valor semeado"
        );
        assert_eq!(cpu.registers.pc, 0x0101, "${opcode:02X}: o M1 é o fetch");
        assert!(
            !cpu.is_between_instructions(),
            "${opcode:02X} tem 8 T-cycles: dois M-cycles"
        );

        cpu.step(&mut bus);
        assert_eq!(
            pair.read(&cpu),
            0x0100,
            "${opcode:02X}: o M2 (`internal`) escreve a metade alta — o par \
             carrega de 0x00FF para 0x0100"
        );
        assert!(
            cpu.is_between_instructions(),
            "${opcode:02X}: dois M-cycles e acabou"
        );
    }
}

#[test]
fn inc_dec_changes_only_its_own_pair_and_the_program_counter() {
    for (rr, pair) in R16.iter().enumerate() {
        #[expect(clippy::cast_possible_truncation, reason = "índices de 0 a 3")]
        let rr = rr as u8;

        for opcode in [inc_r16(rr), dec_r16(rr)] {
            let (mut cpu, mut bus) = machine(&[opcode]);
            seed_registers(&mut cpu);

            let mut expected = cpu.registers;
            expected.pc = 0x0101;
            let new_value = if opcode == inc_r16(rr) {
                SEED.wrapping_add(1)
            } else {
                SEED.wrapping_sub(1)
            };
            match pair {
                Pair::Bc => expected.set_bc(new_value),
                Pair::De => expected.set_de(new_value),
                Pair::Hl => expected.set_hl(new_value),
                Pair::Sp => expected.sp = new_value,
            }

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
}

#[test]
fn the_fourth_pair_of_r16_is_sp_and_not_af() {
    let (mut cpu, mut bus) = machine(&[inc_r16(3)]);
    seed_registers(&mut cpu);
    let af_before = cpu.registers.af();

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        cpu.registers.sp,
        SEED.wrapping_add(1),
        "$33 é `INC SP`: o índice 3 do placeholder `r16` é `sp`"
    );
    assert_eq!(
        cpu.registers.af(),
        af_before,
        "`af` é o índice 3 do `r16stk` (o do `PUSH`/`POP`), que é outra tabela"
    );
}

#[test]
fn the_block_this_item_decodes_is_exactly_the_eight_opcodes_of_00_rr_0011_and_00_rr_1011() {
    const ILLEGAL: [u8; 11] = [
        0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
    ];

    let in_this_block: Vec<u8> = (0..4).map(inc_r16).chain((0..4).map(dec_r16)).collect();

    for opcode in 0x00..=0xFFu8 {
        let (mut cpu, mut bus) = machine(&[opcode, 0x00, 0x00]);
        seed_registers(&mut cpu);

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        if in_this_block.contains(&opcode) {
            assert_eq!(
                cpu.lockup(),
                None,
                "${opcode:02X} é um dos oito `INC`/`DEC r16` e o 1.7a o decodifica"
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
                 tem de continuar dizendo `falta implementar`. Uma máscara \
                 frouxa aqui leva junto `ADD HL,r16` (`00 rr 1001`), que é o 1.7b"
            );
        }
    }
}
