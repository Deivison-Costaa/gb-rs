//! ROADMAP 1.6e — `INC r8` e `DEC r8`: os blocos `00 ddd 100` e `00 ddd 101`,
//! 16 opcodes.
//!
//! Primeira operação da ALU que deixa uma coluna de flag **intocada**: `C` não
//! é `0`/`1` literal como no 1.6c, nem calculada como no 1.6a/1.6b — fica
//! exatamente como estava antes do opcode. `$34`/`$35` (`INC`/`DEC (HL)`) são
//! 3 M-cycles, `fetch → read((HL)) → write((HL))` — o mesmo endereço em dois
//! passos diferentes, como o `$36` do 1.4b (erro #1 da 0015 numa forma nova).

use gb_core::bus::Bus;
use gb_core::cart::{CartridgeHeader, NoMbc};
use gb_core::cpu::{Cpu, Flag, Lockup, Registers};

mod support;
use support::decoded_elsewhere;

const ENTRY: usize = 0x0100;

const OPERAND_SLOT: u16 = 0xC7A0;

const SEED_A: u8 = 0x21;
const SEED: [u8; 6] = [0x31, 0x42, 0x53, 0x64, 0xC7, 0xA0];

const R8_NAMES: [&str; 8] = ["B", "C", "D", "E", "H", "L", "(HL)", "A"];

const HL_INDEX: u8 = 6;

const fn inc_r8(r8: u8) -> u8 {
    0b0000_0100 | (r8 << 3)
}

const fn dec_r8(r8: u8) -> u8 {
    0b0000_0101 | (r8 << 3)
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

fn seed(registers: &mut Registers) {
    registers.a = SEED_A;
    registers.b = SEED[0];
    registers.c = SEED[1];
    registers.d = SEED[2];
    registers.e = SEED[3];
    registers.h = SEED[4];
    registers.l = SEED[5];
}

fn write_r8(registers: &mut Registers, r8: u8, value: u8, bus: &mut Bus) {
    match r8 {
        0 => registers.b = value,
        1 => registers.c = value,
        2 => registers.d = value,
        3 => registers.e = value,
        4 => registers.h = value,
        5 => registers.l = value,
        6 => bus.write(OPERAND_SLOT, value),
        _ => registers.a = value,
    }
}

fn read_r8(registers: &Registers, r8: u8, bus: &Bus) -> u8 {
    match r8 {
        0 => registers.b,
        1 => registers.c,
        2 => registers.d,
        3 => registers.e,
        4 => registers.h,
        5 => registers.l,
        6 => bus.read(OPERAND_SLOT),
        _ => registers.a,
    }
}

fn m_cycles_of(r8: u8) -> usize {
    if r8 == HL_INDEX { 3 } else { 1 }
}

fn run(cpu: &mut Cpu, bus: &mut Bus, r8: u8) {
    if r8 == HL_INDEX {
        cpu.registers.set_hl(OPERAND_SLOT);
    }
    for _ in 0..m_cycles_of(r8) {
        cpu.step(bus);
    }
}

#[test]
fn the_block_2_layout_gives_the_sixteen_opcodes_gbops_lists() {
    let inc: Vec<u8> = (0..8).map(inc_r8).collect();
    let dec: Vec<u8> = (0..8).map(dec_r8).collect();

    assert_eq!(
        inc,
        vec![0x04, 0x0C, 0x14, 0x1C, 0x24, 0x2C, 0x34, 0x3C],
        "`00 ddd 100` do `02-cpu.md`: o `INC` em 5-3 é o registrador, 2-0 fixo em `100`"
    );
    assert_eq!(
        dec,
        vec![0x05, 0x0D, 0x15, 0x1D, 0x25, 0x2D, 0x35, 0x3D],
        "`00 ddd 101`: mesmo campo `ddd`, os três bits baixos em `101`"
    );
}

#[test]
fn inc_adds_one_wrapping_to_the_operand() {
    for r8 in 0..8u8 {
        let opcode = inc_r8(r8);
        let (mut cpu, mut bus) = machine(&[opcode]);
        seed(&mut cpu.registers);
        write_r8(&mut cpu.registers, r8, 0xFE, &mut bus);

        run(&mut cpu, &mut bus, r8);

        assert_eq!(
            read_r8(&cpu.registers, r8, &bus),
            0xFF,
            "${opcode:02X} é `INC {}`: 0xFE + 1 = 0xFF",
            R8_NAMES[r8 as usize]
        );
    }
}

#[test]
fn inc_wraps_from_ff_to_00() {
    for r8 in 0..8u8 {
        let opcode = inc_r8(r8);
        let (mut cpu, mut bus) = machine(&[opcode]);
        seed(&mut cpu.registers);
        write_r8(&mut cpu.registers, r8, 0xFF, &mut bus);

        run(&mut cpu, &mut bus, r8);

        assert_eq!(
            read_r8(&cpu.registers, r8, &bus),
            0x00,
            "${opcode:02X}: `INC {}` em 0xFF dá a volta para 0x00, não estoura o tipo",
            R8_NAMES[r8 as usize]
        );
    }
}

#[test]
fn dec_subtracts_one_wrapping_to_the_operand() {
    for r8 in 0..8u8 {
        let opcode = dec_r8(r8);
        let (mut cpu, mut bus) = machine(&[opcode]);
        seed(&mut cpu.registers);
        write_r8(&mut cpu.registers, r8, 0x01, &mut bus);

        run(&mut cpu, &mut bus, r8);

        assert_eq!(
            read_r8(&cpu.registers, r8, &bus),
            0x00,
            "${opcode:02X} é `DEC {}`: 0x01 - 1 = 0x00",
            R8_NAMES[r8 as usize]
        );
    }
}

#[test]
fn dec_wraps_from_00_to_ff() {
    for r8 in 0..8u8 {
        let opcode = dec_r8(r8);
        let (mut cpu, mut bus) = machine(&[opcode]);
        seed(&mut cpu.registers);
        write_r8(&mut cpu.registers, r8, 0x00, &mut bus);

        run(&mut cpu, &mut bus, r8);

        assert_eq!(
            read_r8(&cpu.registers, r8, &bus),
            0xFF,
            "${opcode:02X}: `DEC {}` em 0x00 dá a volta para 0xFF",
            R8_NAMES[r8 as usize]
        );
    }
}

#[test]
fn inc_sets_n_to_zero_and_dec_sets_n_to_one_no_matter_the_result() {
    for r8 in 0..8u8 {
        let (mut cpu, mut bus) = machine(&[inc_r8(r8)]);
        seed(&mut cpu.registers);
        cpu.registers.f = 0xF0;
        write_r8(&mut cpu.registers, r8, 0x10, &mut bus);
        run(&mut cpu, &mut bus, r8);
        assert!(
            !cpu.registers.flag(Flag::N),
            "INC {}: N é `0` literal na coluna",
            R8_NAMES[r8 as usize]
        );

        let (mut cpu, mut bus) = machine(&[dec_r8(r8)]);
        seed(&mut cpu.registers);
        cpu.registers.f = 0x00;
        write_r8(&mut cpu.registers, r8, 0x10, &mut bus);
        run(&mut cpu, &mut bus, r8);
        assert!(
            cpu.registers.flag(Flag::N),
            "DEC {}: N é `1` literal na coluna",
            R8_NAMES[r8 as usize]
        );
    }
}

#[test]
fn inc_and_dec_set_zero_only_when_the_result_is_zero() {
    for r8 in 0..8u8 {
        // INC $FF -> $00: Z liga.
        let (mut cpu, mut bus) = machine(&[inc_r8(r8)]);
        seed(&mut cpu.registers);
        cpu.registers.f = 0x00;
        write_r8(&mut cpu.registers, r8, 0xFF, &mut bus);
        run(&mut cpu, &mut bus, r8);
        assert!(
            cpu.registers.flag(Flag::Z),
            "INC {} de 0xFF dá 0x00: Z liga",
            R8_NAMES[r8 as usize]
        );

        // INC $00 -> $01: Z não liga — controle do caso acima.
        let (mut cpu, mut bus) = machine(&[inc_r8(r8)]);
        seed(&mut cpu.registers);
        cpu.registers.f = 0xF0;
        write_r8(&mut cpu.registers, r8, 0x00, &mut bus);
        run(&mut cpu, &mut bus, r8);
        assert!(
            !cpu.registers.flag(Flag::Z),
            "INC {} de 0x00 dá 0x01: Z não liga",
            R8_NAMES[r8 as usize]
        );

        // DEC $01 -> $00: Z liga.
        let (mut cpu, mut bus) = machine(&[dec_r8(r8)]);
        seed(&mut cpu.registers);
        cpu.registers.f = 0x00;
        write_r8(&mut cpu.registers, r8, 0x01, &mut bus);
        run(&mut cpu, &mut bus, r8);
        assert!(
            cpu.registers.flag(Flag::Z),
            "DEC {} de 0x01 dá 0x00: Z liga",
            R8_NAMES[r8 as usize]
        );

        // DEC $02 -> $01: Z não liga — controle do caso acima.
        let (mut cpu, mut bus) = machine(&[dec_r8(r8)]);
        seed(&mut cpu.registers);
        cpu.registers.f = 0xF0;
        write_r8(&mut cpu.registers, r8, 0x02, &mut bus);
        run(&mut cpu, &mut bus, r8);
        assert!(
            !cpu.registers.flag(Flag::Z),
            "DEC {} de 0x02 dá 0x01: Z não liga",
            R8_NAMES[r8 as usize]
        );
    }
}

#[test]
fn inc_sets_half_carry_exactly_at_the_low_nibble_boundary() {
    for r8 in 0..8u8 {
        // 0x0F -> 0x10: estoura o nibble baixo, H liga.
        let (mut cpu, mut bus) = machine(&[inc_r8(r8)]);
        seed(&mut cpu.registers);
        cpu.registers.f = 0x00;
        write_r8(&mut cpu.registers, r8, 0x0F, &mut bus);
        run(&mut cpu, &mut bus, r8);
        assert!(
            cpu.registers.flag(Flag::H),
            "INC {} de 0x0F: nibble baixo 0xF + 1 estoura, H liga",
            R8_NAMES[r8 as usize]
        );

        // 0x0E -> 0x0F: não estoura — controle do caso acima.
        let (mut cpu, mut bus) = machine(&[inc_r8(r8)]);
        seed(&mut cpu.registers);
        cpu.registers.f = 0xF0;
        write_r8(&mut cpu.registers, r8, 0x0E, &mut bus);
        run(&mut cpu, &mut bus, r8);
        assert!(
            !cpu.registers.flag(Flag::H),
            "INC {} de 0x0E: nibble baixo 0xE + 1 não estoura, H não liga",
            R8_NAMES[r8 as usize]
        );
    }
}

#[test]
fn dec_sets_half_borrow_exactly_at_the_low_nibble_boundary() {
    for r8 in 0..8u8 {
        // 0x10 -> 0x0F: pede emprestado do nibble alto, H liga.
        let (mut cpu, mut bus) = machine(&[dec_r8(r8)]);
        seed(&mut cpu.registers);
        cpu.registers.f = 0x00;
        write_r8(&mut cpu.registers, r8, 0x10, &mut bus);
        run(&mut cpu, &mut bus, r8);
        assert!(
            cpu.registers.flag(Flag::H),
            "DEC {} de 0x10: nibble baixo 0x0 - 1 pede emprestado, H liga",
            R8_NAMES[r8 as usize]
        );

        // 0x11 -> 0x10: não pede emprestado — controle do caso acima.
        let (mut cpu, mut bus) = machine(&[dec_r8(r8)]);
        seed(&mut cpu.registers);
        cpu.registers.f = 0xF0;
        write_r8(&mut cpu.registers, r8, 0x11, &mut bus);
        run(&mut cpu, &mut bus, r8);
        assert!(
            !cpu.registers.flag(Flag::H),
            "DEC {} de 0x11: nibble baixo 0x1 - 1 não pede emprestado, H não liga",
            R8_NAMES[r8 as usize]
        );
    }
}

// O ponto inteiro do 1.6e: `C` fica como estava, não é `0`/`1` literal (1.6c)
// nem calculado (1.6a/1.6b). Os dois casos abaixo distinguem "preservado" de
// "calculado como se fosse ADD/SUB de verdade": 0xFF+1 estoura o byte inteiro
// (um C "genérico" ligaria), e 0x01+1 não estoura nada (um C que zerasse por
// padrão apagaria o que já estava ligado).
#[test]
fn neither_inc_nor_dec_touches_the_carry_flag() {
    for r8 in 0..8u8 {
        // INC que estoura o byte inteiro (0xFF->0x00) com C começando limpo:
        // um C calculado por carry aritmético ligaria aqui — o correto é ficar limpo.
        let (mut cpu, mut bus) = machine(&[inc_r8(r8)]);
        seed(&mut cpu.registers);
        cpu.registers.set_flag(Flag::C, false);
        write_r8(&mut cpu.registers, r8, 0xFF, &mut bus);
        run(&mut cpu, &mut bus, r8);
        assert!(
            !cpu.registers.flag(Flag::C),
            "INC {} de 0xFF: C não é calculado — continua limpo",
            R8_NAMES[r8 as usize]
        );

        // INC sem estouro nenhum (0x01->0x02) com C começando ligado: um C que
        // fosse zerado incondicionalmente apagaria isso — o correto é continuar ligado.
        let (mut cpu, mut bus) = machine(&[inc_r8(r8)]);
        seed(&mut cpu.registers);
        cpu.registers.set_flag(Flag::C, true);
        write_r8(&mut cpu.registers, r8, 0x01, &mut bus);
        run(&mut cpu, &mut bus, r8);
        assert!(
            cpu.registers.flag(Flag::C),
            "INC {} de 0x01: C não é calculado — continua ligado",
            R8_NAMES[r8 as usize]
        );

        // DEC que estoura o byte inteiro (0x00->0xFF) com C começando limpo:
        // um C calculado por empréstimo ligaria aqui — o correto é ficar limpo.
        let (mut cpu, mut bus) = machine(&[dec_r8(r8)]);
        seed(&mut cpu.registers);
        cpu.registers.set_flag(Flag::C, false);
        write_r8(&mut cpu.registers, r8, 0x00, &mut bus);
        run(&mut cpu, &mut bus, r8);
        assert!(
            !cpu.registers.flag(Flag::C),
            "DEC {} de 0x00: C não é calculado — continua limpo",
            R8_NAMES[r8 as usize]
        );

        // DEC sem empréstimo nenhum (0x02->0x01) com C começando ligado.
        let (mut cpu, mut bus) = machine(&[dec_r8(r8)]);
        seed(&mut cpu.registers);
        cpu.registers.set_flag(Flag::C, true);
        write_r8(&mut cpu.registers, r8, 0x02, &mut bus);
        run(&mut cpu, &mut bus, r8);
        assert!(
            cpu.registers.flag(Flag::C),
            "DEC {} de 0x02: C não é calculado — continua ligado",
            R8_NAMES[r8 as usize]
        );
    }
}

#[test]
fn inc_and_dec_hl_are_three_m_cycles_and_the_write_is_the_third() {
    const BEFORE: u8 = 0x0F;
    const MUTATED: u8 = 0x2F;

    for opcode in [0x34u8, 0x35] {
        let (mut cpu, mut bus) = machine(&[opcode]);
        seed(&mut cpu.registers);
        cpu.registers.set_hl(OPERAND_SLOT);
        bus.write(OPERAND_SLOT, BEFORE);

        cpu.step(&mut bus);
        assert_eq!(
            bus.read(OPERAND_SLOT),
            BEFORE,
            "${opcode:02X}: o M1 é só o fetch — nada foi lido ou escrito ainda"
        );
        assert!(
            !cpu.is_between_instructions(),
            "${opcode:02X} tem 12 T-cycles: três M-cycles"
        );

        // Muda a memória entre o fetch e o M2: se a leitura já tivesse
        // acontecido no fetch, o resultado usaria BEFORE, não MUTATED.
        bus.write(OPERAND_SLOT, MUTATED);
        cpu.step(&mut bus);
        assert_eq!(
            bus.read(OPERAND_SLOT),
            MUTATED,
            "${opcode:02X}: o M2 leu — mas a escrita ainda não aconteceu, \
             então a memória continua com o valor mutado, não o computado"
        );
        assert!(
            !cpu.is_between_instructions(),
            "${opcode:02X}: ainda falta o `write((HL))`"
        );

        cpu.step(&mut bus);
        let expected = if opcode == 0x34 {
            MUTATED.wrapping_add(1)
        } else {
            MUTATED.wrapping_sub(1)
        };
        assert_eq!(
            bus.read(OPERAND_SLOT),
            expected,
            "${opcode:02X}: o M3 escreve o resultado calculado sobre o valor \
             lido no M2 (MUTATED), não sobre o valor de antes do fetch (BEFORE)"
        );
        assert!(
            cpu.is_between_instructions(),
            "${opcode:02X}: três M-cycles e acabou"
        );
    }
}

#[test]
fn inc_and_dec_of_a_plain_register_take_a_single_m_cycle() {
    for r8 in 0..8u8 {
        if r8 == HL_INDEX {
            continue;
        }
        for opcode in [inc_r8(r8), dec_r8(r8)] {
            let (mut cpu, mut bus) = machine(&[opcode]);
            seed(&mut cpu.registers);

            cpu.step(&mut bus);

            assert!(
                cpu.is_between_instructions(),
                "${opcode:02X}: `INC`/`DEC {}` é `fetch` só — 4 T-cycles, um M-cycle",
                R8_NAMES[r8 as usize]
            );
        }
    }
}

#[test]
fn inc_and_dec_touch_only_the_operand_the_flags_and_the_program_counter() {
    for r8 in 0..8u8 {
        for opcode in [inc_r8(r8), dec_r8(r8)] {
            let (mut cpu, mut bus) = machine(&[opcode]);
            seed(&mut cpu.registers);
            bus.write(OPERAND_SLOT, 0x0C);
            if r8 == HL_INDEX {
                cpu.registers.set_hl(OPERAND_SLOT);
            }

            let mut expected = cpu.registers;

            run(&mut cpu, &mut bus, r8);

            expected.pc = 0x0101;
            expected.f = cpu.registers.f;
            match r8 {
                0 => expected.b = cpu.registers.b,
                1 => expected.c = cpu.registers.c,
                2 => expected.d = cpu.registers.d,
                3 => expected.e = cpu.registers.e,
                4 => expected.h = cpu.registers.h,
                5 => expected.l = cpu.registers.l,
                6 => {}
                _ => expected.a = cpu.registers.a,
            }

            assert_eq!(
                cpu.registers, expected,
                "${opcode:02X}: só o operando, `F` e `PC` mudam"
            );
        }
    }
}

#[test]
fn inc_dec_of_hl_content_does_not_touch_the_hl_register_itself() {
    for opcode in [0x34u8, 0x35] {
        let (mut cpu, mut bus) = machine(&[opcode]);
        seed(&mut cpu.registers);
        cpu.registers.set_hl(OPERAND_SLOT);

        cpu.step(&mut bus);
        cpu.step(&mut bus);
        cpu.step(&mut bus);

        assert_eq!(
            cpu.registers.hl(),
            OPERAND_SLOT,
            "${opcode:02X}: `HL` é o endereço, não o operando — quem muda é o byte na memória"
        );
    }
}

#[test]
fn the_blocks_this_item_decodes_are_exactly_the_sixteen_opcodes_of_100_and_101() {
    const ILLEGAL: [u8; 11] = [
        0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
    ];

    let in_this_block: Vec<u8> = (0..8).map(inc_r8).chain((0..8).map(dec_r8)).collect();

    for opcode in 0x00..=0xFFu8 {
        let (mut cpu, mut bus) = machine(&[opcode, 0x00, 0x00]);
        seed(&mut cpu.registers);

        for _ in 0..3 {
            cpu.step(&mut bus);
        }

        if in_this_block.contains(&opcode) {
            assert_eq!(
                cpu.lockup(),
                None,
                "${opcode:02X} é um dos dezesseis `INC`/`DEC r8` e o 1.6e o decodifica"
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
