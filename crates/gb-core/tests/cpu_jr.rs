//! spec: docs/reference/03-opcodes.md § control/br (JR $18 $20 $28 $30 $38)
//! iteração 0039.

use gb_core::bus::Bus;
use gb_core::cart::CartridgeHeader;
use gb_core::cart::NoMbc;
use gb_core::cpu::{Cpu, Flag, Lockup};
use std::boxed::Box;

mod support;

const ENTRY: u16 = 0x0100;

fn machine(program: &[u8]) -> (Cpu, Bus) {
    let mut rom = vec![0x00; NoMbc::MAX_ROM_LEN];
    rom[ENTRY as usize..ENTRY as usize + program.len()].copy_from_slice(program);
    let checksum = CartridgeHeader::parse(&rom)
        .expect("header valido")
        .checksum();
    let cartridge = NoMbc::new(rom).expect("ROM 32KB valida");
    (Cpu::after_boot_rom(checksum), Bus::new(Box::new(cartridge)))
}

fn machine_with_rom(rom: Vec<u8>) -> (Cpu, Bus) {
    let checksum = CartridgeHeader::parse(&rom)
        .expect("header valido")
        .checksum();
    let cartridge = NoMbc::new(rom).expect("ROM 32KB valida");
    (Cpu::after_boot_rom(checksum), Bus::new(Box::new(cartridge)))
}

#[test]
fn jr_unconditional_forward_jumps_pc_by_positive_offset() {
    // $18 $05: JR +5
    // PC after reading JR i8 = ENTRY + 2 = $0102
    // target = $0102 + 5 = $0107
    let (mut cpu, mut bus) = machine(&[0x18, 0x05]);
    for _ in 0..3 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 2 + 5);
}

#[test]
fn jr_unconditional_backward_jumps_pc_by_negative_offset() {
    // $18 $FE: JR -2
    // PC after reading JR i8 = ENTRY + 2 = $0102
    // $FE as i8 = -2
    // target = $0102 - 2 = $0100
    let (mut cpu, mut bus) = machine(&[0x18, 0xFE]);
    for _ in 0..3 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY);
}

#[test]
fn jr_unconditional_zero_offset_stays_at_same_address() {
    // $18 $00: JR +0
    // target = same as PC after JR = ENTRY + 2
    let (mut cpu, mut bus) = machine(&[0x18, 0x00]);
    for _ in 0..3 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 2);
}

#[test]
fn jr_unconditional_does_not_affect_flags() {
    const DIRTY_F: u8 = 0b1010_0000;
    let (mut cpu, mut bus) = machine(&[0x18, 0x05]);
    cpu.registers.f = DIRTY_F;
    for _ in 0..3 {
        cpu.step(&mut bus);
    }
    assert_eq!(cpu.registers.f, DIRTY_F, "as flags nao podem mudar");
}

#[test]
fn jr_unconditional_takes_three_m_cycles() {
    let (mut cpu, mut bus) = machine(&[0x18, 0x05]);
    cpu.step(&mut bus); // M1: fetch
    assert!(
        !cpu.is_between_instructions(),
        "M1 nao finaliza a instrucao"
    );
    assert_eq!(cpu.registers.pc, ENTRY + 1, "M1 consumiu o opcode $18");

    cpu.step(&mut bus); // M2: read(i8)
    assert!(
        !cpu.is_between_instructions(),
        "M2 nao finaliza — JR sempre toma o desvio"
    );
    assert_eq!(cpu.registers.pc, ENTRY + 2, "M2 consumiu o offset");

    cpu.step(&mut bus); // M3: internal(modify PC)
    assert!(cpu.is_between_instructions(), "M3 finaliza a instrucao");
    assert_eq!(cpu.registers.pc, ENTRY + 2 + 5, "M3 aplicou o offset");
}

#[test]
fn jr_nz_taken_when_z_is_clear() {
    // $20 $0A: JR NZ,+10 — Z=0 toma
    let (mut cpu, mut bus) = machine(&[0x20, 0x0A]);
    cpu.registers.f &= !0b1000_0000; // Z = 0
    for _ in 0..3 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 2 + 10);
}

#[test]
fn jr_nz_not_taken_when_z_is_set() {
    // $20 $0A: JR NZ,+10 — Z=1 nao toma
    let (mut cpu, mut bus) = machine(&[0x20, 0x0A]);
    cpu.registers.f |= 0b1000_0000; // Z = 1
    for _ in 0..2 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(
        cpu.registers.pc,
        ENTRY + 2,
        "PC parou apos ler o offset, sem desvio"
    );
}

#[test]
fn jr_z_taken_when_z_is_set() {
    let (mut cpu, mut bus) = machine(&[0x28, 0x0A]);
    cpu.registers.f |= 0b1000_0000; // Z = 1
    for _ in 0..3 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 2 + 10);
}

#[test]
fn jr_z_not_taken_when_z_is_clear() {
    let (mut cpu, mut bus) = machine(&[0x28, 0x0A]);
    cpu.registers.f &= !0b1000_0000; // Z = 0
    for _ in 0..2 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 2);
}

#[test]
fn jr_nc_taken_when_c_is_clear() {
    let (mut cpu, mut bus) = machine(&[0x30, 0x0A]);
    cpu.registers.f &= !0b0001_0000; // C = 0
    for _ in 0..3 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 2 + 10);
}

#[test]
fn jr_nc_not_taken_when_c_is_set() {
    let (mut cpu, mut bus) = machine(&[0x30, 0x0A]);
    cpu.registers.f |= 0b0001_0000; // C = 1
    for _ in 0..2 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 2);
}

#[test]
fn jr_c_taken_when_c_is_set() {
    let (mut cpu, mut bus) = machine(&[0x38, 0x0A]);
    cpu.registers.f |= 0b0001_0000; // C = 1
    for _ in 0..3 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 2 + 10);
}

#[test]
fn jr_c_not_taken_when_c_is_clear() {
    let (mut cpu, mut bus) = machine(&[0x38, 0x0A]);
    cpu.registers.f &= !0b0001_0000; // C = 0
    for _ in 0..2 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 2);
}

#[test]
fn jr_conditional_takes_two_m_cycles_when_not_taken() {
    let (mut cpu, mut bus) = machine(&[0x20, 0x0A]);
    cpu.registers.f |= 0b1000_0000; // Z = 1, nao toma
    cpu.step(&mut bus); // M1: fetch
    assert!(!cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 1);

    cpu.step(&mut bus); // M2: read(i8)
    assert!(
        cpu.is_between_instructions(),
        "sem desvio, instrucao termina no M2"
    );
    assert_eq!(
        cpu.registers.pc,
        ENTRY + 2,
        "PC avancou sobre o offset mas nao desviou"
    );
}

#[test]
fn jr_conditional_takes_three_m_cycles_when_taken() {
    let (mut cpu, mut bus) = machine(&[0x20, 0x05]);
    cpu.registers.f &= !0b1000_0000; // Z = 0, toma
    cpu.step(&mut bus); // M1: fetch
    cpu.step(&mut bus); // M2: read(i8)
    assert!(
        !cpu.is_between_instructions(),
        "com desvio, M2 nao finaliza"
    );
    assert_eq!(cpu.registers.pc, ENTRY + 2);

    cpu.step(&mut bus); // M3: internal(modify PC)
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 2 + 5);
}

#[test]
fn jr_conditional_does_not_affect_flags() {
    const DIRTY_F: u8 = 0b1010_0000;
    struct Case {
        opcode: u8,
        set_z: Option<bool>,
        set_c: Option<bool>,
    }
    for case in [
        Case {
            opcode: 0x20,
            set_z: Some(false),
            set_c: None,
        }, // NZ, Z=0 toma
        Case {
            opcode: 0x20,
            set_z: Some(true),
            set_c: None,
        }, // NZ, Z=1 nao toma
        Case {
            opcode: 0x28,
            set_z: Some(true),
            set_c: None,
        }, // Z, Z=1 toma
        Case {
            opcode: 0x28,
            set_z: Some(false),
            set_c: None,
        }, // Z, Z=0 nao toma
        Case {
            opcode: 0x30,
            set_z: None,
            set_c: Some(false),
        }, // NC, C=0 toma
        Case {
            opcode: 0x30,
            set_z: None,
            set_c: Some(true),
        }, // NC, C=1 nao toma
        Case {
            opcode: 0x38,
            set_z: None,
            set_c: Some(true),
        }, // C, C=1 toma
        Case {
            opcode: 0x38,
            set_z: None,
            set_c: Some(false),
        }, // C, C=0 nao toma
    ] {
        let (mut cpu, mut bus) = machine(&[case.opcode, 0x05]);
        cpu.registers.f = DIRTY_F;
        if let Some(z_val) = case.set_z {
            cpu.registers.set_flag(Flag::Z, z_val);
        }
        if let Some(c_val) = case.set_c {
            cpu.registers.set_flag(Flag::C, c_val);
        }
        let expected_f = cpu.registers.f;
        loop {
            cpu.step(&mut bus);
            if cpu.is_between_instructions() {
                break;
            }
        }
        assert_eq!(
            cpu.registers.f, expected_f,
            "flags nao devem ser afetadas para opcode {:#04X}",
            case.opcode
        );
    }
}

#[test]
fn all_jr_opcodes_are_recognized_and_do_not_lock_up() {
    let jr_opcodes = [0x18u8, 0x20, 0x28, 0x30, 0x38];
    for &opcode in &jr_opcodes {
        let (mut cpu, mut bus) = machine(&[opcode, 0x00]);
        loop {
            cpu.step(&mut bus);
            if cpu.is_between_instructions() {
                break;
            }
        }
        assert_eq!(cpu.lockup(), None, "opcode {opcode:#04X} nao deve travar");
    }
}

#[test]
fn jr_opcodes_the_rest_of_block_is_still_undecoded_or_illegal() {
    let jr_opcodes = [0x18u8, 0x20, 0x28, 0x30, 0x38];
    const ILLEGAL: [u8; 11] = [
        0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
    ];
    for opcode in 0x00..=0xFFu8 {
        if jr_opcodes.contains(&opcode) {
            continue;
        }
        let mut rom = vec![0x00u8; NoMbc::MAX_ROM_LEN];
        rom[ENTRY as usize] = opcode;
        rom[ENTRY as usize + 1] = 0x00;
        rom[ENTRY as usize + 2] = 0x00;
        let (mut cpu, mut bus) = machine_with_rom(rom);

        for _ in 0..4 {
            cpu.step(&mut bus);
            if cpu.lockup().is_some() {
                break;
            }
            if cpu.is_between_instructions() {
                break;
            }
        }

        if ILLEGAL.contains(&opcode) {
            assert_eq!(
                cpu.lockup(),
                Some(Lockup::IllegalOpcode(opcode)),
                "opcode {opcode:#04X} deve ser ilegal"
            );
        } else if support::decoded_elsewhere(opcode) {
            assert_eq!(
                cpu.lockup(),
                None,
                "opcode {opcode:#04X} ja decodificado, nao pode travar"
            );
        } else {
            assert_eq!(
                cpu.lockup(),
                Some(Lockup::UndecodedOpcode(opcode)),
                "opcode {opcode:#04X} ainda nao decodificado"
            );
        }
    }
}

#[test]
fn jr_opcodes_now_known_in_decoded_elsewhere() {
    let jr_opcodes = [0x18u8, 0x20, 0x28, 0x30, 0x38];
    for &opcode in &jr_opcodes {
        assert!(
            support::decoded_elsewhere(opcode),
            "opcode {opcode:#04X} deve estar em decoded_elsewhere depois da implementacao"
        );
    }
}
