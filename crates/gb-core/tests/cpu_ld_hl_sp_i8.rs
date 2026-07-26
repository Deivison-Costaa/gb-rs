//! ROADMAP 1.7d — `LD HL,SP+i8`: opcode `$F8`, 3 M-cycles.
//!
//! `Z` = `0` e `N` = `0` literais. `H`/`C` calculados sobre o **byte baixo**
//! de `SP` + `i8` (carry do bit 3 e do bit 7), não sobre o par de 16 bits
//! como o `ADD HL,r16` do 1.7b — a mesma armadilha do 1.7c. `i8` é signed:
//! `0xFF` = `-1`. M-cycles: `fetch → read(i8) → internal`.
//! Ao contrário do `$E8`, `SP` **não** é modificado: o destino é `HL`.

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::{Cpu, Flag, Lockup};

mod support;
use support::decoded_elsewhere;

const ENTRY: usize = 0x0100;

const ALL_FLAGS_SET: u8 = 0b1111_0000;
const ALL_FLAGS_CLEAR: u8 = 0b0000_1111;

fn machine(program: &[u8]) -> (Cpu, Bus) {
    let mut rom = vec![0x00; NoMbc::MAX_ROM_LEN];
    rom[ENTRY..ENTRY + program.len()].copy_from_slice(program);

    let checksum = CartridgeHeader::parse(&rom)
        .expect("32 KiB de ROM têm cabeçalho de sobra")
        .checksum();
    let cartridge = NoMbc::new(rom).expect("32 KiB cabem num cartucho sem MBC");

    (Cpu::after_boot_rom(checksum), Bus::new(Box::new(cartridge)))
}

fn sign_extend(b: u8) -> u16 {
    (b as i8) as u16
}

fn h_flag(sp: u16, i8: u8) -> bool {
    let low = (sp & 0xFF) as u8;
    (low & 0x0F) + (i8 & 0x0F) > 0x0F
}

fn c_flag(sp: u16, i8: u8) -> bool {
    let low = (sp & 0xFF) as u8;
    u16::from(low) + u16::from(i8) > 0xFF
}

#[test]
fn the_opcode_is_0xf8() {
    assert_eq!(0xF8u8, 0xF8, "gbops lista o opcode `$F8` — `LD HL,SP+i8`");
}

#[test]
fn ld_adds_signed_i8_to_sp_and_stores_in_hl_without_touching_sp() {
    let cases: &[(u16, u8)] = &[
        (0x1234, 0x01), // +1
        (0x1234, 0x7F), // +127
        (0x1234, 0xFF), // -1
        (0x0000, 0x80), // -128 → 0xFF80
        (0xFF80, 0x80), // wraps to 0xFF00
        (0xFFFF, 0x01), // wrapped to 0x0000
        (0x0001, 0xFF), // wraps down to 0x0000
    ];

    for &(sp, i8) in cases {
        let (mut cpu, mut bus) = machine(&[0xF8, i8]);
        cpu.registers.sp = sp;

        cpu.step(&mut bus);
        cpu.step(&mut bus);
        cpu.step(&mut bus);

        let expected = sp.wrapping_add(sign_extend(i8));
        assert_eq!(
            cpu.registers.hl(),
            expected,
            "`LD HL,SP+{i8:#04X}` ({:+}) com SP={sp:#06X}: HL = {sp:#06X} + {:+} = {expected:#06X}",
            i8 as i8,
            i8 as i8
        );
        assert_eq!(
            cpu.registers.sp, sp,
            "SP não é alterado — `LD HL,SP+i8` lê SP, não o escreve"
        );
    }
}

#[test]
fn ld_zeros_z_and_n_regardless_of_result() {
    for (sp, i8) in [
        (0x1234u16, 0x01u8),
        (0x0000u16, 0x00u8),
        (0xFFFFu16, 0x80u8),
    ] {
        for dirty_f in [ALL_FLAGS_SET, ALL_FLAGS_CLEAR] {
            let (mut cpu, mut bus) = machine(&[0xF8, i8]);
            cpu.registers.sp = sp;
            cpu.registers.f = dirty_f;

            cpu.step(&mut bus);
            cpu.step(&mut bus);
            cpu.step(&mut bus);

            assert!(
                !cpu.registers.flag(Flag::Z),
                "`Z` = `0` literal — a coluna é `0`, SP={sp:#06X} i8={i8:#04X}"
            );
            assert!(
                !cpu.registers.flag(Flag::N),
                "`N` = `0` literal — a coluna é `0`, SP={sp:#06X} i8={i8:#04X}"
            );
        }
    }
}

// Mesma armadilha do 1.7c: `H`/`C` vêm do byte baixo (bit 3 e bit 7),
// não do par inteiro de 16 bits como o 1.7b faria.
#[test]
fn ld_calculates_h_and_c_from_low_byte_only_not_the_full_16_bit_pair() {
    let cases: &[(u16, u8)] = &[
        (0x0FF0, 0x10),
        (0x000F, 0x01),
        (0x00FF, 0x01),
        (0x1234, 0x01),
    ];

    for &(sp, i8) in cases {
        let (mut cpu, mut bus) = machine(&[0xF8, i8]);
        cpu.registers.sp = sp;
        cpu.registers.f = ALL_FLAGS_CLEAR;

        cpu.step(&mut bus);
        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(
            cpu.registers.flag(Flag::H),
            h_flag(sp, i8),
            "SP={sp:#06X} i8={i8:#04X}: `H` é carry do bit 3 da soma do byte baixo"
        );
        assert_eq!(
            cpu.registers.flag(Flag::C),
            c_flag(sp, i8),
            "SP={sp:#06X} i8={i8:#04X}: `C` é carry do bit 7 da soma do byte baixo"
        );
    }
}

#[test]
fn each_half_of_hl_lands_on_the_internal_m_cycle_and_sp_is_never_written() {
    // SP = 0x00FF, i8 = 0x01 (+1): resultado 0x0100.
    // M1: fetch. M2: read(i8), calcula flags, latcha. M3: internal (escreve HL = 0x0100).
    let (mut cpu, mut bus) = machine(&[0xF8, 0x01]);
    cpu.registers.sp = 0x00FF;
    cpu.registers.set_hl(0x0000);

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x0101, "M1: fetch do opcode");
    assert_eq!(cpu.registers.hl(), 0x0000, "M1: HL ainda não foi escrito");

    cpu.step(&mut bus);
    assert_eq!(cpu.registers.pc, 0x0102, "M2: leitura do i8");
    assert_eq!(
        cpu.registers.hl(),
        0x0000,
        "M2: HL ainda não foi escrito — só no M3"
    );
    assert_eq!(
        cpu.registers.sp, 0x00FF,
        "M2: SP não é alterado — `LD HL,SP+i8` não escreve em SP"
    );

    cpu.step(&mut bus);
    assert_eq!(
        cpu.registers.hl(),
        0x0100,
        "M3: HL = SP + i8 = 0x00FF + 1 = 0x0100"
    );
    assert_eq!(
        cpu.registers.sp, 0x00FF,
        "M3: SP permanece inalterado — a instrução é `LD HL,SP+i8`, não `ADD SP,i8`"
    );
    assert!(cpu.is_between_instructions(), "três M-cycles e acabou");
}

#[test]
fn ld_hl_sp_i8_changes_only_hl_flags_and_pc() {
    let (mut cpu, mut bus) = machine(&[0xF8, 0x05]);
    cpu.registers.sp = 0x2000;
    cpu.registers.set_bc(0xAAAA);
    cpu.registers.set_de(0xBBBB);
    cpu.registers.set_hl(0xCCCC);
    cpu.registers.f = ALL_FLAGS_CLEAR;

    let mut expected = cpu.registers;
    expected.set_hl(0x2005);
    expected.pc = 0x0102;
    expected.set_flag(Flag::Z, false);
    expected.set_flag(Flag::N, false);
    expected.set_flag(Flag::H, false);
    expected.set_flag(Flag::C, false);
    expected.set_bc(0xAAAA);
    expected.set_de(0xBBBB);
    // SP não é alterado:
    expected.sp = 0x2000;

    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);

    assert_eq!(
        cpu.registers, expected,
        "`LD HL,SP+i8`: mexe em HL, Z/N/H/C, e PC — SP, BC, DE não são tocados"
    );
}

#[test]
fn the_block_sweep_confirms_only_f8_is_decoded() {
    const ILLEGAL: [u8; 11] = [
        0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
    ];

    for opcode in 0x00..=0xFFu8 {
        let (mut cpu, mut bus) = machine(&[opcode, 0x00, 0x00, 0x00]);

        cpu.step(&mut bus);
        cpu.step(&mut bus);

        if opcode == 0xF8 {
            assert_eq!(
                cpu.lockup(),
                None,
                "$F8 é `LD HL,SP+i8` e o 1.7d o decodifica"
            );
        } else if ILLEGAL.contains(&opcode) {
            assert_eq!(
                cpu.lockup(),
                Some(Lockup::IllegalOpcode(opcode)),
                "{opcode:02X} continua sendo um dos onze que não existem"
            );
        } else if decoded_elsewhere(opcode) {
            assert_eq!(
                cpu.lockup(),
                None,
                "{opcode:02X}: decodificado por outro sub-item"
            );
        } else {
            assert_eq!(
                cpu.lockup(),
                Some(Lockup::UndecodedOpcode(opcode)),
                "{opcode:02X} é opcode legítimo fora deste sub-item"
            );
        }
    }
}
