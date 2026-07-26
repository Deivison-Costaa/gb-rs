//! ROADMAP 1.4c — o indireto por par de registradores: `LD (r16mem),A` e

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::{Cpu, Lockup};

const ENTRY: usize = 0x0100;

const VIA_BC: u16 = 0xC0A7;
const VIA_DE: u16 = 0xC1B3;
const VIA_HL: u16 = 0xC2C9;

const STORED: u8 = 0x5A;

const UNTOUCHED: u8 = 0x6D;

const AT_TARGET: u8 = 0xE7;

const AT_NEXT: u8 = 0x3C;

const AT_PREVIOUS: u8 = 0x91;

const R16MEM: [R16Mem; 4] = [
    R16Mem::Bc,
    R16Mem::De,
    R16Mem::HlIncrement,
    R16Mem::HlDecrement,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum R16Mem {
    Bc,
    De,
    HlIncrement,
    HlDecrement,
}

impl R16Mem {
    const fn name(self) -> &'static str {
        match self {
            Self::Bc => "BC",
            Self::De => "DE",
            Self::HlIncrement => "HL+",
            Self::HlDecrement => "HL-",
        }
    }

    const fn address(self) -> u16 {
        match self {
            Self::Bc => VIA_BC,
            Self::De => VIA_DE,
            Self::HlIncrement | Self::HlDecrement => VIA_HL,
        }
    }

    const fn hl_afterwards(self) -> u16 {
        match self {
            Self::Bc | Self::De => VIA_HL,
            Self::HlIncrement => VIA_HL.wrapping_add(1),
            Self::HlDecrement => VIA_HL.wrapping_sub(1),
        }
    }
}

const fn store_through(pair: u8) -> u8 {
    0b0000_0010 | (pair << 4)
}

const fn load_through(pair: u8) -> u8 {
    0b0000_1010 | (pair << 4)
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
    cpu.registers.a = STORED;
    cpu.registers.set_bc(VIA_BC);
    cpu.registers.set_de(VIA_DE);
    cpu.registers.set_hl(VIA_HL);
}

fn seed_memory_around(bus: &mut Bus, address: u16) {
    bus.write(address.wrapping_sub(1), AT_PREVIOUS);
    bus.write(address, AT_TARGET);
    bus.write(address.wrapping_add(1), AT_NEXT);
}

#[test]
fn the_two_layouts_of_block_0_give_the_eight_opcodes_gbops_lists() {
    assert_eq!(
        [
            store_through(0),
            store_through(1),
            store_through(2),
            store_through(3)
        ],
        [0x02, 0x12, 0x22, 0x32],
        "`LD (BC),A` `LD (DE),A` `LD (HL+),A` `LD (HL-),A`"
    );
    assert_eq!(
        [
            load_through(0),
            load_through(1),
            load_through(2),
            load_through(3)
        ],
        [0x0A, 0x1A, 0x2A, 0x3A],
        "`LD A,(BC)` `LD A,(DE)` `LD A,(HL+)` `LD A,(HL-)`"
    );
}

#[test]
fn every_store_through_a_pair_writes_a_at_the_address_the_pair_holds() {
    for (index, pair) in R16MEM.iter().enumerate() {
        #[expect(clippy::cast_possible_truncation, reason = "índices de 0 a 3")]
        let opcode = store_through(index as u8);

        let (mut cpu, mut bus) = machine(&[opcode]);
        seed_registers(&mut cpu);
        bus.write(pair.address(), UNTOUCHED);

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(
            bus.read(pair.address()),
            STORED,
            "${opcode:02X} é `LD ({}),A`",
            pair.name()
        );
        assert_eq!(
            cpu.registers.a, STORED,
            "${opcode:02X} lê `A`, não escreve nele"
        );
        assert_eq!(cpu.registers.pc, 0x0101, "${opcode:02X} tem 1 byte");
    }
}

#[test]
fn a_store_through_a_pair_is_two_m_cycles_and_the_write_is_the_second() {
    let (mut cpu, mut bus) = machine(&[0x02]);
    seed_registers(&mut cpu);
    bus.write(VIA_BC, UNTOUCHED);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x0101, "o M1 é o fetch do opcode");
    assert_eq!(
        bus.read(VIA_BC),
        UNTOUCHED,
        "no M1 só o opcode chegou; nada foi escrito"
    );
    assert!(
        !cpu.is_between_instructions(),
        "`LD (BC),A` tem 8 T-cycles: uma implementação que já terminou no fetch \
         fez o acesso um M-cycle adiantado"
    );

    cpu.step(&mut bus);
    assert_eq!(bus.read(VIA_BC), STORED, "o M2 é o `write(A->(BC))`");
    assert!(
        cpu.is_between_instructions(),
        "dois M-cycles e acabou: não há terceiro"
    );
}

#[test]
fn every_load_through_a_pair_reads_into_a_from_the_address_the_pair_holds() {
    for (index, pair) in R16MEM.iter().enumerate() {
        #[expect(clippy::cast_possible_truncation, reason = "índices de 0 a 3")]
        let opcode = load_through(index as u8);

        let (mut cpu, mut bus) = machine(&[opcode]);
        seed_registers(&mut cpu);
        seed_memory_around(&mut bus, pair.address());

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(
            cpu.registers.a,
            AT_TARGET,
            "${opcode:02X} é `LD A,({})`",
            pair.name()
        );
        assert_eq!(cpu.registers.pc, 0x0101, "${opcode:02X} tem 1 byte");
    }
}

#[test]
fn a_load_through_a_pair_is_two_m_cycles_and_a_changes_on_the_second() {
    let (mut cpu, mut bus) = machine(&[0x0A]);
    seed_registers(&mut cpu);
    seed_memory_around(&mut bus, VIA_BC);

    cpu.step(&mut bus);
    assert_eq!(
        cpu.registers.a, STORED,
        "no M1 só o opcode chegou; `A` ainda é o de antes"
    );
    assert_eq!(cpu.registers.pc, 0x0101, "o M1 é o fetch do opcode");
    assert!(
        !cpu.is_between_instructions(),
        "`LD A,(BC)` tem 8 T-cycles: dois M-cycles"
    );

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.a, AT_TARGET, "o M2 é o `read((BC)->A)`");
    assert!(
        cpu.is_between_instructions(),
        "dois M-cycles e acabou: não há terceiro"
    );
}

#[test]
fn the_hl_variants_access_the_address_from_before_the_side_effect() {
    for (index, pair, expected_neighbour) in [
        (2u8, R16Mem::HlIncrement, AT_NEXT),
        (3u8, R16Mem::HlDecrement, AT_PREVIOUS),
    ] {
        let opcode = load_through(index);
        let (mut cpu, mut bus) = machine(&[opcode]);
        seed_registers(&mut cpu);
        seed_memory_around(&mut bus, VIA_HL);

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(
            cpu.registers.a,
            AT_TARGET,
            "${opcode:02X} é `LD A,({})`: o endereço é o `HL` de antes, e não \
             o vizinho ({expected_neighbour:#04X})",
            pair.name()
        );
        assert_eq!(
            cpu.registers.hl(),
            pair.hl_afterwards(),
            "${opcode:02X} deixa `HL` movido em um"
        );
    }
}

#[test]
fn the_hl_side_effect_lands_on_the_second_m_cycle_and_not_on_the_fetch() {
    for (opcode, expected) in [
        (0x22u8, VIA_HL.wrapping_add(1)),
        (0x32u8, VIA_HL.wrapping_sub(1)),
    ] {
        let (mut cpu, mut bus) = machine(&[opcode]);
        seed_registers(&mut cpu);
        bus.write(VIA_HL, UNTOUCHED);

        cpu.step(&mut bus);
        assert_eq!(
            cpu.registers.hl(),
            VIA_HL,
            "${opcode:02X}: no M1 o `HL` ainda não se moveu — o `HL±` é do M2, \
             junto com o acesso"
        );
        assert_eq!(
            bus.read(VIA_HL),
            UNTOUCHED,
            "${opcode:02X}: e nada foi escrito ainda"
        );

        cpu.step(&mut bus);
        assert_eq!(
            bus.read(VIA_HL),
            STORED,
            "${opcode:02X}: o M2 escreve no endereço de antes do efeito"
        );
        assert_eq!(
            cpu.registers.hl(),
            expected,
            "${opcode:02X}: e é o mesmo M2 que move o `HL`"
        );
    }
}

#[test]
fn only_the_hl_variants_move_their_pair() {
    for (index, pair) in R16MEM.iter().enumerate() {
        #[expect(clippy::cast_possible_truncation, reason = "índices de 0 a 3")]
        let index = index as u8;

        for (opcode, loaded) in [
            (store_through(index), None),
            (load_through(index), Some(AT_TARGET)),
        ] {
            let (mut cpu, mut bus) = machine(&[opcode]);
            seed_registers(&mut cpu);
            seed_memory_around(&mut bus, pair.address());

            let mut expected = cpu.registers;
            expected.pc = 0x0101;
            expected.set_hl(pair.hl_afterwards());
            if let Some(value) = loaded {
                expected.a = value;
            }

            cpu.step(&mut bus);
            cpu.step(&mut bus);

            assert_eq!(
                cpu.registers,
                expected,
                "${opcode:02X} usa ({}) e mexe só no que a coluna diz",
                pair.name()
            );
        }
    }
}

#[test]
fn the_side_effect_wraps_and_does_not_saturate() {
    let (mut cpu, mut bus) = machine(&[0x22]);
    seed_registers(&mut cpu);
    cpu.registers.set_hl(0xFFFF);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        bus.read(0xFFFF),
        STORED,
        "`LD (HL+),A` com `HL = $FFFF` escreve em `$FFFF`"
    );
    assert_eq!(
        cpu.registers.hl(),
        0x0000,
        "e `HL` dá a volta: depois de `$FFFF` vem `$0000`, não `$FFFF` outra vez"
    );

    let (mut cpu, mut bus) = machine(&[0x32]);
    seed_registers(&mut cpu);
    cpu.registers.set_hl(0x0000);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        cpu.registers.hl(),
        0xFFFF,
        "e no outro sentido: antes de `$0000` vem `$FFFF`, não `$0000` outra vez"
    );
}

#[test]
fn no_load_or_store_through_a_pair_touches_the_flags() {
    const DIRTY_F: u8 = 0b1111_1010;

    for index in 0..4u8 {
        for opcode in [store_through(index), load_through(index)] {
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
}

fn decoded_elsewhere(opcode: u8) -> bool {
    opcode == 0x00
        || opcode == 0xC3
        || ((0x40..=0x7F).contains(&opcode) && opcode != 0x76)
        || opcode & 0b1100_0111 == 0b0000_0110
        || opcode & 0b1100_1111 == 0b0000_0001
        || opcode & 0b1100_1111 == 0b1100_0101
        || opcode & 0b1100_1111 == 0b1100_0001
        || matches!(opcode, 0xE0 | 0xE2 | 0xEA | 0xF0 | 0xF2 | 0xFA)
}

#[test]
fn the_block_this_item_decodes_is_exactly_the_eight_opcodes_of_00_mm_x010() {
    const ILLEGAL: [u8; 11] = [
        0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
    ];

    for opcode in 0x00..=0xFFu8 {
        let (mut cpu, mut bus) = machine(&[opcode, 0x00, 0x00]);
        seed_registers(&mut cpu);

        cpu.step(&mut bus);
        cpu.step(&mut bus);
        cpu.step(&mut bus);

        let in_block = opcode & 0b1100_0111 == 0b0000_0010;

        if in_block {
            assert_eq!(
                cpu.lockup(),
                None,
                "${opcode:02X} é um dos oito `r16mem` e o 1.4c o decodifica"
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
