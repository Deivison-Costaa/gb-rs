//! spec: docs/reference/03-opcodes.md § control/br (CALL $C4 $CC $CD $D4 $DC)
//! iteracao 0041.

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

fn initial_sp(cpu: &Cpu) -> u16 {
    cpu.registers.sp
}

// --- CALL u16 ($CD) ---

#[test]
fn call_u16_jumps_to_dest_and_pushes_return_address() {
    let dest = 0x0500u16;
    let (mut cpu, mut bus) = machine(&[0xCD, dest as u8, (dest >> 8) as u8]);
    let sp_antes = initial_sp(&cpu);
    let ret_addr = ENTRY + 3;
    for _ in 0..6 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, dest);
    assert_eq!(cpu.registers.sp, sp_antes.wrapping_sub(2));
    assert_eq!(bus.read(sp_antes.wrapping_sub(1)), (ret_addr >> 8) as u8);
    assert_eq!(bus.read(sp_antes.wrapping_sub(2)), (ret_addr & 0xFF) as u8);
}

#[test]
fn call_u16_does_not_affect_flags() {
    const DIRTY_F: u8 = 0b1010_0000;
    let (mut cpu, mut bus) = machine(&[0xCD, 0x00, 0x05]);
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
fn call_u16_takes_six_m_cycles() {
    let (mut cpu, mut bus) = machine(&[0xCD, 0x34, 0x12]);
    let sp_antes = initial_sp(&cpu);

    cpu.step(&mut bus); // M1: fetch
    assert!(!cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 1);

    cpu.step(&mut bus); // M2: read(u16:lower)
    assert!(!cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 2);

    cpu.step(&mut bus); // M3: read(u16:upper)
    assert!(!cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 3);

    cpu.step(&mut bus); // M4: internal
    assert!(!cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 3, "PC segue ret_addr no internal");

    cpu.step(&mut bus); // M5: write(PC:upper->(--SP))
    assert!(!cpu.is_between_instructions());
    assert_eq!(cpu.registers.sp, sp_antes.wrapping_sub(1));

    cpu.step(&mut bus); // M6: write(PC:lower->(--SP)), PC = dest
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, 0x1234);
    assert_eq!(cpu.registers.sp, sp_antes.wrapping_sub(2));
}

// --- CALL NZ,u16 ($C4) ---

#[test]
fn call_nz_taken_when_z_is_clear() {
    let dest = 0x0500u16;
    let (mut cpu, mut bus) = machine(&[0xC4, dest as u8, (dest >> 8) as u8]);
    cpu.registers.f &= !0b1000_0000;
    let sp_antes = initial_sp(&cpu);
    for _ in 0..6 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, dest);
    assert_eq!(cpu.registers.sp, sp_antes.wrapping_sub(2));
}

#[test]
fn call_nz_not_taken_when_z_is_set() {
    let (mut cpu, mut bus) = machine(&[0xC4, 0x00, 0x05]);
    cpu.registers.f |= 0b1000_0000;
    let sp_antes = initial_sp(&cpu);
    for _ in 0..3 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 3);
    assert_eq!(cpu.registers.sp, sp_antes, "SP nao muda sem desvio");
}

// --- CALL Z,u16 ($CC) ---

#[test]
fn call_z_taken_when_z_is_set() {
    let dest = 0x0500u16;
    let (mut cpu, mut bus) = machine(&[0xCC, dest as u8, (dest >> 8) as u8]);
    cpu.registers.f |= 0b1000_0000;
    let sp_antes = initial_sp(&cpu);
    for _ in 0..6 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, dest);
    assert_eq!(cpu.registers.sp, sp_antes.wrapping_sub(2));
}

#[test]
fn call_z_not_taken_when_z_is_clear() {
    let (mut cpu, mut bus) = machine(&[0xCC, 0x00, 0x05]);
    cpu.registers.f &= !0b1000_0000;
    let sp_antes = initial_sp(&cpu);
    for _ in 0..3 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 3);
    assert_eq!(cpu.registers.sp, sp_antes, "SP nao muda sem desvio");
}

// --- CALL NC,u16 ($D4) ---

#[test]
fn call_nc_taken_when_c_is_clear() {
    let dest = 0x0500u16;
    let (mut cpu, mut bus) = machine(&[0xD4, dest as u8, (dest >> 8) as u8]);
    cpu.registers.f &= !0b0001_0000;
    let sp_antes = initial_sp(&cpu);
    for _ in 0..6 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, dest);
    assert_eq!(cpu.registers.sp, sp_antes.wrapping_sub(2));
}

#[test]
fn call_nc_not_taken_when_c_is_set() {
    let (mut cpu, mut bus) = machine(&[0xD4, 0x00, 0x05]);
    cpu.registers.f |= 0b0001_0000;
    let sp_antes = initial_sp(&cpu);
    for _ in 0..3 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 3);
    assert_eq!(cpu.registers.sp, sp_antes, "SP nao muda sem desvio");
}

// --- CALL C,u16 ($DC) ---

#[test]
fn call_c_taken_when_c_is_set() {
    let dest = 0x0500u16;
    let (mut cpu, mut bus) = machine(&[0xDC, dest as u8, (dest >> 8) as u8]);
    cpu.registers.f |= 0b0001_0000;
    let sp_antes = initial_sp(&cpu);
    for _ in 0..6 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, dest);
    assert_eq!(cpu.registers.sp, sp_antes.wrapping_sub(2));
}

#[test]
fn call_c_not_taken_when_c_is_clear() {
    let (mut cpu, mut bus) = machine(&[0xDC, 0x00, 0x05]);
    cpu.registers.f &= !0b0001_0000;
    let sp_antes = initial_sp(&cpu);
    for _ in 0..3 {
        cpu.step(&mut bus);
    }
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 3);
    assert_eq!(cpu.registers.sp, sp_antes, "SP nao muda sem desvio");
}

// --- timing condicional ---

#[test]
fn call_conditional_takes_three_m_cycles_when_not_taken() {
    let (mut cpu, mut bus) = machine(&[0xC4, 0x00, 0x05]);
    cpu.registers.f |= 0b1000_0000;
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
fn call_conditional_takes_six_m_cycles_when_taken() {
    let (mut cpu, mut bus) = machine(&[0xC4, 0x34, 0x12]);
    cpu.registers.f &= !0b1000_0000;
    let sp_antes = initial_sp(&cpu);

    cpu.step(&mut bus); // M1: fetch
    cpu.step(&mut bus); // M2: read(u16:lower)
    cpu.step(&mut bus); // M3: read(u16:upper)
    assert!(!cpu.is_between_instructions(), "com desvio M3 nao finaliza");
    assert_eq!(cpu.registers.pc, ENTRY + 3);

    cpu.step(&mut bus); // M4: internal
    assert!(!cpu.is_between_instructions());

    cpu.step(&mut bus); // M5: write(PC:upper->(--SP))
    assert!(!cpu.is_between_instructions());
    assert_eq!(cpu.registers.sp, sp_antes.wrapping_sub(1));

    cpu.step(&mut bus); // M6: write(PC:lower->(--SP)), PC = dest
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, 0x1234);
    assert_eq!(cpu.registers.sp, sp_antes.wrapping_sub(2));
}

// --- flags (todos os opcodes) ---

#[test]
fn call_conditional_does_not_affect_flags() {
    const DIRTY_F: u8 = 0b1010_0000;
    for condition_opcode in [0xC4u8, 0xCC, 0xD4, 0xDC, 0xCD] {
        let (mut cpu, mut bus) = machine(&[condition_opcode, 0x05, 0x00]);
        cpu.registers.f = DIRTY_F;
        let expected_f = cpu.registers.f;
        match condition_opcode {
            0xC4 => cpu.registers.set_flag(Flag::Z, false),
            0xCC => cpu.registers.set_flag(Flag::Z, true),
            0xD4 => cpu.registers.set_flag(Flag::C, false),
            0xDC => cpu.registers.set_flag(Flag::C, true),
            _ => {}
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

// --- stack: conteudo e ordem ---

#[test]
fn call_pushes_high_byte_first_then_low_byte() {
    let dest = 0x0678u16;
    let (mut cpu, mut bus) = machine(&[0xCD, dest as u8, (dest >> 8) as u8]);
    let sp_antes = initial_sp(&cpu);
    let ret_addr = ENTRY + 3;
    for _ in 0..6 {
        cpu.step(&mut bus);
    }
    // high byte no endereco mais alto (sp_antes - 1), low byte no mais baixo (sp_antes - 2)
    assert_eq!(bus.read(sp_antes.wrapping_sub(1)), (ret_addr >> 8) as u8);
    assert_eq!(bus.read(sp_antes.wrapping_sub(2)), (ret_addr & 0xFF) as u8);
}

// --- CALL u16 com SP no limite ---

#[test]
fn call_u16_with_sp_at_end_of_wram_decrements_correctly() {
    let dest = 0x0300u16;
    let (mut cpu, mut bus) = machine(&[0xCD, dest as u8, (dest >> 8) as u8]);
    cpu.registers.sp = 0xDFFF;
    let sp_antes = cpu.registers.sp;
    for _ in 0..6 {
        cpu.step(&mut bus);
    }
    assert_eq!(cpu.registers.pc, dest);
    assert_eq!(cpu.registers.sp, sp_antes.wrapping_sub(2));
}

// --- CALL condicional sem desvio nao modifica pilha ---

#[test]
fn call_not_taken_does_not_modify_stack() {
    for condition_opcode in [0xC4u8, 0xCC, 0xD4, 0xDC] {
        let (mut cpu, mut bus) = machine(&[condition_opcode, 0x00, 0x05]);
        match condition_opcode {
            0xC4 => cpu.registers.f |= 0b1000_0000,  // Z=1, NZ nao toma
            0xCC => cpu.registers.f &= !0b1000_0000, // Z=0, Z nao toma
            0xD4 => cpu.registers.f |= 0b0001_0000,  // C=1, NC nao toma
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
            "SP nao deve mudar sem desvio para {condition_opcode:#04X}"
        );
    }
}

// --- todos os opcodes CALL reconhecidos ---

#[test]
fn all_call_opcodes_are_recognized_and_do_not_lock_up() {
    let call_opcodes = [0xC4u8, 0xCC, 0xCD, 0xD4, 0xDC];
    for &opcode in &call_opcodes {
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
fn call_opcodes_the_rest_of_block_is_still_undecoded_or_illegal() {
    let call_opcodes = [0xC4u8, 0xCC, 0xCD, 0xD4, 0xDC];
    const ILLEGAL: [u8; 11] = [
        0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
    ];
    for opcode in 0x00..=0xFFu8 {
        if call_opcodes.contains(&opcode) {
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
fn call_opcodes_now_known_in_decoded_elsewhere() {
    let call_opcodes = [0xC4u8, 0xCC, 0xCD, 0xD4, 0xDC];
    for &opcode in &call_opcodes {
        assert!(
            support::decoded_elsewhere(opcode),
            "opcode {opcode:#04X} deve estar em decoded_elsewhere depois da implementacao"
        );
    }
}
