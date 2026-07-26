//! ROADMAP 1.7b — `ADD HL,r16`: bloco `00 rr 1001`, 4 opcodes.
//!
//! `N` = `0` literal, `H`/`C` calculados sobre o par de 16 bits (carry do bit
//! 11 e do bit 15). `Z` não é afetada. M-cycles: `fetch → internal`, como o
//! 1.7a — a metade baixa no fetch, a alta no internal.

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::{Cpu, Flag, Lockup};

mod support;
use support::decoded_elsewhere;

const ENTRY: usize = 0x0100;

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

    const fn set(self, cpu: &mut Cpu, value: u16) {
        match self {
            Self::Bc => cpu.registers.set_bc(value),
            Self::De => cpu.registers.set_de(value),
            Self::Hl => cpu.registers.set_hl(value),
            Self::Sp => cpu.registers.sp = value,
        }
    }
}

const SOURCES: [(u8, Pair); 4] = [
    (0x00, Pair::Bc),
    (0x01, Pair::De),
    (0x02, Pair::Hl),
    (0x03, Pair::Sp),
];

const fn add_hl_r16(rr: u8) -> u8 {
    0b0000_1001 | (rr << 4)
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

#[test]
fn the_block_gives_the_four_opcodes_gbops_lists() {
    let opcodes: Vec<u8> = (0..4).map(add_hl_r16).collect();
    assert_eq!(
        opcodes,
        vec![0x09, 0x19, 0x29, 0x39],
        "`00 rr 1001`: `ADD HL,BC/DE/HL/SP`"
    );
}

#[test]
fn add_adds_source_to_hl() {
    for (rr, pair) in SOURCES {
        let opcode = add_hl_r16(rr);
        let (mut cpu, mut bus) = machine(&[opcode]);
        // set destination HL first, then source — for HL=HL it's just one write
        cpu.registers.set_hl(0x1000);
        if pair as usize != Pair::Hl as usize {
            pair.set(&mut cpu, 0x1234);
        }
        let expected = if pair as usize == Pair::Hl as usize {
            // 0x1000 + 0x1000 (source = HL = 0x1000)
            0x2000_u16
        } else {
            0x1234_u16 + 0x1000
        };

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(
            cpu.registers.hl(),
            expected,
            "${opcode:02X} é `ADD HL,{}`: esperado {expected:#06X}",
            pair.name()
        );
    }
}

#[test]
fn add_wraps_past_ffff() {
    for (rr, pair) in SOURCES {
        let opcode = add_hl_r16(rr);
        let (mut cpu, mut bus) = machine(&[opcode]);
        cpu.registers.set_hl(0xFFFF);
        if pair as usize != Pair::Hl as usize {
            pair.set(&mut cpu, 0x0001);
        }

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        let expected = if pair as usize == Pair::Hl as usize {
            0xFFFF_u16.wrapping_add(0xFFFF)
        } else {
            0x0000_u16
        };
        assert_eq!(
            cpu.registers.hl(),
            expected,
            "${opcode:02X}: `ADD HL,{}`: wrapping",
            pair.name()
        );
    }
}

#[test]
fn add_sets_n_to_zero_and_leaves_z_untouched() {
    for (rr, pair) in SOURCES {
        let opcode = add_hl_r16(rr);

        for z_state in [false, true] {
            let (mut cpu, mut bus) = machine(&[opcode]);
            cpu.registers.set_hl(0x0000);
            if pair as usize != Pair::Hl as usize {
                pair.set(&mut cpu, 0x0001);
            }
            cpu.registers.set_flag(Flag::Z, z_state);

            cpu.step(&mut bus);
            cpu.step(&mut bus);

            let expected_hl = if pair as usize == Pair::Hl as usize {
                0x0000_u16
            } else {
                0x0001
            };
            assert_eq!(
                cpu.registers.hl(),
                expected_hl,
                "${opcode:02X}: a instrução executou"
            );
            assert!(
                !cpu.registers.flag(Flag::N),
                "${opcode:02X}: `N` = `0` literal na coluna de gbops"
            );
            assert_eq!(
                cpu.registers.flag(Flag::Z),
                z_state,
                "${opcode:02X}: `Z` não é tocada — a coluna é `-`"
            );
        }
    }
}

#[test]
fn add_sets_h_on_carry_from_bit_11_and_c_on_carry_from_bit_15() {
    #[allow(clippy::cast_possible_truncation)]
    fn h_flag(hl: u16, op: u16) -> bool {
        ((hl & 0x0FFF).wrapping_add(op & 0x0FFF) >> 12) != 0
    }
    fn c_flag(hl: u16, op: u16) -> bool {
        (hl as u32).wrapping_add(op as u32) > 0xFFFF
    }

    for (rr, pair) in SOURCES {
        let opcode = add_hl_r16(rr);
        let is_hl = pair as usize == Pair::Hl as usize;

        let test_cases: Vec<(u16, u16)> = vec![
            // H ativo, C inativo: estoura bit 11, não o 16
            if is_hl {
                (0x0FFF, 0x0FFF)
            } else {
                (0x0FFF, 0x0001)
            },
            // Nem H nem C
            (0x0100, if is_hl { 0x0100 } else { 0x0001 }),
            // H e C: estoura os dois
            if is_hl {
                (0x8FFF, 0x8001)
            } else {
                (0x0FFF, 0xF001)
            },
            // Nenhum (redundante por polaridade oposta)
            (0x0000, 0x0000),
        ];

        for (hl_val, src_val) in test_cases {
            let (mut cpu, mut bus) = machine(&[opcode]);
            cpu.registers.set_hl(hl_val);
            if !is_hl {
                pair.set(&mut cpu, src_val);
            }
            let effective_src = if is_hl { hl_val } else { src_val };

            cpu.step(&mut bus);
            cpu.step(&mut bus);

            assert_eq!(
                cpu.registers.flag(Flag::H),
                h_flag(hl_val, effective_src),
                "${opcode:02X}: H flag para {hl_val:#06X} + {effective_src:#06X}"
            );
            assert_eq!(
                cpu.registers.flag(Flag::C),
                c_flag(hl_val, effective_src),
                "${opcode:02X}: C flag para {hl_val:#06X} + {effective_src:#06X}"
            );
        }
    }
}

#[test]
fn each_half_of_hl_lands_on_its_own_m_cycle() {
    for (rr, pair) in SOURCES {
        let opcode = add_hl_r16(rr);
        let (mut cpu, mut bus) = machine(&[opcode]);
        cpu.registers.set_hl(0x00FF);
        if pair as usize != Pair::Hl as usize {
            pair.set(&mut cpu, 0x0100);
        }

        let expected_low = if pair as usize == Pair::Hl as usize {
            // 0x00FF + 0x00FF = 0x01FE → L = 0xFE
            0xFE_u8
        } else {
            // 0x00FF + 0x0100 = 0x01FF → L = 0xFF
            0xFF
        };
        let expected_full = if pair as usize == Pair::Hl as usize {
            0x01FE_u16
        } else {
            0x01FF
        };

        cpu.step(&mut bus);
        assert_eq!(
            cpu.registers.l, expected_low,
            "${opcode:02X}: L escrita no fetch"
        );
        assert_eq!(
            cpu.registers.h, 0x00,
            "${opcode:02X}: H ainda não foi escrita no fetch"
        );
        assert!(
            !cpu.is_between_instructions(),
            "${opcode:02X}: dois M-cycles"
        );

        cpu.step(&mut bus);
        assert_eq!(
            cpu.registers.hl(),
            expected_full,
            "${opcode:02X}: M2 escreve H"
        );
        assert!(
            cpu.is_between_instructions(),
            "${opcode:02X}: dois M-cycles e acabou"
        );
    }
}

#[test]
fn add_hl_changes_only_hl_and_flags_and_pc() {
    for (rr, pair) in SOURCES {
        let opcode = add_hl_r16(rr);
        let (mut cpu, mut bus) = machine(&[opcode]);
        cpu.registers.set_hl(0x0FFF);
        let source_value = if pair as usize == Pair::Hl as usize {
            cpu.registers.hl()
        } else {
            pair.set(&mut cpu, 0x0001);
            pair.read(&cpu)
        };
        let mut expected = cpu.registers;

        let result = 0x0FFFu16.wrapping_add(source_value);
        expected.set_hl(result);
        expected.pc = 0x0101;
        expected.set_flag(Flag::N, false);
        expected.set_flag(
            Flag::H,
            (0x0FFF_u32.wrapping_add((source_value & 0x0FFF) as u32)) >> 12 > 0,
        );
        expected.set_flag(
            Flag::C,
            0x0FFF_u32.wrapping_add(source_value as u32) > 0xFFFF,
        );

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(
            cpu.registers,
            expected,
            "${opcode:02X}: `ADD HL,{}` mexe em HL, N/H/C, e PC — mais nada",
            pair.name()
        );
    }
}

#[test]
fn add_hl_hl_doubles_hl() {
    let opcode = 0x29;
    let (mut cpu, mut bus) = machine(&[opcode]);
    cpu.registers.set_hl(0x1234);

    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        cpu.registers.hl(),
        0x2468,
        "$29 é `ADD HL,HL`: HL = HL + HL = 0x1234 + 0x1234 = 0x2468"
    );
}

#[test]
fn the_block_is_exactly_the_four_add_hl_r16_opcodes() {
    const ILLEGAL: [u8; 11] = [
        0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
    ];

    let in_this_block: Vec<u8> = (0..4).map(add_hl_r16).collect();

    for opcode in 0x00..=0xFFu8 {
        let (mut cpu, mut bus) = machine(&[opcode, 0x00, 0x00]);

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        if in_this_block.contains(&opcode) {
            assert_eq!(
                cpu.lockup(),
                None,
                "${opcode:02X} é um dos quatro `ADD HL,r16` e o 1.7b o decodifica"
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
                 tem de continuar dizendo `falta implementar`"
            );
        }
    }
}
