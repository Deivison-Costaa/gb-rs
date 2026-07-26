//! spec: docs/reference/03-opcodes.md § control/br (RST $C7 $CF $D7 $DF $E7 $EF $F7 $FF)
//! iteracao 0043.

use gb_core::bus::Bus;
use gb_core::cart::CartridgeHeader;
use gb_core::cart::NoMbc;
use gb_core::cpu::{Cpu, Lockup};
use std::boxed::Box;

mod support;

const ENTRY: u16 = 0x0100;

fn machine(opcode: u8) -> (Cpu, Bus) {
    let mut rom = vec![0x00; NoMbc::MAX_ROM_LEN];
    rom[ENTRY as usize] = opcode;
    let checksum = CartridgeHeader::parse(&rom)
        .expect("header valido")
        .checksum();
    let cartridge = NoMbc::new(rom).expect("ROM 32KB valida");
    (Cpu::after_boot_rom(checksum), Bus::new(Box::new(cartridge)))
}

fn rst_opcodes() -> [(u8, u16); 8] {
    [
        (0xC7, 0x0000),
        (0xCF, 0x0008),
        (0xD7, 0x0010),
        (0xDF, 0x0018),
        (0xE7, 0x0020),
        (0xEF, 0x0028),
        (0xF7, 0x0030),
        (0xFF, 0x0038),
    ]
}

// --- salta para o endereco correto ---

#[test]
fn rst_jumps_to_destination_and_pushes_return_address() {
    for (opcode, dest) in rst_opcodes() {
        let (mut cpu, mut bus) = machine(opcode);
        let sp_antes = cpu.registers.sp;
        let ret_addr = ENTRY + 1;
        for _ in 0..4 {
            cpu.step(&mut bus);
        }
        assert!(cpu.is_between_instructions());
        assert_eq!(
            cpu.registers.pc, dest,
            "RST {opcode:#04X} deve saltar para {dest:#06X}"
        );
        assert_eq!(cpu.registers.sp, sp_antes.wrapping_sub(2));
        assert_eq!(bus.read(sp_antes.wrapping_sub(1)), (ret_addr >> 8) as u8);
        assert_eq!(bus.read(sp_antes.wrapping_sub(2)), (ret_addr & 0xFF) as u8);
    }
}

// --- flags ---

#[test]
fn rst_does_not_affect_flags() {
    const DIRTY_F: u8 = 0b1010_0000;
    for (opcode, _) in rst_opcodes() {
        let (mut cpu, mut bus) = machine(opcode);
        cpu.registers.f = DIRTY_F;
        loop {
            cpu.step(&mut bus);
            if cpu.is_between_instructions() || cpu.lockup().is_some() {
                break;
            }
        }
        assert_eq!(
            cpu.registers.f, DIRTY_F,
            "flags nao devem ser afetadas para opcode {opcode:#04X}"
        );
    }
}

// --- quatro M-cycles ---

#[test]
fn rst_takes_four_m_cycles() {
    let (opcode, dest) = (0xC7, 0x0000u16);
    let (mut cpu, mut bus) = machine(opcode);
    let sp_antes = cpu.registers.sp;

    cpu.step(&mut bus); // M1: fetch, latcha destino
    assert!(!cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 1, "PC avanca apos fetch");

    cpu.step(&mut bus); // M2: internal
    assert!(!cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, ENTRY + 1, "PC inalterado no internal");
    assert_eq!(cpu.registers.sp, sp_antes, "SP inalterado no internal");

    cpu.step(&mut bus); // M3: write(PC:upper->(--SP))
    assert!(!cpu.is_between_instructions());
    assert_eq!(
        cpu.registers.pc,
        ENTRY + 1,
        "PC segue ret_addr no push high"
    );
    assert_eq!(cpu.registers.sp, sp_antes.wrapping_sub(1));

    cpu.step(&mut bus); // M4: write(PC:lower->(--SP)), PC = dest
    assert!(cpu.is_between_instructions());
    assert_eq!(cpu.registers.pc, dest);
    assert_eq!(cpu.registers.sp, sp_antes.wrapping_sub(2));
}

// --- stack: conteudo e ordem ---

#[test]
fn rst_pushes_high_byte_first_then_low_byte() {
    let (mut cpu, mut bus) = machine(0xCF);
    let sp_antes = cpu.registers.sp;
    let ret_addr = ENTRY + 1;
    for _ in 0..4 {
        cpu.step(&mut bus);
    }
    assert_eq!(bus.read(sp_antes.wrapping_sub(1)), (ret_addr >> 8) as u8);
    assert_eq!(bus.read(sp_antes.wrapping_sub(2)), (ret_addr & 0xFF) as u8);
}

// --- SP no limite ---

#[test]
fn rst_with_sp_at_zero_wraps_correctly() {
    let (mut cpu, mut bus) = machine(0xDF);
    cpu.registers.sp = 0x0000;
    for _ in 0..4 {
        cpu.step(&mut bus);
    }
    assert_eq!(cpu.registers.sp, 0xFFFE);
}

// --- todos os opcodes RST reconhecidos ---

#[test]
fn all_rst_opcodes_are_recognized_and_do_not_lock_up() {
    for (opcode, _) in rst_opcodes() {
        let (mut cpu, mut bus) = machine(opcode);
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

const ILLEGAL: [u8; 11] = [
    0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
];

#[test]
fn rst_opcodes_the_rest_of_block_is_still_undecoded_or_illegal() {
    let rst_set: [u8; 8] = [0xC7, 0xCF, 0xD7, 0xDF, 0xE7, 0xEF, 0xF7, 0xFF];
    for opcode in 0x00..=0xFFu8 {
        if rst_set.contains(&opcode) {
            continue;
        }
        let mut rom = vec![0x00u8; NoMbc::MAX_ROM_LEN];
        rom[ENTRY as usize] = opcode;
        rom[ENTRY as usize + 1] = 0x00;
        rom[ENTRY as usize + 2] = 0x00;
        rom[ENTRY as usize + 3] = 0x00;
        let checksum = CartridgeHeader::parse(&rom)
            .expect("header valido")
            .checksum();
        let cartridge = NoMbc::new(rom).expect("ROM 32KB valida");
        let (mut cpu, mut bus) = (Cpu::after_boot_rom(checksum), Bus::new(Box::new(cartridge)));

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
fn rst_opcodes_now_known_in_decoded_elsewhere() {
    let rst_set: [u8; 8] = [0xC7, 0xCF, 0xD7, 0xDF, 0xE7, 0xEF, 0xF7, 0xFF];
    for &opcode in &rst_set {
        assert!(
            support::decoded_elsewhere(opcode),
            "opcode {opcode:#04X} deve estar em decoded_elsewhere depois da implementacao"
        );
    }
}
