//! spec: docs/reference/03-opcodes.md § control/br (JP $C2 $C3 $CA $D2 $DA $E9)
//! iteracao 0040.

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

// --- JP HL ($E9) ---

#[test]
fn jp_hl_copies_hl_to_pc_in_one_m_cycle() {
    let (mut cpu, mut bus) = machine(&[0xE9]);
    cpu.registers.set_hl(0x3456);
    cpu.step(&mut bus);
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, 0x3456);
}

#[test]
fn jp_hl_does_not_affect_flags() {
    const DIRTY_F: u8 = 0b1010_0000;
    let (mut cpu, mut bus) = machine(&[0xE9]);
    cpu.registers.f = DIRTY_F;
    cpu.registers.set_hl(0x1234);
    cpu.step(&mut bus);
    assert_eq!(cpu.registers.f, DIRTY_F);
}

#[test]
fn jp_hl_consumes_one_m_cycle() {
    let (mut cpu, mut bus) = machine(&[0xE9]);
    cpu.registers.set_hl(0x5678);
    cpu.step(&mut bus);
    assert!(cpu.is_between_instructions(), "JP HL termina no M1");
}

// --- JP u16 ($C3) regressao ---

#[test]
fn jp_u16_still_jumps_to_immediate_address() {
    let target = 0x0456u16;
    let (mut cpu, mut bus) = machine(&[0xC3, target as u8, (target >> 8) as u8]);
    for _ in 0..4 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, target);
}

#[test]
fn jp_u16_does_not_affect_flags() {
    const DIRTY_F: u8 = 0b1010_0000;
    let (mut cpu, mut bus) = machine(&[0xC3, 0x34, 0x12]);
    cpu.registers.f = DIRTY_F;
    for _ in 0..4 {
        cpu.step(&mut bus);
    }
    assert_eq!(cpu.registers.f, DIRTY_F);
}

#[test]
fn jp_u16_takes_four_m_cycles() {
    let (mut cpu, mut bus) = machine(&[0xC3, 0x34, 0x12]);
    cpu.step(&mut bus); // M1: fetch
    assert!(!cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 1);

    cpu.step(&mut bus); // M2: read(u16:lower)
    assert!(!cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 2);

    cpu.step(&mut bus); // M3: read(u16:upper)
    assert!(!cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 3);

    cpu.step(&mut bus); // M4: internal(set PC)
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, 0x1234);
}

// --- JP NZ,u16 ($C2) ---

#[test]
fn jp_nz_taken_when_z_is_clear() {
    let target = 0x0500u16;
    let (mut cpu, mut bus) = machine(&[0xC2, target as u8, (target >> 8) as u8]);
    cpu.registers.f &= !0b1000_0000;
    for _ in 0..4 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, target);
}

#[test]
fn jp_nz_not_taken_when_z_is_set() {
    let (mut cpu, mut bus) = machine(&[0xC2, 0x00, 0x05]);
    cpu.registers.f |= 0b1000_0000;
    for _ in 0..3 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(
        cpu.registers.pc,
        ENTRY + 3,
        "PC passou pelo operando sem desviar"
    );
}

// --- JP Z,u16 ($CA) ---

#[test]
fn jp_z_taken_when_z_is_set() {
    let target = 0x0500u16;
    let (mut cpu, mut bus) = machine(&[0xCA, target as u8, (target >> 8) as u8]);
    cpu.registers.f |= 0b1000_0000;
    for _ in 0..4 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, target);
}

#[test]
fn jp_z_not_taken_when_z_is_clear() {
    let (mut cpu, mut bus) = machine(&[0xCA, 0x00, 0x05]);
    cpu.registers.f &= !0b1000_0000;
    for _ in 0..3 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 3);
}

// --- JP NC,u16 ($D2) ---

#[test]
fn jp_nc_taken_when_c_is_clear() {
    let target = 0x0500u16;
    let (mut cpu, mut bus) = machine(&[0xD2, target as u8, (target >> 8) as u8]);
    cpu.registers.f &= !0b0001_0000;
    for _ in 0..4 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, target);
}

#[test]
fn jp_nc_not_taken_when_c_is_set() {
    let (mut cpu, mut bus) = machine(&[0xD2, 0x00, 0x05]);
    cpu.registers.f |= 0b0001_0000;
    for _ in 0..3 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 3);
}

// --- JP C,u16 ($DA) ---

#[test]
fn jp_c_taken_when_c_is_set() {
    let target = 0x0500u16;
    let (mut cpu, mut bus) = machine(&[0xDA, target as u8, (target >> 8) as u8]);
    cpu.registers.f |= 0b0001_0000;
    for _ in 0..4 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, target);
}

#[test]
fn jp_c_not_taken_when_c_is_clear() {
    let (mut cpu, mut bus) = machine(&[0xDA, 0x00, 0x05]);
    cpu.registers.f &= !0b0001_0000;
    for _ in 0..3 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 3);
}

// --- timing condicional: JP cc,u16 ---

#[test]
fn jp_conditional_takes_three_m_cycles_when_not_taken() {
    let (mut cpu, mut bus) = machine(&[0xC2, 0x00, 0x05]);
    cpu.registers.f |= 0b1000_0000; // Z=1, NZ nao toma
    cpu.step(&mut bus); // M1: fetch
    assert!(!cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 1);

    cpu.step(&mut bus); // M2: read(u16:lower)
    assert!(!cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 2);

    cpu.step(&mut bus); // M3: read(u16:upper), sem desvio
    assert!(cpu.is_between_instructions(), "sem desvio termina no M3");
    assert_eq!(cpu.registers.pc, ENTRY + 3);
}

#[test]
fn jp_conditional_takes_four_m_cycles_when_taken() {
    let (mut cpu, mut bus) = machine(&[0xC2, 0x34, 0x12]);
    cpu.registers.f &= !0b1000_0000; // Z=0, NZ toma
    cpu.step(&mut bus); // M1: fetch
    cpu.step(&mut bus); // M2: read(u16:lower)
    cpu.step(&mut bus); // M3: read(u16:upper)
    assert!(!cpu.is_between_instructions(), "com desvio M3 nao finaliza");
    assert_eq!(cpu.registers.pc, ENTRY + 3);

    cpu.step(&mut bus); // M4: internal(set PC)
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, 0x1234);
}

// --- flags ---

#[test]
fn jp_conditional_does_not_affect_flags() {
    const DIRTY_F: u8 = 0b1010_0000;
    for condition_opcode in [0xC2u8, 0xCA, 0xD2, 0xDA] {
        let (mut cpu, mut bus) = machine(&[condition_opcode, 0x05, 0x00]);
        cpu.registers.f = DIRTY_F;
        let expected_f = cpu.registers.f;
        // NZ toma com Z=0, Z toma com Z=1, NC toma com C=0, C toma com C=1
        match condition_opcode {
            0xC2 => cpu.registers.set_flag(Flag::Z, false),
            0xCA => cpu.registers.set_flag(Flag::Z, true),
            0xD2 => cpu.registers.set_flag(Flag::C, false),
            _ => cpu.registers.set_flag(Flag::C, true),
        }
        cpu.registers.f = expected_f;
        loop {
            cpu.step(&mut bus);
            if cpu.is_between_instructions() || cpu.lockup().is_some() {
                break;
            }
        }
        assert_eq!(
            cpu.registers.f, expected_f,
            "flags nao devem ser afetadas para opcode {condition_opcode:#04X}"
        );
    }
}

// --- todos os opcodes JP reconhecidos ---

#[test]
fn all_jp_opcodes_are_recognized_and_do_not_lock_up() {
    let jp_opcodes = [0xC2u8, 0xC3, 0xCA, 0xD2, 0xDA, 0xE9];
    for &opcode in &jp_opcodes {
        let (mut cpu, mut bus) = machine(&[opcode, 0x00, 0x05]);
        loop {
            cpu.step(&mut bus);
            if cpu.is_between_instructions() || cpu.lockup().is_some() {
                break;
            }
        }
        assert_eq!(cpu.lockup(), None, "opcode {opcode:#04X} nao deve travar");
    }
}

// --- controle negativo ---

#[test]
fn jp_opcodes_the_rest_of_block_is_still_undecoded_or_illegal() {
    let jp_opcodes = [0xC2u8, 0xC3, 0xCA, 0xD2, 0xDA, 0xE9];
    const ILLEGAL: [u8; 11] = [
        0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
    ];
    for opcode in 0x00..=0xFFu8 {
        if jp_opcodes.contains(&opcode) {
            continue;
        }
        let mut rom = vec![0x00u8; NoMbc::MAX_ROM_LEN];
        rom[ENTRY as usize] = opcode;
        rom[ENTRY as usize + 1] = 0x00;
        rom[ENTRY as usize + 2] = 0x00;
        rom[ENTRY as usize + 3] = 0x00;
        let (mut cpu, mut bus) = machine_with_rom(rom);

        for _ in 0..5 {
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
fn jp_opcodes_now_known_in_decoded_elsewhere() {
    let jp_opcodes = [0xC2u8, 0xCA, 0xD2, 0xDA, 0xE9];
    for &opcode in &jp_opcodes {
        assert!(
            support::decoded_elsewhere(opcode),
            "opcode {opcode:#04X} deve estar em decoded_elsewhere depois da implementacao"
        );
    }
}
