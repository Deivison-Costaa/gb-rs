//! spec: docs/reference/03-opcodes.md § control/br (RET $C0 $C8 $C9 $D0 $D8 $D9)
//! iteracao 0042.

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

fn machine_with_stack(program: &[u8], sp: u16, ret_addr: u16) -> (Cpu, Bus) {
    let (mut cpu, mut bus) = machine(program);
    cpu.registers.sp = sp;
    bus.write(sp, (ret_addr & 0xFF) as u8);
    bus.write(sp.wrapping_add(1), (ret_addr >> 8) as u8);
    (cpu, bus)
}

fn initial_sp(cpu: &Cpu) -> u16 {
    cpu.registers.sp
}

// --- RET ($C9) ---

#[test]
fn ret_pops_return_address_from_stack_and_jumps() {
    let ret_addr = 0x0500u16;
    let (mut cpu, mut bus) = machine_with_stack(&[0xC9], 0xC000, ret_addr);
    let sp_antes = initial_sp(&cpu);
    for _ in 0..4 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ret_addr);
    assert_eq!(cpu.registers.sp, sp_antes.wrapping_add(2));
}

#[test]
fn ret_does_not_affect_flags() {
    const DIRTY_F: u8 = 0b1010_0000;
    let (mut cpu, mut bus) = machine_with_stack(&[0xC9], 0xC000, 0x0500);
    cpu.registers.f = DIRTY_F;
    loop {
        cpu.step(&mut bus);
        if cpu.is_between_instructions() || cpu.lockup().is_some() {
            break;
        }
    }
    assert_eq!(cpu.registers.f, DIRTY_F);
}

#[test]
fn ret_takes_four_m_cycles() {
    let ret_addr = 0x0500u16;
    let (mut cpu, mut bus) = machine_with_stack(&[0xC9], 0xC000, ret_addr);
    let sp_antes = initial_sp(&cpu);

    cpu.step(&mut bus); // M1: fetch
    assert!(!cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 1);

    cpu.step(&mut bus); // M2: read((SP++)->lower)
    assert!(!cpu.is_between_instructions());
    assert_eq!(
        cpu.registers.sp,
        sp_antes.wrapping_add(1),
        "SP++ apos leitura do low"
    );

    cpu.step(&mut bus); // M3: read((SP++)->upper)
    assert!(!cpu.is_between_instructions());
    assert_eq!(
        cpu.registers.sp,
        sp_antes.wrapping_add(2),
        "SP++ apos leitura do high"
    );

    cpu.step(&mut bus); // M4: internal(set PC)
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ret_addr);
    assert_eq!(cpu.registers.sp, sp_antes.wrapping_add(2));
}

#[test]
fn ret_pops_low_byte_first_then_high_byte() {
    let ret_addr = 0xABCDu16;
    let (mut cpu, mut bus) = machine_with_stack(&[0xC9], 0xC000, ret_addr);
    let sp_antes = initial_sp(&cpu);

    cpu.step(&mut bus); // M1: fetch
    cpu.step(&mut bus); // M2: read low byte from (SP), SP++
    // apos M2, o latch tem o byte baixo
    cpu.step(&mut bus); // M3: read high byte from (SP), SP++
    cpu.step(&mut bus); // M4: set PC
    assert_eq!(cpu.registers.pc, ret_addr);
    assert_eq!(cpu.registers.sp, sp_antes.wrapping_add(2));
}

#[test]
fn ret_with_sp_at_wram_boundary_increments_correctly() {
    let (mut cpu, mut bus) = machine_with_stack(&[0xC9], 0xDFFD, 0x0300);
    let sp_antes = initial_sp(&cpu);
    for _ in 0..4 {
        cpu.step(&mut bus);
    }
    assert_eq!(cpu.registers.pc, 0x0300);
    assert_eq!(cpu.registers.sp, sp_antes.wrapping_add(2));
}

// --- RET NZ ($C0) ---

#[test]
fn ret_nz_taken_when_z_is_clear() {
    let ret_addr = 0x0500u16;
    let (mut cpu, mut bus) = machine_with_stack(&[0xC0], 0xC000, ret_addr);
    cpu.registers.f &= !0b1000_0000;
    let sp_antes = initial_sp(&cpu);
    for _ in 0..5 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ret_addr);
    assert_eq!(cpu.registers.sp, sp_antes.wrapping_add(2));
}

#[test]
fn ret_nz_not_taken_when_z_is_set() {
    let (mut cpu, mut bus) = machine_with_stack(&[0xC0], 0xC000, 0x0500);
    cpu.registers.f |= 0b1000_0000;
    let sp_antes = initial_sp(&cpu);
    for _ in 0..2 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 1);
    assert_eq!(cpu.registers.sp, sp_antes, "SP nao muda sem desvio");
}

// --- RET Z ($C8) ---

#[test]
fn ret_z_taken_when_z_is_set() {
    let ret_addr = 0x0500u16;
    let (mut cpu, mut bus) = machine_with_stack(&[0xC8], 0xC000, ret_addr);
    cpu.registers.f |= 0b1000_0000;
    let sp_antes = initial_sp(&cpu);
    for _ in 0..5 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ret_addr);
    assert_eq!(cpu.registers.sp, sp_antes.wrapping_add(2));
}

#[test]
fn ret_z_not_taken_when_z_is_clear() {
    let (mut cpu, mut bus) = machine_with_stack(&[0xC8], 0xC000, 0x0500);
    cpu.registers.f &= !0b1000_0000;
    let sp_antes = initial_sp(&cpu);
    for _ in 0..2 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 1);
    assert_eq!(cpu.registers.sp, sp_antes, "SP nao muda sem desvio");
}

// --- RET NC ($D0) ---

#[test]
fn ret_nc_taken_when_c_is_clear() {
    let ret_addr = 0x0500u16;
    let (mut cpu, mut bus) = machine_with_stack(&[0xD0], 0xC000, ret_addr);
    cpu.registers.f &= !0b0001_0000;
    let sp_antes = initial_sp(&cpu);
    for _ in 0..5 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ret_addr);
    assert_eq!(cpu.registers.sp, sp_antes.wrapping_add(2));
}

#[test]
fn ret_nc_not_taken_when_c_is_set() {
    let (mut cpu, mut bus) = machine_with_stack(&[0xD0], 0xC000, 0x0500);
    cpu.registers.f |= 0b0001_0000;
    let sp_antes = initial_sp(&cpu);
    for _ in 0..2 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 1);
    assert_eq!(cpu.registers.sp, sp_antes, "SP nao muda sem desvio");
}

// --- RET C ($D8) ---

#[test]
fn ret_c_taken_when_c_is_set() {
    let ret_addr = 0x0500u16;
    let (mut cpu, mut bus) = machine_with_stack(&[0xD8], 0xC000, ret_addr);
    cpu.registers.f |= 0b0001_0000;
    let sp_antes = initial_sp(&cpu);
    for _ in 0..5 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ret_addr);
    assert_eq!(cpu.registers.sp, sp_antes.wrapping_add(2));
}

#[test]
fn ret_c_not_taken_when_c_is_clear() {
    let (mut cpu, mut bus) = machine_with_stack(&[0xD8], 0xC000, 0x0500);
    cpu.registers.f &= !0b0001_0000;
    let sp_antes = initial_sp(&cpu);
    for _ in 0..2 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 1);
    assert_eq!(cpu.registers.sp, sp_antes, "SP nao muda sem desvio");
}

// --- timing condicional ---

#[test]
fn ret_conditional_takes_two_m_cycles_when_not_taken() {
    let (mut cpu, mut bus) = machine_with_stack(&[0xC0], 0xC000, 0x0500);
    cpu.registers.f |= 0b1000_0000; // Z=1, NZ nao toma
    let sp_antes = initial_sp(&cpu);

    cpu.step(&mut bus); // M1: fetch
    assert!(!cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 1);

    cpu.step(&mut bus); // M2: internal, sem desvio
    assert!(cpu.is_between_instructions(), "sem desvio termina no M2");
    assert_eq!(cpu.registers.pc, ENTRY + 1);
    assert_eq!(cpu.registers.sp, sp_antes, "SP nao muda sem desvio");
}

#[test]
fn ret_conditional_takes_five_m_cycles_when_taken() {
    let ret_addr = 0x0500u16;
    let (mut cpu, mut bus) = machine_with_stack(&[0xC0], 0xC000, ret_addr);
    cpu.registers.f &= !0b1000_0000; // Z=0, NZ toma
    let sp_antes = initial_sp(&cpu);

    cpu.step(&mut bus); // M1: fetch
    assert!(!cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 1);

    cpu.step(&mut bus); // M2: internal, com desvio
    assert!(!cpu.is_between_instructions(), "com desvio M2 nao finaliza");
    assert_eq!(cpu.registers.pc, ENTRY + 1);

    cpu.step(&mut bus); // M3: read((SP++)->lower)
    assert!(!cpu.is_between_instructions());
    assert_eq!(cpu.registers.sp, sp_antes.wrapping_add(1));

    cpu.step(&mut bus); // M4: read((SP++)->upper)
    assert!(!cpu.is_between_instructions());
    assert_eq!(cpu.registers.sp, sp_antes.wrapping_add(2));

    cpu.step(&mut bus); // M5: internal(set PC)
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ret_addr);
    assert_eq!(cpu.registers.sp, sp_antes.wrapping_add(2));
}

// --- flags (todos os opcodes) ---

#[test]
fn ret_conditional_does_not_affect_flags() {
    const DIRTY_F: u8 = 0b1010_0000;
    for opcode in [0xC0u8, 0xC8, 0xD0, 0xD8, 0xC9, 0xD9] {
        let (mut cpu, mut bus) = machine_with_stack(&[opcode], 0xC000, 0x0500);
        cpu.registers.f = DIRTY_F;
        match opcode {
            0xC0 => cpu.registers.set_flag(Flag::Z, false),
            0xC8 => cpu.registers.set_flag(Flag::Z, true),
            0xD0 => cpu.registers.set_flag(Flag::C, false),
            0xD8 => cpu.registers.set_flag(Flag::C, true),
            _ => {}
        }
        let expected_f = cpu.registers.f;
        loop {
            cpu.step(&mut bus);
            if cpu.is_between_instructions() || cpu.lockup().is_some() {
                break;
            }
        }
        assert_eq!(
            cpu.registers.f, expected_f,
            "flags nao devem ser afetadas para opcode {opcode:#04X}"
        );
    }
}

// --- stack ---

#[test]
fn ret_conditional_not_taken_does_not_modify_stack() {
    for opcode in [0xC0u8, 0xC8, 0xD0, 0xD8] {
        let (mut cpu, mut bus) = machine_with_stack(&[opcode], 0xC000, 0x0500);
        match opcode {
            0xC0 => cpu.registers.f |= 0b1000_0000,  // Z=1, NZ nao toma
            0xC8 => cpu.registers.f &= !0b1000_0000, // Z=0, Z nao toma
            0xD0 => cpu.registers.f |= 0b0001_0000,  // C=1, NC nao toma
            _ => cpu.registers.f &= !0b0001_0000,    // C=0, C nao toma
        }
        let sp_antes = cpu.registers.sp;
        loop {
            cpu.step(&mut bus);
            if cpu.is_between_instructions() || cpu.lockup().is_some() {
                break;
            }
        }
        assert_eq!(
            cpu.registers.sp, sp_antes,
            "SP nao deve mudar sem desvio para {opcode:#04X}"
        );
    }
}

// --- RETI ($D9) ---

#[test]
fn reti_behaves_like_ret_and_sets_ime() {
    let ret_addr = 0x0500u16;
    let (mut cpu, mut bus) = machine_with_stack(&[0xD9], 0xC000, ret_addr);
    let sp_antes = initial_sp(&cpu);
    for _ in 0..4 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ret_addr);
    assert_eq!(cpu.registers.sp, sp_antes.wrapping_add(2));
    assert!(cpu.ime, "RETI deve habilitar IME");
}

#[test]
fn reti_takes_four_m_cycles() {
    let ret_addr = 0x0500u16;
    let (mut cpu, mut bus) = machine_with_stack(&[0xD9], 0xC000, ret_addr);

    cpu.step(&mut bus); // M1: fetch
    assert!(!cpu.is_between_instructions());

    cpu.step(&mut bus); // M2: read((SP++)->lower)
    assert!(!cpu.is_between_instructions());

    cpu.step(&mut bus); // M3: read((SP++)->upper)
    assert!(!cpu.is_between_instructions());

    cpu.step(&mut bus); // M4: internal(set PC, IME=1)
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ret_addr);
    assert!(cpu.ime, "IME deve ser 1 apos RETI");
}

#[test]
fn reti_does_not_affect_flags() {
    const DIRTY_F: u8 = 0b1010_0000;
    let (mut cpu, mut bus) = machine_with_stack(&[0xD9], 0xC000, 0x0500);
    cpu.registers.f = DIRTY_F;
    loop {
        cpu.step(&mut bus);
        if cpu.is_between_instructions() || cpu.lockup().is_some() {
            break;
        }
    }
    assert_eq!(cpu.registers.f, DIRTY_F);
}

#[test]
fn ret_does_not_set_ime() {
    let ret_addr = 0x0500u16;
    let (mut cpu, mut bus) = machine_with_stack(&[0xC9], 0xC000, ret_addr);
    for _ in 0..4 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert!(!cpu.ime, "RET nao deve habilitar IME");
}

// --- todos os opcodes RET reconhecidos ---

#[test]
fn all_ret_opcodes_are_recognized_and_do_not_lock_up() {
    let ret_opcodes = [0xC0u8, 0xC8, 0xC9, 0xD0, 0xD8, 0xD9];
    for &opcode in &ret_opcodes {
        let (mut cpu, mut bus) = machine_with_stack(&[opcode], 0xC000, 0x0500);
        match opcode {
            0xC0 | 0xD0 => cpu.registers.f &= !(0b1000_0000 | 0b0001_0000),
            0xC8 => cpu.registers.f |= 0b1000_0000,
            0xD8 => cpu.registers.f |= 0b0001_0000,
            _ => {}
        }
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
fn ret_opcodes_the_rest_of_block_is_still_undecoded_or_illegal() {
    let ret_opcodes = [0xC0u8, 0xC8, 0xC9, 0xD0, 0xD8, 0xD9];
    const ILLEGAL: [u8; 11] = [
        0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
    ];
    for opcode in 0x00..=0xFFu8 {
        if ret_opcodes.contains(&opcode) {
            continue;
        }
        let mut rom = vec![0x00u8; NoMbc::MAX_ROM_LEN];
        rom[ENTRY as usize] = opcode;
        rom[ENTRY as usize + 1] = 0x00;
        rom[ENTRY as usize + 2] = 0x00;
        rom[ENTRY as usize + 3] = 0x00;
        let (mut cpu, mut bus) = machine_with_rom(rom);

        for _ in 0..6 {
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
fn ret_opcodes_now_known_in_decoded_elsewhere() {
    let ret_opcodes = [0xC0u8, 0xC8, 0xC9, 0xD0, 0xD8, 0xD9];
    for &opcode in &ret_opcodes {
        assert!(
            support::decoded_elsewhere(opcode),
            "opcode {opcode:#04X} deve estar em decoded_elsewhere depois da implementacao"
        );
    }
}
